use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Investor {
    pub has_withdrawn_tokens: bool,
    pub investor_deposited: bool,
    pub token_allocation: u64,
    pub required_amount: u64,
    pub wallet: Pubkey,
    pub agreement: Pubkey,
    pub bump: u8,
}
