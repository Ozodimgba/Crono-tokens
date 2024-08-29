use anchor_lang::prelude::*;
use crate::state::{Mint, DecayPool};
use crate::error::TokenError;
use crate::extensions::ChronoExtension;

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = DecayPool::LEN,
        seeds = [b"decay_pool", mint.key().as_ref()],
        bump
    )]
    pub decay_pool: Account<'info, DecayPool>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeMint>,
    decimals: u8,
    supply: u64,
    freeze_authority: Option<Pubkey>,
    _bump: u8,
    enable_chrono_hook: bool,
    chrono_hook_program_id: Option<Pubkey>
) -> Result<()> {
    let mint = &mut ctx.accounts.mint;

    if mint.is_initialized {
        return Err(TokenError::AlreadyInUse.into());
    }

    mint.mint_authority = ctx.accounts.authority.key();
    mint.decimals = decimals;
    mint.supply = supply;
    mint.freeze_authority = Some(freeze_authority.expect("Error with freeze authority value"));

    if enable_chrono_hook {
        if let Some(program_id) = chrono_hook_program_id {
            let extension = ChronoExtension::new(ctx.accounts.authority.key(), program_id);
            let extension_data = extension.try_to_vec()?;

            // Append extension data to the end of the mint account
            let mint_info = mint.to_account_info();
            let mut data = mint_info.try_borrow_mut_data()?;
            let start_index = Mint::LEN;
            data[start_index..start_index + extension_data.len()].copy_from_slice(&extension_data);
        } else {
            return Err(ProgramError::InvalidArgument.into());
        }
    }

    Ok(())
}