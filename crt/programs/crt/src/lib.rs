use anchor_lang::prelude::*;
pub mod utils;
pub mod instructions;
pub mod error;
pub mod state;
pub mod events;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod chrono_token {
    use super::*;

    pub fn initialize_mint(ctx: Context<InitializeMint>, decimals: u8, supply: u64, freeze_authority: Option<Pubkey>, _bump: u8) -> Result<()> {
        instructions::initialize_mint::handler(ctx, decimals, supply, freeze_authority, _bump)
    }

    pub fn initialize_token_account(ctx: Context<InitializeTokenAccount>, delegate: Option<Pubkey>) -> Result<()> {
        instructions::initialize_token_account::handler(ctx, delegate)
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        instructions::transfer::handler(ctx, amount)
    }

    pub fn mint_to(ctx: Context<MintTo>, amount: u64) -> Result<()> {
        instructions::mint_to::handler(ctx, amount)
    }

    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        instructions::burn::handler(ctx, amount)
    }
}

