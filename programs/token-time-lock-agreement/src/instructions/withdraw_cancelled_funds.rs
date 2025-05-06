use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode, Agreement, Investor, AGREEMENT_SEED, INVESTOR_SEED, PAYMENT_SEED,
    PROJECT_SEED,
};

#[derive(Accounts)]
#[instruction(params: WithdrawCancelledFundsParams)]
pub struct WithdrawCancelledFunds<'info> {
    #[account(mut,
        seeds = [AGREEMENT_SEED.as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump
    )]
    pub agreement: Account<'info, Agreement>,
    #[account(mut,
        seeds = [INVESTOR_SEED.as_bytes(), agreement.key().as_ref(), investor.wallet.as_ref()],
        bump,
    )]
    pub investor: Option<Account<'info, Investor>>,
    #[account(mut,
        seeds = [investor.as_ref().map(|_| PAYMENT_SEED).unwrap_or(PROJECT_SEED).as_bytes(), agreement.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub agreement_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawCancelledFundsParams;

impl<'info> WithdrawCancelledFunds<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: WithdrawCancelledFundsParams,
    ) -> Result<()> {
        if !ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementNotCancelled.into());
        }

        if ctx.accounts.agreement.token_seller == *ctx.accounts.payer.key {
            let remaining_tokens = ctx.accounts.agreement_token_account.amount;
            if remaining_tokens > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.agreement_token_account.to_account_info(),
                            to: ctx.accounts.destination_token_account.to_account_info(),
                            authority: ctx.accounts.agreement.to_account_info(),
                        },
                        &[&[
                            AGREEMENT_SEED.as_bytes(),
                            ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                            &[ctx.accounts.agreement.bump],
                        ]],
                    ),
                    remaining_tokens,
                )?;

                // Reset sellerDeposited flag if it was set
                ctx.accounts.agreement.seller_deposited = false;
            } else {
                return Err(ErrorCode::NoFundsToWithdraw.into());
            }
        } else if let Some(investor) = &ctx.accounts.investor {
            if !investor.investor_deposited {
                return Err(ErrorCode::NoFundsToWithdraw.into());
            }

            let invested_amount = investor.required_amount;
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.agreement_token_account.to_account_info(),
                        to: ctx.accounts.destination_token_account.to_account_info(),
                        authority: ctx.accounts.agreement.to_account_info(),
                    },
                    &[&[
                        AGREEMENT_SEED.as_bytes(),
                        ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                        &[ctx.accounts.agreement.bump],
                    ]],
                ),
                invested_amount,
            )?;
        } else {
            return Err(ErrorCode::NotAnInvestor.into());
        }

        Ok(())
    }
}
