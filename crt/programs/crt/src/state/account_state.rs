use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum AccountState {
    Uninitialized,
    Initialized,
    Pause,
    Unpause,
    Frozen,
}