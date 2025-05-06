use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode, Agreement, Settings, AGREEMENT_SEED, BASIC_POINTS, PAYMENT_SEED,
    SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: WithdrawFundsParams)]
pub struct WithdrawFunds<'info> {
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
        seeds = [PAYMENT_SEED.as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub payment_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawFundsParams;

impl<'info> WithdrawFunds<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: WithdrawFundsParams,
    ) -> Result<()> {
        if ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementAlreadyCancelled.into());
        }
        if ctx.accounts.agreement.total_invested != ctx.accounts.agreement.expected_payment {
            return Err(ErrorCode::FullAmountNotCollected.into());
        }
        if ctx.accounts.agreement.funds_commision_collected {
            return Err(ErrorCode::FundCommissionAlreadyCollected.into());
        }
        if ctx.accounts.agreement.token_seller != ctx.accounts.seller_token_account.owner.key() {
            return Err(ErrorCode::InvalidDestination.into());
        }
        if ctx.accounts.settings.owner != ctx.accounts.owner_token_account.owner.key() {
            return Err(ErrorCode::InvalidDestination.into());
        }

        let commission_amount = (ctx.accounts.agreement.expected_payment
            * ctx.accounts.settings.commission_basis_points as u64)
            / BASIC_POINTS;
        let seller_amount = ctx.accounts.agreement.expected_payment - commission_amount;

        if commission_amount > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.payment_token_account.to_account_info(),
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

            ctx.accounts.agreement.funds_commision_collected = true;
        }

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payment_token_account.to_account_info(),
                    to: ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.agreement.to_account_info(),
                },
                &[&[
                    AGREEMENT_SEED.as_bytes(),
                    ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                    &[ctx.accounts.agreement.bump],
                ]],
            ),
            seller_amount,
        )?;

        Ok(())
    }
}
