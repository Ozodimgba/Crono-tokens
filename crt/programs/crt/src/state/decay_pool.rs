use anchor_lang::prelude::*;


#[account]
pub struct DecayPool {
    pub token_account: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl DecayPool {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}