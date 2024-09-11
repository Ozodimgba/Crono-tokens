use anchor_lang::prelude::*;
use crate::state::{Mint, DecayPool, EquationParams};
use crate::error::TokenError;
use crate::state::{ChronoEquationType, PauseType};
use crate::extensions::ChronoExtension;

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, space = Mint::LEN)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        space = ChronoExtension::LEN,
        seeds = [b"chrono_extension", mint.key().as_ref()],
        bump
    )]
    pub chrono_extension: Account<'info, ChronoExtension>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeMint>,
    decimals: u8,
    supply: u64,
    freeze_authority: Option<Pubkey>,
    _bump: u8,
    enable_chrono_hook: bool,
    chrono_hook_program_id: Option<Pubkey>,
    equation_type: Option<ChronoEquationType>,
    pause_type: Option<PauseType>,
    equation_params: Option<EquationParams>,
    reup_percentage: Option<u8>,
) -> Result<()> {
    let mint = &mut ctx.accounts.mint;

    if mint.is_initialized {
        return Err(TokenError::AlreadyInUse.into());
    }

    mint.mint_authority = ctx.accounts.authority.key();
    mint.decimals = decimals;
    mint.supply = supply;
    mint.freeze_authority = Some(freeze_authority.expect("Error with freeze authority value"));
    mint.chrono_equation = equation_type.expect("error with equation type");
    mint.pause_type = pause_type.expect("error with pause type");

    if enable_chrono_hook {
        if let (Some(program_id), Some(eq_type), Some(p_type), Some(params)) = (
            chrono_hook_program_id,
            equation_type,
            pause_type,
            equation_params,
        ) {
            let reup_percentage = match (p_type, reup_percentage) {
                (PauseType::ReUp, Some(percentage)) if percentage <= 100 => percentage,
                (PauseType::ReUp, Some(_)) => return Err(TokenError::InvalidReUpPercentage.into()),
                (PauseType::ReUp, None) => return Err(TokenError::MissingReUpPercentage.into()),
                (_, Some(_)) => return Err(TokenError::UnexpectedReUpPercentage.into()),
                (_, None) => 0, // Default value when not ReUp
            };

            let chrono_extension = &mut ctx.accounts.chrono_extension;
            chrono_extension.authority = ctx.accounts.authority.key();
            chrono_extension.program_id = program_id;
            chrono_extension.equation_type = eq_type;
            chrono_extension.pause_type = p_type;
            chrono_extension.equation_params = params;
            chrono_extension.reup_percentage = reup_percentage;
        } else {
            return Err(ProgramError::InvalidArgument.into());
        }
    }


    Ok(())
}