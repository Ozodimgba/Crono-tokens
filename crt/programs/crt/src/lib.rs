use anchor_lang::prelude::*;

pub mod utils;
pub mod instructions;
pub mod error;
pub mod state;
pub mod events;
mod extensions;
mod tokenizer;

use instructions::*;
use crate::state::{ChronoEquationType, PauseType, EquationParams};

declare_id!("crnXvAtgkLMzJKEFdveTZ4Redy3mHa1YY9UXP9wZ91c");

#[program]
pub mod chrono_token {
    use super::*;

    pub fn initialize_mint(ctx: Context<InitializeMint>,
                           decimals: u8,
                           supply: u64,
                           freeze_authority: Option<Pubkey>,
                           _bump: u8,
                           enable_chrono_hook: bool,
                           chrono_hook_program_id: Option<Pubkey>,
                           equation_type: Option<ChronoEquationType>,
                           pause_type: Option<PauseType>,
                           equation_params: Option<EquationParams>,
                           reup_percentage: Option<u8>,
    ) -> Result<()> {
        instructions::initialize_mint::handler(ctx,
                                               decimals,
                                               supply,
                                               freeze_authority,
                                               _bump,
                                               enable_chrono_hook,
                                               chrono_hook_program_id,
                                               equation_type,
                                               pause_type,
                                               equation_params,
                                               reup_percentage
        )
    }

    pub fn initialize_token_account(ctx: Context<InitializeTokenAccount>,
                                    delegate: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_token_account::handler(ctx,
                                                        delegate,
        )
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        instructions::transfer::handler(ctx, amount)
    }

    pub fn mint_to(ctx: Context<MintTo>, amount: u64) -> Result<()> {
        instructions::mint_to::handler(ctx, amount)
    }

    pub fn reup(ctx: Context<ReUp>) -> Result<()> {
        instructions::reup::handler(ctx)
    }

    pub fn pause_decay(ctx: Context<PauseDecay>) -> Result<()> {
        instructions::pause_decay::handler(ctx)
    }

    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        instructions::burn::handler(ctx, amount)
    }
}