use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{error::ErrorCode, Agreement, Investor, AGREEMENT_SEED, PAYMENT_SEED};

#[derive(Accounts)]
#[instruction(params: WithdrawCancelledFundsBatchParams)]
pub struct WithdrawCancelledFundsBatch<'info> {
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
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawCancelledFundsBatchParams;

impl<'info> WithdrawCancelledFundsBatch<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: WithdrawCancelledFundsBatchParams,
    ) -> Result<()> {
        if !ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementNotCancelled.into());
        }

        let mut success_count = 0;
        for accounts in ctx.remaining_accounts.chunks(2) {
            let investor_account = &accounts[0];
            let investor_token_account = &accounts[1];

            // decode investor account
            let mut investor = Investor::try_from_slice(&investor_account.data.borrow())?;
            if investor.agreement != ctx.accounts.agreement.key() {
                return Err(ErrorCode::InvalidInvestorAccount.into());
            }

            if !investor.investor_deposited {
                continue;
            }

            let invested_amount = investor.required_amount;
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.payment_token_account.to_account_info(),
                        to: investor_token_account.to_account_info(),
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

            investor.investor_deposited = false;
            success_count += 1;
        }

        if success_count == 0 {
            return Err(ErrorCode::NoFundsToWithdraw.into());
        }

        Ok(())
    }
}
