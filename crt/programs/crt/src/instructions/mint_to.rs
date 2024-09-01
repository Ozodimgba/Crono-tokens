use anchor_lang::prelude::*;
use crate::state::{Mint, TokenAccount, EquationType};
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
    let time_elapsed = current_time - token_account.creation_time;

    // Check mint authority
    if mint.mint_authority != ctx.accounts.authority.key() {
        return Err(TokenError::InvalidMintAuthority.into());
    }

    // Update mint supply
    mint.supply = mint.supply.checked_add(amount).ok_or(TokenError::Overflow)?;

    // Update token account balance formula
    let current_balance = evaluate_balance(&token_account.balance, time_elapsed)?;
    let new_balance = current_balance.checked_add(amount).ok_or(TokenError::Overflow)?;

    // Create a new balance formula that adds the minted amount based on the equation type
    token_account.balance = update_balance_equation(
        &token_account.balance,
        &token_account.equation_type,
        amount,
        true,
        time_elapsed,
    )?;

    // Emit an event for the mint
    emit!(MintToEvent {
        mint: mint.key(),
        token_account: token_account.key(),
        amount,
        new_balance,
    });

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