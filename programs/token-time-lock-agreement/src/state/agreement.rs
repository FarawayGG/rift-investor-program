use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Agreement {
    pub agreement_id: u64,
    pub payment_token_mint: Pubkey,
    pub project_token_mint: Pubkey,
    pub company_wallet: Pubkey,
    pub token_seller: Pubkey,
    pub expected_payment: u64,
    pub expected_tokens: u64,
    pub total_invested: u64,
    pub total_required: u64,
    pub total_token_allocation: u64,
    pub hold_duration: i64,
    pub hold_duration_start: i64,
    pub seller_deposited: bool,
    pub first_deposit_time: i64,
    pub funds_commision_collected: bool,
    pub token_commision_collected: u64,
    pub agreement_cancelled: bool,
    pub investors_count: u64,
    pub owner: Pubkey,
    pub bump: u8,
}
