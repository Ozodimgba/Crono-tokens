use anchor_lang::prelude::*;
use crate::state::{Mint, TokenAccount, DecayPool, EquationType};
use crate::error::TokenError;
use crate::utils::evaluate_balance;

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"decay_pool", from.key().as_ref()],
        bump = from_decay_pool.bump
    )]
    pub from_decay_pool: Account<'info, DecayPool>,
    #[account(
        mut,
        seeds = [b"decay_pool", to.key().as_ref()],
        bump = to_decay_pool.bump
    )]
    pub to_decay_pool: Account<'info, DecayPool>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Evaluate current balances
    let from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time - ctx.accounts.from.creation_time)?;
    let to_balance = evaluate_balance(&ctx.accounts.to.balance, current_time - ctx.accounts.to.creation_time)?;

    // Check if sender has sufficient funds
    if from_balance < amount {
        return Err(TokenError::InsufficientFunds.into());
    }

    // Check authority
    if ctx.accounts.from.owner != ctx.accounts.authority.key() {
        if ctx.accounts.from.delegate != ctx.accounts.authority.key() {
            return Err(TokenError::InvalidAuthority.into());
        }
        let delegated_amount = evaluate_balance(&ctx.accounts.from.delegated_amount, current_time - ctx.accounts.from.creation_time)?;
        if delegated_amount < amount {
            return Err(TokenError::InsufficientDelegatedAmount.into());
        }
        // Update delegated amount equation
        ctx.accounts.from.delegated_amount = update_delegated_amount_equation(
            &ctx.accounts.from.delegated_amount,
            &ctx.accounts.from.equation_type,
            amount,
            current_time - ctx.accounts.from.creation_time,
        )?;
    }

    // Update balance equations
    ctx.accounts.from.balance = update_balance_equation(
        &ctx.accounts.from.balance,
        &ctx.accounts.from.equation_type,
        amount,
        false,
        current_time - ctx.accounts.from.creation_time,
    )?;

    ctx.accounts.to.balance = update_balance_equation(
        &ctx.accounts.to.balance,
        &ctx.accounts.to.equation_type,
        amount,
        true,
        current_time - ctx.accounts.to.creation_time,
    )?;

    // Update decay pool
    let new_from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time - ctx.accounts.from.creation_time)?;
    let decay_amount = from_balance - new_from_balance;
    ctx.accounts.from_decay_pool.supply += decay_amount;

    Ok(())
}

fn update_balance_equation(
    current_equation: &str,
    equation_type: &EquationType,
    amount: u64,
    is_receiving: bool,
    time_elapsed: i64,
) -> Result<String> {
    let current_balance = evaluate_balance(current_equation, time_elapsed)?;
    let new_balance = if is_receiving { current_balance + amount } else { current_balance - amount };

    match equation_type {
        EquationType::Subscription => Ok(format!("max(0, {} - ((x + {}) / 2592000))", new_balance, time_elapsed)),
        EquationType::Inflationary => Ok(format!("{} + ((x + {}) / 86400)", new_balance, time_elapsed)),
        EquationType::Deflationary => Ok(format!("max(0, {} - ((x + {}) / 86400))", new_balance, time_elapsed)),
        EquationType::Linear => Ok(format!("max(0, {} - ((x + {}) / 31536000) * 10)", new_balance, time_elapsed)),
        EquationType::Exponential => Ok(format!("{} * exp(-(x + {}) / 31536000)", new_balance, time_elapsed)),
    }
}

fn update_delegated_amount_equation(
    current_equation: &str,
    equation_type: &EquationType,
    amount: u64,
    time_elapsed: i64,
) -> Result<String> {
    let current_delegated_amount = evaluate_balance(current_equation, time_elapsed)?;
    let new_delegated_amount = current_delegated_amount.saturating_sub(amount);

    match equation_type {
        EquationType::Subscription => Ok(format!("max(0, {} - ((x + {}) / 2592000))", new_delegated_amount, time_elapsed)),
        EquationType::Inflationary => Ok(format!("{} + ((x + {}) / 86400)", new_delegated_amount, time_elapsed)),
        EquationType::Deflationary => Ok(format!("max(0, {} - ((x + {}) / 86400))", new_delegated_amount, time_elapsed)),
        EquationType::Linear => Ok(format!("max(0, {} - ((x + {}) / 31536000) * 10)", new_delegated_amount, time_elapsed)),
        EquationType::Exponential => Ok(format!("{} * exp(-(x + {}) / 31536000)", new_delegated_amount, time_elapsed)),
    }
}