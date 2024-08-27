use anchor_lang::prelude::*;
use crate::state::{Mint, TokenAccount, DecayPool};
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
    let from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time)?;
    let to_balance = evaluate_balance(&ctx.accounts.to.balance, current_time)?;

    // Check if sender has sufficient funds
    if from_balance < amount {
        return Err(TokenError::InsufficientFunds.into());
    }

    // Check authority
    if ctx.accounts.from.owner != ctx.accounts.authority.key() {
        if ctx.accounts.from.delegate != ctx.accounts.authority.key() {
            return Err(TokenError::InvalidAuthority.into());
        }
        let delegated_amount = evaluate_balance(&ctx.accounts.from.delegated_amount, current_time)?;
        if delegated_amount < amount {
            return Err(TokenError::InsufficientDelegatedAmount.into());
        }
        // Update delegated amount equation
        ctx.accounts.from.delegated_amount = format!("max(0, min(({0}), ({1}) - {2}))",ctx.accounts.from.delegated_amount, ctx.accounts.from.balance, amount);
    }

    // Update balance equations
    ctx.accounts.from.balance = format!("max(0, ({}) - {})", ctx.accounts.from.balance, amount);
    ctx.accounts.to.balance = format!("({}) + {}", ctx.accounts.to.balance, amount);

    // Update decay pool
    let new_from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time)?;
    let decay_amount = from_balance - new_from_balance;
    ctx.accounts.from_decay_pool.supply += decay_amount;

    Ok(())
}