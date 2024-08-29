use anchor_lang::prelude::*;

#[error_code]
pub enum TokenError {
    #[msg("Mint authority is invalid")]
    MintAuthorityInvalid,
    #[msg("Mint mismatch")]
    MintMismatch,
    #[msg("Owner mismatch")]
    OwnerMismatch,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Numerical overflow")]
    Overflow,
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Account is is already intialized")]
    AlreadyInUse,
    #[msg("Insufficient delegated amount for transfer")]
    InsufficientDelegatedAmount,
    #[msg("Invalid authority for operation")]
    InvalidAuthority,
    #[msg("Error evaluating balance equation")]
    BalanceEvaluationError,
    #[msg("Invalid mint authority for operation")]
    InvalidMintAuthority,
    #[msg("Account is already paused")]
    AlreadyPaused,
}