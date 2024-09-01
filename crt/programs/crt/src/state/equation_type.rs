use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum EquationType {
    Subscription,
    Inflationary,
    Deflationary,
    Linear,
    Exponential,
    // Add more types as needed
}