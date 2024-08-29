use anchor_lang::event;
use anchor_lang::prelude::*;

#[event]
pub struct MintToEvent {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct BurnEvent {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct PauseEvent {
    pub mint: Pubkey,
    pub token_account: Pubkey,
}