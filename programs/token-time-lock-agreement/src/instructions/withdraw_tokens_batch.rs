use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode, Agreement, Investor, Settings, AGREEMENT_SEED, BASIC_POINTS, PROJECT_SEED,
    SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: WithdrawTokensBatchParams)]
pub struct WithdrawTokensBatch<'info> {
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
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawTokensBatchParams;

impl<'info> WithdrawTokensBatch<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: WithdrawTokensBatchParams,
    ) -> Result<()> {
        if !ctx.accounts.agreement.seller_deposited {
            return Err(ErrorCode::TokensNotDeposited.into());
        }
        if ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementAlreadyCancelled.into());
        }
        if ctx.accounts.agreement.hold_duration_start == 0 {
            return Err(ErrorCode::HoldDurationNotStarted.into());
        }
        if ctx.accounts.clock.unix_timestamp
            < ctx.accounts.agreement.hold_duration_start + ctx.accounts.agreement.hold_duration
        {
            return Err(ErrorCode::HoldDurationPeriodNotExpired.into());
        }
        if ctx.accounts.settings.owner != ctx.accounts.owner_token_account.owner {
            return Err(ErrorCode::InvalidDestination.into());
        }

        let mut success_count = 0;
        let mut total_commission = 0;

        for accounts in ctx.remaining_accounts.chunks(2) {
            let investor_account = &accounts[0];
            let investor_token_account = &accounts[1];

            // decode investor account
            let mut investor = Investor::try_from_slice(&investor_account.data.borrow())?;
            if investor.agreement != ctx.accounts.agreement.key() {
                return Err(ErrorCode::InvalidInvestorAccount.into());
            }

            if !investor.investor_deposited || investor.has_withdrawn_tokens {
                continue;
            }

            let investor_tokens = investor.token_allocation;
            let commission_amount = (investor_tokens
                * ctx.accounts.settings.commission_basis_points as u64
                / BASIC_POINTS) as u64;
            let final_token_amount = investor_tokens - commission_amount;

            total_commission += commission_amount;

            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.project_token_account.to_account_info(),
                        to: investor_token_account.to_account_info(),
                        authority: ctx.accounts.agreement.to_account_info(),
                    },
                    &[&[
                        AGREEMENT_SEED.as_bytes(),
                        ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                        &[ctx.accounts.agreement.bump],
                    ]],
                ),
                final_token_amount,
            )?;

            investor.has_withdrawn_tokens = true;
            success_count += 1;
        }

        if success_count == 0 {
            return Err(ErrorCode::NotAnInvestor.into());
        }

        if total_commission > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.project_token_account.to_account_info(),
                        to: ctx.accounts.owner_token_account.to_account_info(),
                        authority: ctx.accounts.agreement.to_account_info(),
                    },
                    &[&[
                        AGREEMENT_SEED.as_bytes(),
                        ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                        &[ctx.accounts.agreement.bump],
                    ]],
                ),
                total_commission,
            )?;

            ctx.accounts.agreement.token_commision_collected += total_commission;
        }

        Ok(())
    }
}
