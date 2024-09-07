use anchor_lang::prelude::*;
use crate::state::{Mint, TokenAccount};
use crate::error::TokenError;
use crate::utils::evaluate_balance;
use crate::events::BurnEvent;


#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Burn>, amount: u64) -> Result<()> {
    let mint = &mut ctx.accounts.mint;
    let token_account = &mut ctx.accounts.token_account;
    let current_time = Clock::get()?.unix_timestamp;

    // Evaluate current balance
    let current_balance = evaluate_balance(
        token_account.last_balance_snapshot,
        &token_account.current_chrono_equation,
        token_account.creation_time,
        current_time)?;

    // Check if there are sufficient tokens to burn
    if current_balance < amount {
        return Err(TokenError::InsufficientFunds.into());
    }

    // Update mint supply
    mint.supply = mint.supply.checked_sub(amount).ok_or(TokenError::Overflow)?;

    // Update token account balance formula
    let new_balance = current_balance.checked_sub(amount).ok_or(TokenError::Overflow)?;

    // Create a new balance formula that subtracts the burned amount
    token_account.last_balance_snapshot = new_balance;

    // // Update last update time
    // token_account.last_update_time = current_time;

    // Emit an event for the burn
    emit!(BurnEvent {
            mint: mint.key(),
            token_account: token_account.key(),
            amount,
            new_balance,
        });

    Ok(())
}