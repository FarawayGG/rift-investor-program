use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::{
    error::ErrorCode, Agreement, Investor, Settings, AGREEMENT_SEED, INVESTOR_SEED, PROJECT_SEED,
    SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: CancelAgreementParams)]
pub struct CancelAgreement<'info> {
    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump
    )]
    pub settings: Account<'info, Settings>,
    #[account(mut,
        seeds = [AGREEMENT_SEED.as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump
    )]
    pub agreement: Account<'info, Agreement>,
    #[account(mut,
        seeds = [PROJECT_SEED.as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub project_token_account: Account<'info, TokenAccount>,
    #[account(mut,
        seeds = [INVESTOR_SEED.as_bytes(), agreement.key().as_ref(), investor.wallet.as_ref()],
        bump,
    )]
    pub investor: Option<Account<'info, Investor>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct CancelAgreementParams;

impl<'info> CancelAgreement<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: CancelAgreementParams,
    ) -> Result<()> {
        if let Some(investor) = &mut ctx.accounts.investor {
            if investor.wallet != *ctx.accounts.payer.key {
                return Err(ErrorCode::NotAuthorized.into());
            }
        } else {
            if ctx.accounts.agreement.token_seller != *ctx.accounts.payer.key
                && ctx.accounts.agreement.owner != *ctx.accounts.payer.key
            {
                return Err(ErrorCode::NotAuthorized.into());
            }
        }
        if ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementAlreadyCancelled.into());
        }

        let current_balance = ctx.accounts.project_token_account.amount;

        // Tokens present but not processed - can cancel immediately
        if current_balance > 0 && !ctx.accounts.agreement.seller_deposited {
            ctx.accounts.agreement.agreement_cancelled = true;
            return Ok(());
        }

        // Tokens processed but funds not complete after cancellation timeout
        if ctx.accounts.agreement.seller_deposited {
            if ctx.accounts.agreement.first_deposit_time == 0 {
                return Err(ErrorCode::NoFirstDeposit.into());
            }
            if ctx.accounts.clock.unix_timestamp
                < ctx.accounts.agreement.first_deposit_time + ctx.accounts.settings.cancel_timeout
            {
                return Err(ErrorCode::CancellationTimeoutNotReached.into());
            }
            if ctx.accounts.agreement.total_invested == ctx.accounts.agreement.expected_payment
                && ctx.accounts.agreement.hold_duration_start > 0
            {
                return Err(ErrorCode::FullAmountNotCollected.into());
            }
            ctx.accounts.agreement.agreement_cancelled = true;
            return Ok(());
        }

        return Err(ErrorCode::TokensNotDeposited.into());
    }
}
