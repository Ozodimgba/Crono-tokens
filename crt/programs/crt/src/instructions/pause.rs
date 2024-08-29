use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use crate::error::TokenError;
use crate::state::{Mint, TokenAccount, PauseHook, AccountState};
use crate::events::PauseEvent;
use crate::extensions::ChronoExtension;

#[derive(Accounts)]
pub struct Pause<'info> {
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    /// CHECK: This is the chrono hook program, only used if chrono hook is enabled
    #[account(mut)]
    pub chrono_hook_program: AccountInfo<'info>,
}


pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let mint_info = ctx.accounts.mint.to_account_info();
    let data = mint_info.try_borrow_data()?;

    if data.len() > Mint::LEN {
        let extension_data = &data[Mint::LEN..];
        if let Ok(extension) = ChronoExtension::try_from_slice(extension_data) {
            let chrono_program_id = extension.program_id;

            // Ensure the provided chrono_program matches the one in the extension
            if chrono_program_id != ctx.accounts.chrono_hook_program.key() {
                return Err(ProgramError::InvalidAccountData.into());
            }

            // Call the chrono program using invoke
            let accounts = vec![
                AccountMeta::new_readonly(ctx.accounts.mint.key(), false),
                AccountMeta::new(ctx.accounts.token_account.key(), false),
                AccountMeta::new_readonly(ctx.accounts.authority.key(), true),
            ];

            let instruction = Instruction {
                program_id: chrono_program_id,
                accounts,
                data: AnchorSerialize::try_to_vec(&0u64)?, // Passing 0 as dummy data
            };

            invoke(&instruction, &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.token_account.to_account_info(),
                ctx.accounts.authority.to_account_info(),
            ])?;
        }
    }

    // Implement pause logic here
    //ctx.accounts.token_account.state = AccountState::Pause;

    Ok(())
}

