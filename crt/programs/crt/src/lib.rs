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
       freeze_authority: Option<Pubkey>
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
        ctx: Context<InitializeTokenAccount>
    )-> Result<()> {
        let token_account = &mut ctx.accounts.account;
        let clock = Clock::get()?;

        token_account.mint = ctx.accounts.mint.key();
        token_account.owner = ctx.accounts.authority.key();

        token_account.balance = "x + 1".to_string();
        token_account.delegate = None;
        token_account.state = AccountState::Initialized;
        token_account.creation_time = clock.unix_timestamp;

        token_account.delegated_amount_equation = "x * 0".to_string();
        token_account.close_authority = Some(ctx.accounts.authority.key());
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let clock = &ctx.accounts.clock;
        let from = &mut ctx.accounts.from;
        let to = &mut ctx.accounts.to;

        // Calculate the current balance for the 'from' account
        let current_time = clock.unix_timestamp;
        let time_since_creation = current_time - from.creation_time;
        let from_balance = evaluate_balance(&from.balance, time_since_creation)?;

        // Ensure sufficient balance
        if from_balance < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        // Update 'from' account balance equation
        from.balance = format!("max(0, ({}) - {})", from.balance, amount);

        // Update 'to' account balance equation
        to.balance = format!("({}) + {}", to.balance, amount);

        // If there's a delegate, update delegated amount
        if let Some(delegate) = from.delegate {
            if delegate == ctx.accounts.authority.key() {
                let delegated_amount = evaluate_balance(&from.delegated_amount_equation, time_since_creation)?;
                if delegated_amount < amount {
                    return Err(TokenError::InsufficientDelegatedAmount.into());
                }
                from.delegated_amount_equation = format!("max(0, ({}) - {})", from.delegated_amount_equation, amount);
            }
        }

        Ok(())
    }

    //add more instructions as needed (burn, freeze, thaw, etc.)
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeTokenAccount<'info> {
    #[account(init, payer = payer, space = TokenAccount::LEN)]
    pub account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
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
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    /// The equation for the amount of tokens this account holds.
    pub balance: String,
    pub creation_time: i64,
    pub state: AccountState,
    pub delegate: Option<Pubkey>,
    pub delegated_amount_equation: String,
    /// Optional authority to close the account.
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
    #[msg("Error evaluating balance equation")]
    BalanceEvaluationError,
}

// Helper function to evaluate balance based on equation
fn evaluate_balance(equation: &str, time: i64) -> Result<u64> {
    let equation = equation.replace("x", &time.to_string());
    eval_str(&equation)
        .map(|result| result.round() as u64)
        .map_err(|_| TokenError::BalanceEvaluationError.into())
}