use anchor_lang::prelude::*;
use super::AccountState;

#[account]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    /// The equation for the amount of tokens this account holds.
    pub balance: String,
    pub creation_time: i64,
    pub state: AccountState,
    /// CHECK: This is safe because it's set by the program and verified in relevant instruction
    pub delegate: Pubkey,
    /// CHECK: This is safe because it's set by the program and verified in relevant instruction
    pub delegated_amount: String,
    /// CHECK: This is safe because SafeOptionPubkey is a custom type that safely represents an optional Pubkey.
    /// The close authority is only used when closing the account, which is checked separately.
    #[doc(hidden)]
    pub close_authority: Option<Pubkey>,
}

impl TokenAccount {
    pub const LEN: usize = 8 + 32 + 32 + 200 + 8 + 1 + 32 + 200 + 32; // Approximate size, adjust as needed

    /// Checks if account is frozen
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen
    }
}