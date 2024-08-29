use anchor_lang::prelude::*;
use crate::state::{Mint, TokenAccount};
use crate::error::TokenError;
use crate::events::MintToEvent;
use crate::utils::evaluate_balance;

#[derive(Accounts)]
pub struct MintTo<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<MintTo>, amount: u64) -> Result<()> {
    let mint = &mut ctx.accounts.mint;
    let token_account = &mut ctx.accounts.token_account;
    let current_time = Clock::get()?.unix_timestamp;

    // Check mint authority
    if mint.mint_authority != ctx.accounts.authority.key() {
        return Err(TokenError::InvalidMintAuthority.into());
    }

    // Update mint supply
    mint.supply = mint.supply.checked_add(amount).ok_or(TokenError::Overflow)?;

    // Update token account balance formula
    let current_balance = evaluate_balance(&token_account.balance, current_time)?;
    let new_balance = current_balance.checked_add(amount).ok_or(TokenError::Overflow)?;

    // Create a new balance formula that adds the minted amount
    token_account.balance = format!(
        "({}) + {}",
        token_account.balance,
        amount
    );


    // Emit an event for the mint
    emit!(MintToEvent {
            mint: mint.key(),
            token_account: token_account.key(),
            amount,
            new_balance,
        });

    Ok(())
}