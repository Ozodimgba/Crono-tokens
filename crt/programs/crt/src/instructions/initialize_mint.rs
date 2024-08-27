use anchor_lang::prelude::*;
use crate::state::{Mint, DecayPool};
use crate::error::TokenError;

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
    _bump: u8
) -> Result<()> {
    let mint = &mut ctx.accounts.mint;

    if mint.is_initialized {
        return Err(TokenError::AlreadyInUse.into());
    }

    mint.mint_authority = ctx.accounts.authority.key();
    mint.decimals = decimals;
    mint.supply = supply;
    mint.freeze_authority = Some(freeze_authority.expect("Error with freeze authority value"));

    Ok(())
}