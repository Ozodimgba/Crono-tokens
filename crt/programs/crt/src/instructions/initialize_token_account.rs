use anchor_lang::prelude::*;
use crate::state::{TokenAccount, Mint, DecayPool, AccountState, EquationType};


#[derive(Accounts)]
pub struct InitializeTokenAccount<'info> {
    #[account(init, payer = payer, space = TokenAccount::LEN)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
        space = DecayPool::LEN,
        seeds = [b"decay_pool", token_account.key().as_ref()],
        bump
    )]
    pub decay_pool: Account<'info, DecayPool>,
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeTokenAccount>,
    delegate: Option<Pubkey>,
    equation_type: EquationType
) -> Result<()> {
    let token_account = &mut ctx.accounts.token_account;
    let clock = Clock::get()?;

    token_account.mint = ctx.accounts.mint.key();
    token_account.owner = ctx.accounts.authority.key();

    token_account.delegate = delegate.unwrap_or(Pubkey::default());
    token_account.state = AccountState::Initialized;
    token_account.creation_time = clock.unix_timestamp;

    token_account.balance = match equation_type {
        EquationType::Subscription => "0".to_string(), // Starts at 0, will be increased by minting
        EquationType::Inflationary => "(x / 86400)".to_string(), // Increases by 1 per day
        EquationType::Deflationary => "0".to_string(), // Starts at 0, will be increased by minting and then decay
        EquationType::Linear => "0".to_string(), // Starts at 0, will be increased by minting and then decay linearly
        EquationType::Exponential => "0".to_string(), // Starts at 0, will be increased by minting and then decay exponentially
    };

    token_account.delegated_amount = "x * 0".to_string();
    token_account.close_authority = Some(ctx.accounts.authority.key());
    Ok(())
}