mod utils;

use anchor_lang::prelude::*;
use meval::eval_str;
use meval::tokenizer::Token;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod chrono_token {
    use super::*;

   pub fn initialize_mint(
       ctx: Context<InitializeMint>,
       decimals: u8,
       supply: u64,
       freeze_authority: Option<Pubkey>,
       _bump: u8
   ) -> Result<()> {
       let mint = &mut ctx.accounts.mint;

       if mint.is_initialized {
           return Err(TokenError::AlreadyInUse.into());
       }

       mint.mint_authority = ctx.accounts.authority.key();
       mint.decimals = decimals;
       mint.supply = supply;
       mint.freeze_authority = Some(freeze_authority.expect("Error with freeze authority value"));

       
       Ok(())
    }

    pub fn initialize_token_account(
        ctx: Context<InitializeTokenAccount>,
        delegate: Option<Pubkey>
    ) -> Result<()> {
        let token_account = &mut ctx.accounts.token_account;
        let clock = Clock::get()?;

        token_account.mint = ctx.accounts.mint.key();
        token_account.owner = ctx.accounts.authority.key();

        token_account.balance = "x + 1".to_string();
        token_account.delegate = delegate.unwrap_or(Pubkey::default());
        token_account.state = AccountState::Initialized;
        token_account.creation_time = clock.unix_timestamp;

        token_account.delegated_amount = "x * 0".to_string();
        token_account.close_authority = Some(ctx.accounts.authority.key());
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Evaluate current balances
        let from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time)?;
        let to_balance = evaluate_balance(&ctx.accounts.to.balance, current_time)?;

        // Check if sender has sufficient funds
        if from_balance < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        // Check authority
        if ctx.accounts.from.owner != ctx.accounts.authority.key() {
            if ctx.accounts.from.delegate != ctx.accounts.authority.key() {
                return Err(TokenError::InvalidAuthority.into());
            }
            let delegated_amount = evaluate_balance(&ctx.accounts.from.delegated_amount, current_time)?;
            if delegated_amount < amount {
                return Err(TokenError::InsufficientDelegatedAmount.into());
            }
            // Update delegated amount equation
            ctx.accounts.from.delegated_amount = format!("max(0, min(({0}), ({1}) - {2}))",ctx.accounts.from.delegated_amount, ctx.accounts.from.balance, amount);
        }

        // Update balance equations
        ctx.accounts.from.balance = format!("max(0, ({}) - {})", ctx.accounts.from.balance, amount);
        ctx.accounts.to.balance = format!("({}) + {}", ctx.accounts.to.balance, amount);

        // Update decay pool
        let new_from_balance = evaluate_balance(&ctx.accounts.from.balance, current_time)?;
        let decay_amount = from_balance - new_from_balance;
        ctx.accounts.from_decay_pool.supply += decay_amount;

        Ok(())
    }

    pub fn mint_to(ctx: Context<MintTo>, amount: u64) -> Result<()> {
        let mint = &mut ctx.accounts.mint;
        let token_account = &mut ctx.accounts.token_account;
        let current_time = Clock::get()?.unix_timestamp;

        // Check mint authority
        if mint.mint_authority != ctx.accounts.authority.key() {
            return Err(TokenError::InvalidMintAuthority.into());
        }

        // Update mint supply
        mint.supply = mint.supply.checked_add(amount).ok_or(TokenError::Overflow)?;

        // Update token account balance formula
        let current_balance = evaluate_balance(&token_account.balance, current_time)?;
        let new_balance = current_balance.checked_add(amount).ok_or(TokenError::Overflow)?;

        // Create a new balance formula that adds the minted amount
        token_account.balance = format!(
            "({}) + {}",
            token_account.balance,
            amount
        );


        // Emit an event for the mint
        emit!(MintToEvent {
            mint: mint.key(),
            token_account: token_account.key(),
            amount,
            new_balance,
        });

        Ok(())
    }

    pub fn burn(ctx: Context<Burn>, amount: u64) -> Result<()> {
        let mint = &mut ctx.accounts.mint;
        let token_account = &mut ctx.accounts.token_account;
        let current_time = Clock::get()?.unix_timestamp;

        // Evaluate current balance
        let current_balance = evaluate_balance(&token_account.balance, current_time)?;

        // Check if there are sufficient tokens to burn
        if current_balance < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        // Update mint supply
        mint.supply = mint.supply.checked_sub(amount).ok_or(TokenError::Overflow)?;

        // Update token account balance formula
        let new_balance = current_balance.checked_sub(amount).ok_or(TokenError::Overflow)?;

        // Create a new balance formula that subtracts the burned amount
        token_account.balance = format!(
            "max(0, ({}) - {})",
            token_account.balance,
            amount
        );

        // // Update last update time
        // token_account.last_update_time = current_time;

        // Emit an event for the burn
        emit!(BurnEvent {
            mint: mint.key(),
            token_account: token_account.key(),
            amount,
            new_balance,
        });

        Ok(())
    }

    //add more instructions as needed (burn, freeze, thaw, etc.)
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = DecayPool::LEN,
        seeds = [b"decay_pool", mint.key().as_ref()],
        bump
    )]
    pub decay_pool: Account<'info, DecayPool>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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

#[derive(Accounts)]
pub struct Burn<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintTo<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"decay_pool", from.key().as_ref()],
        bump = from_decay_pool.bump
    )]
    pub from_decay_pool: Account<'info, DecayPool>,
    #[account(
        mut,
        seeds = [b"decay_pool", to.key().as_ref()],
        bump = to_decay_pool.bump
    )]
    pub to_decay_pool: Account<'info, DecayPool>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Mint {
    pub mint_authority: Pubkey,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: Option<Pubkey>,
    pub supply: u64,
}

impl Mint {
    pub const LEN: usize = 8 + 32 + 1 + 1 + 32 + 200; // Approximate size, adjust as needed
}

#[account]
pub struct DecayPool {
    pub token_account: Pubkey,
    pub supply: u64,
    pub bump: u8,
}

impl DecayPool {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum AccountState {
    Uninitialized,
    Initialized,
    Pause,
    Unpause,
    Frozen,
}

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
}

// Helper function to evaluate balance based on equation
fn evaluate_balance(equation: &str, time: i64) -> Result<u64> {
    let equation = equation.replace("x", &time.to_string());
    eval_str(&equation)
        .map(|result| result.round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}