use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct ChronoExtension {
    pub authority: Pubkey,
    pub program_id: Pubkey,
}

impl ChronoExtension {
    pub const EXTENSION_TYPE: u8 = 1;

    pub fn new(authority: Pubkey, program_id: Pubkey) -> Self {
        Self { authority, program_id }
    }
}