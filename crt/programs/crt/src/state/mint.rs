use anchor_lang::prelude::*;
use crate::state::{ChronoEquationType, PauseType};

#[account]
pub struct Mint {
    pub mint_authority: Pubkey,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<Pubkey>,
    pub chrono_equation: ChronoEquationType,
    pub pause_type: PauseType,
    pub supply: u64,
    pub pause_hook: Pubkey,
}

impl Mint {
    pub const LEN: usize = 8 + 32 + 1 + 1 + 32 + 200; // Approximate size, adjust as needed
}