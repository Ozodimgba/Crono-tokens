use anchor_lang::prelude::*;
use meval::eval_str;
use crate::error::TokenError;

// Helper function to evaluate balance based on equation
pub fn evaluate_balance(equation: &str, time: i64) -> Result<u64> {
    let equation = equation.replace("x", &time.to_string());
    eval_str(&equation)
        .map(|result| result.round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}
