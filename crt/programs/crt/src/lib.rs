use anchor_lang::prelude::*;

declare_id!("CYonGNksY6zhLRxKu9Wk5L6p3VqznLfNdKZkAiFvpzt9");

#[program]
pub mod crt {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
