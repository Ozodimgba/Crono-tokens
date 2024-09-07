use anchor_lang::prelude::*;
use super::{AccountState, ChronoEquationType};

#[account]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    /// The last recorded balance snapshot
    pub last_balance_snapshot: u64,
    /// The current equation for calculating the balance
    pub current_chrono_equation: ChronoEquationType,
    pub creation_time: i64,
    pub state: AccountState,
    /// CHECK: This is safe because it's set by the program and verified in relevant instruction
    pub delegate: Pubkey,
    /// CHECK: This is safe because it's set by the program and verified in relevant instruction
    pub delegated_amount: u64,
    /// CHECK: This is safe because SafeOptionPubkey is a custom type that safely represents an optional Pubkey.
    /// The close authority is only used when closing the account, which is checked separately.
    pub close_authority: Option<Pubkey>
}

impl TokenAccount {
    pub const LEN: usize = 8 + // Anchor account discriminator
        32 + // mint
        32 + // owner
        8 + // last_balance_snapshot
        4 + 200 + // current_chrono_equation (4 bytes for length + max 200 bytes for content)
        8 + // creation_time
        1 + // state
        32 + // delegate
        4 + 200 + // delegated_amount (4 bytes for length + max 200 bytes for content)
        1 + 32 + // close_authority
        1; // equation_type


    /// Checks if account is frozen
    pub fn is_frozen(&self) -> bool {
        self.state == AccountState::Frozen
    }
}