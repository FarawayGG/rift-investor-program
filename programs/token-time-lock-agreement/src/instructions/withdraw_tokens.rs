use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode, Agreement, Investor, Settings, AGREEMENT_SEED, BASIC_POINTS, INVESTOR_SEED,
    PROJECT_SEED, SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: WithdrawTokensParams)]
pub struct WithdrawTokens<'info> {
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
        seeds = [INVESTOR_SEED.as_bytes(), agreement.key().as_ref(), investor.wallet.as_ref()],
        bump,
    )]
    pub investor: Account<'info, Investor>,
    #[account(mut,
        seeds = [PROJECT_SEED.as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub project_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub investor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawTokensParams;

impl<'info> WithdrawTokens<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: WithdrawTokensParams,
    ) -> Result<()> {
        if !ctx.accounts.agreement.seller_deposited {
            return Err(ErrorCode::TokensNotDeposited.into());
        }
        if !ctx.accounts.investor.investor_deposited {
            return Err(ErrorCode::NotAnInvestor.into());
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
        if ctx.accounts.investor.has_withdrawn_tokens {
            return Err(ErrorCode::TokensAlreadyWithdrawn.into());
        }
        if ctx.accounts.settings.owner != ctx.accounts.owner_token_account.owner {
            return Err(ErrorCode::InvalidDestination.into());
        }

        // Calculate commission (1% of tokens)
        let investor_tokens = ctx.accounts.investor.token_allocation;
        let commission_amount =
            (investor_tokens * ctx.accounts.settings.commission_basis_points as u64) / BASIC_POINTS;
        let final_token_amount = investor_tokens - commission_amount;

        // Transfer commission to owner if needed
        if commission_amount > 0 {
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
                commission_amount,
            )?;

            ctx.accounts.agreement.token_commision_collected += commission_amount;
        }

        // Transfer tokens to investor
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.project_token_account.to_account_info(),
                    to: ctx.accounts.investor_token_account.to_account_info(),
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

        ctx.accounts.investor.has_withdrawn_tokens = true;

        Ok(())
    }
}
