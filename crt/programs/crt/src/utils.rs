use anchor_lang::prelude::*;
use crate::error::TokenError;
use crate::state::ChronoEquationType;
use crate::tokenizer::Parser;  // Import the custom parser

const TOKEN_DECIMALS: u8 = 9;
const DECIMALS_FACTOR: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

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

    let mut parser = Parser::new(&equation)?;
    parser.set_variable("x", last_snapshot_f);
    parser.set_variable("t", time_diff);

    // Set parameter variables
    if let Some(expiration_time) = params.expiration_time {
        parser.set_variable("expiration_time", expiration_time as f64);
    }
    if let Some(inflation_rate) = params.inflation_rate {
        parser.set_variable("inflation_rate", inflation_rate as f64);
    }
    if let Some(decay_rate) = params.decay_rate {
        parser.set_variable("decay_rate", decay_rate as f64);
    }
    if let Some(time_unit) = params.time_unit {
        parser.set_variable("time_unit", time_unit as f64);
    }
    if let Some(slope) = params.slope {
        parser.set_variable("slope", slope as f64);
    }
    if let Some(decay_constant) = params.decay_constant {
        parser.set_variable("decay_constant", decay_constant);
    }

    parser.evaluate()
        .map(|result| (result * DECIMALS_FACTOR as f64).round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}

// Helper function to parse amount from string
pub fn parse_amount(amount_str: &str) -> Result<u64> {
    amount_str.parse::<u64>().map_err(|_| TokenError::InvalidAmount.into())
}