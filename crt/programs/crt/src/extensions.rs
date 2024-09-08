use anchor_lang::prelude::*;
use crate::state::{ChronoEquationType, EquationParams, PauseType};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ChronoExtension {
    pub authority: Pubkey,
    pub program_id: Pubkey,
    pub equation_type: ChronoEquationType,
    pub pause_type: PauseType,
    pub equation_params: EquationParams,
    pub reup_percentage: u8,
}

impl ChronoExtension {
    pub const EXTENSION_TYPE: u8 = 1;

    pub fn new(
        authority: Pubkey,
        program_id: Pubkey,
        equation_type: ChronoEquationType,
        pause_type: PauseType,
        equation_params: EquationParams,
        reup_percentage: u8,
    ) -> Self {
        Self {
            authority,
            program_id,
            equation_type,
            pause_type,
            equation_params,
            reup_percentage,
        }
    }
}