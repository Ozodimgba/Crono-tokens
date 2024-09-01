use anchor_lang::prelude::*;
use meval::eval_str;
use crate::error::TokenError;

// Helper function to evaluate balance based on equation
pub fn evaluate_balance(equation: &str, time: i64) -> Result<u64> {
    if equation == "0" {
        return Ok(0);
    }

    let equation = equation.replace("x", &time.to_string());

    // Add support for exponential function
    let equation = if equation.contains("exp(") {
        equation.replace("exp(", "e^(")
    } else {
        equation
    };

    eval_str(&equation)
        .map(|result| result.round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}
