use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use crate::state::{Mint, TokenAccount, AccountState, DecayPool, PauseType};
use crate::error::TokenError;
use crate::events::ReUpEvent;
use crate::extensions::ChronoExtension;
use crate::utils::evaluate_balance;

#[derive(Accounts)]
pub struct ReUp<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub decay_pool: Account<'info, DecayPool>,
    pub authority: Signer<'info>,
    /// CHECK: This is the ReUp hook program, verified in the instruction
    /// CHECK: This is the chrono hook program, only used if chrono hook is enabled
    #[account(mut)]
    pub chrono_hook_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ReUp>) -> Result<()> {
    let mint = &ctx.accounts.mint;
    let token_account = &mut ctx.accounts.token_account;
    let decay_pool = &mut ctx.accounts.decay_pool;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    //////////////////////////////////////////////////////
    let binding = mint.to_account_info();
    let mint_data = binding.try_borrow_data()?;
    if mint_data.len() > Mint::LEN {
        let extension_data = &mint_data[Mint::LEN..];
        if let Ok(extension) = ChronoExtension::try_from_slice(extension_data) {
            // Verify the provided ReUp hook program matches the one in the extension
            if extension.program_id != ctx.accounts.chrono_hook_program.key() {
                return Err(ProgramError::InvalidAccountData.into());
            }

            // Check if pause is allowed
            if extension.pause_type != PauseType::ReUp {
                return Err(TokenError::ReUpNotAllowed.into());
            }

            // Ensure the token account is not paused
            if token_account.state == AccountState::Pause {
                return Err(TokenError::AccountFrozen.into());
            }

            // Ensure the authority is correct
            if token_account.owner != ctx.accounts.authority.key() {
                return Err(TokenError::OwnerMismatch.into());
            }


            // Call the chrono program using invoke
            let accounts = vec![
                AccountMeta::new(mint.key(), false),
                AccountMeta::new(token_account.key(), false),
                AccountMeta::new_readonly(ctx.accounts.authority.key(), true),
            ];

            let instruction = Instruction {
                program_id: extension.program_id,
                accounts,
                data: AnchorSerialize::try_to_vec(&0u64)?, // Passing 0 as dummy data
            };

            invoke(&instruction, &[
                mint.to_account_info(),
                token_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
            ])?;

            //Re up logic here

            //reup percentage gotten from mint account
            let reup_percentage = extension.reup_percentage;
            // Ensure reup_percentage is valid (0-100)
            if reup_percentage > 100 {
                return Err(TokenError::InvalidReUpPercentage.into());
            }

            // Calculate the amount to ReUp from the decay pool
            let reup_amount = (decay_pool.amount as u128 * reup_percentage as u128 / 100) as u64;

            // Get the current balance
            let current_balance = evaluate_balance(
                token_account.last_balance_snapshot,
                &token_account.current_chrono_equation,
                token_account.creation_time,
                current_time
            )?;

            // Apply the ReUp boost
            let new_balance = current_balance.checked_add(reup_amount)
                .ok_or(TokenError::Overflow)?;

            // Update the token account
            token_account.last_balance_snapshot = new_balance;

            //decay pool token account should match signers mint token account
            //SAFE by virtue of token owner check on ln 52
            if token_account.key() != decay_pool.token_account {
                return Err(TokenError::InvalidAuthority.into())
            }

            // Update the decay pool
            decay_pool.amount = decay_pool.amount.checked_sub(reup_amount)
                .ok_or(TokenError::InsufficientFunds)?;

            // Emit a ReUp event
            emit!(ReUpEvent {
                    mint: mint.key(),
                    token_account: token_account.key(),
                    authority: ctx.accounts.authority.key(),
                    amount: reup_amount,
                    new_balance,
                    decay_pool_balance: decay_pool.amount,
               });
        }
    }

    Ok(())
}

