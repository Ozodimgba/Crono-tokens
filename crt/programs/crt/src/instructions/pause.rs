use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use crate::error::TokenError;
use crate::state::{Mint, TokenAccount, AccountState, PauseType};
use crate::events::PauseEvent;
use crate::extensions::ChronoExtension;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    /// CHECK: This is the chrono hook program, only used if chrono hook is enabled
    #[account(mut)]
    pub chrono_hook_program: AccountInfo<'info>,
}

pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let mint = &mut ctx.accounts.mint;
    let token_account = &mut ctx.accounts.token_account;
    let binding = mint.to_account_info();
    let mint_data = binding.try_borrow_data()?;

    if mint_data.len() > Mint::LEN {
        let extension_data = &mint_data[Mint::LEN..];
        if let Ok(extension) = ChronoExtension::try_from_slice(extension_data) {
            if extension.program_id != ctx.accounts.chrono_hook_program.key() {
                return Err(ProgramError::InvalidAccountData.into());
            }

            // Check if pause is allowed
            if extension.pause_type != PauseType::Pause {
                return Err(TokenError::PauseNotAllowed.into());
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

            // Update token account state
            token_account.state = AccountState::Pause;

            // Update the current chrono equation to a linear equation with slope 0
            token_account.current_chrono_equation = mint.chrono_equation;

            // Emit pause event
            emit!(PauseEvent {
                mint: mint.key(),
                token_account: token_account.key(),
                authority: ctx.accounts.authority.key(),
            });
        }
    }

    Ok(())
}
