use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Settings {
    pub cancel_timeout: i64,          // seconds
    pub commission_basis_points: u16, // 1% (100/10000)
    pub owner: Pubkey,                // commission receiver
}
