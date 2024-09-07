use anchor_lang::prelude::*;
use meval::eval_str;
use crate::error::TokenError;
use crate::state::ChronoEquationType;

// Assume these are defined elsewhere in your crate
const TOKEN_DECIMALS: u8 = 9;
const DECIMALS_FACTOR: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

// Helper function to evaluate balance based on equation
pub fn evaluate_balance(
    last_balance_snapshot: u64,
    equation_type: &ChronoEquationType,
    creation_time: i64,
    current_time: i64
) -> Result<u64> {
    let params = equation_type.get_params();
    let equation = equation_type.get_equation_string();

    if equation == "x" {
        return Ok(last_balance_snapshot);
    }

    let time_diff = current_time.saturating_sub(creation_time) as f64;
    let last_snapshot_f = last_balance_snapshot as f64 / DECIMALS_FACTOR as f64;

    let mut equation = equation
        .replace("x", &last_snapshot_f.to_string())
        .replace("t", &time_diff.to_string());

    // Replace parameter placeholders with actual values
    if let Some(expiration_time) = params.expiration_time {
        equation = equation.replace("expiration_time", &expiration_time.to_string());
    }
    if let Some(inflation_rate) = params.inflation_rate {
        equation = equation.replace("inflation_rate", &inflation_rate.to_string());
    }
    if let Some(decay_rate) = params.decay_rate {
        equation = equation.replace("decay_rate", &decay_rate.to_string());
    }
    if let Some(time_unit) = params.time_unit {
        equation = equation.replace("time_unit", &time_unit.to_string());
    }
    if let Some(slope) = params.slope {
        equation = equation.replace("slope", &slope.to_string());
    }
    if let Some(decay_constant) = params.decay_constant {
        equation = equation.replace("decay_constant", &decay_constant.to_string());
    }

    // Add support for exponential function
    let equation = if equation.contains("exp(") {
        equation.replace("exp(", "e^(")
    } else {
        equation
    };

    eval_str(&equation)
        .map(|result| (result * DECIMALS_FACTOR as f64).round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}

// Helper function to parse amount from string
pub fn parse_amount(amount_str: &str) -> Result<u64> {
    amount_str.parse::<u64>().map_err(|_| TokenError::InvalidAmount.into())
}