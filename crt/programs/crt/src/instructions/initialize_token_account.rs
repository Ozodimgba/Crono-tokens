use anchor_lang::prelude::*;
use crate::state::{TokenAccount, Mint, DecayPool, AccountState, ChronoEquationType};


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
) -> Result<()> {
    let token_account = &mut ctx.accounts.token_account;
    let decay_pool = &mut ctx.accounts.decay_pool;
    let mint = &mut ctx.accounts.mint;
    let clock = Clock::get()?;

    token_account.mint = mint.key();
    token_account.owner = ctx.accounts.authority.key();

    token_account.delegate = delegate.unwrap_or(Pubkey::default());
    token_account.state = AccountState::Initialized;
    token_account.creation_time = clock.unix_timestamp;
    token_account.last_balance_snapshot = 0;

    // should be pda for the token account owned by the chrono program
    decay_pool.token_account = token_account.key();
    decay_pool.amount = 0;


    token_account.current_chrono_equation = mint.chrono_equation;

    token_account.delegated_amount = 0;
    token_account.close_authority = Some(ctx.accounts.authority.key());
    Ok(())
}