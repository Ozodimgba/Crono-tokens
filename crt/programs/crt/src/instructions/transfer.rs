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

    let sender = &mut ctx.accounts.from;
    let receiver = &mut ctx.accounts.to;
    let sender_decay_account = &mut ctx.accounts.to_decay_pool;
    let receiver_decay_account = &mut ctx.accounts.from_decay_pool;

    if sender.is_frozen() || receiver.is_frozen() {
        return Err(TokenError::AccountFrozen.into());
    }

    // Prevents self transfers
    let self_transfer = sender.key() == receiver.key();
    if self_transfer {
        return Err(TokenError::SelfTransfer.into());
    }

    if amount == 0 {
        return Ok(());
    }

    // Evaluate current balances
    let from_balance = evaluate_balance(
        sender.last_balance_snapshot,
        &sender.current_chrono_equation,
        sender.creation_time,
        current_time
    )?;

    let to_balance = evaluate_balance(
        receiver.last_balance_snapshot,
        &receiver.current_chrono_equation,
        receiver.creation_time,
        current_time
    )?;

    // Check if sender has sufficient funds
    if from_balance < amount {
        return Err(TokenError::InsufficientFunds.into());
    }

    // Check authority
    // if ctx.accounts.from.owner != ctx.accounts.authority.key() {
    //     if ctx.accounts.from.delegate != ctx.accounts.authority.key() {
    //         return Err(TokenError::InvalidAuthority.into());
    //     }
    //     if ctx.accounts.from.delegated_amount < amount {
    //         return Err(TokenError::InsufficientDelegatedAmount.into());
    //     }
    // }

    check_authority(sender, &ctx.accounts.authority.key(), amount)?;

    // New paradigm how about we store the equation as stand-alone field on the token account and update at intialization and pausing
    // This new paradigm should separate the decay state and keep the parameters last_balance and current_time independent of the equation

    // transfer, minus from_balance eval, add to_balance eval...update snapshot
    sender.last_balance_snapshot = from_balance.checked_sub(amount)
        .ok_or(TokenError::Overflow)?;

    receiver.last_balance_snapshot = to_balance.checked_add(amount)
        .ok_or(TokenError::Overflow)?;


    // Update decay pool
    let new_sender_decayed_amount = sender.last_balance_snapshot - from_balance;
    let new_receiver_decayed_amount = receiver.last_balance_snapshot - to_balance;

    sender_decay_account.amount = sender_decay_account.amount.checked_add(new_sender_decayed_amount)
        .ok_or(TokenError::Overflow)?;

    receiver_decay_account.amount = receiver_decay_account.amount.checked_add(new_receiver_decayed_amount)
        .ok_or(TokenError::Overflow)?;

    Ok(())
}

fn check_authority(account: &TokenAccount, authority: &Pubkey, amount: u64) -> Result<()> {
    if account.owner != *authority {
        if account.delegate != *authority {
            return Err(TokenError::InvalidAuthority.into());
        }
        if account.delegated_amount < amount {
            return Err(TokenError::InsufficientDelegatedAmount.into());
        }
    }
    Ok(())
}