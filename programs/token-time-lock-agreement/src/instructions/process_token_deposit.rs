use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{error::ErrorCode, Agreement, AGREEMENT_SEED, PROJECT_SEED};

#[derive(Accounts)]
#[instruction(params: ProcessTokenDepositParams)]
pub struct ProcessTokenDeposit<'info> {
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
    pub company_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct ProcessTokenDepositParams;

impl<'info> ProcessTokenDeposit<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        _: ProcessTokenDepositParams,
    ) -> Result<()> {
        if ctx.accounts.agreement.agreement_cancelled {
            return Err(ErrorCode::AgreementAlreadyCancelled.into());
        }
        if ctx.accounts.agreement.seller_deposited {
            return Err(ErrorCode::TokensAlreadyDeposited.into());
        }
        if ctx.accounts.company_token_account.mint != ctx.accounts.agreement.project_token_mint
            || ctx.accounts.company_token_account.owner != ctx.accounts.agreement.company_wallet
        {
            return Err(ErrorCode::InvalidDestination.into());
        }

        let current_balance = ctx.accounts.project_token_account.amount;
        if current_balance < ctx.accounts.agreement.expected_tokens {
            return Err(ErrorCode::InsufficientTokenBalance.into());
        }

        // transfer excess tokens to the company token account
        let excess_tokens = current_balance - ctx.accounts.agreement.expected_tokens;
        if excess_tokens > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.project_token_account.to_account_info(),
                        to: ctx.accounts.company_token_account.to_account_info(),
                        authority: ctx.accounts.agreement.to_account_info(),
                    },
                    &[&[
                        AGREEMENT_SEED.as_bytes(),
                        ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                        &[ctx.accounts.agreement.bump],
                    ]],
                ),
                excess_tokens,
            )?;
        }

        ctx.accounts.agreement.seller_deposited = true;
        ctx.accounts.agreement.first_deposit_time = ctx.accounts.clock.unix_timestamp;

        Ok(())
    }
}
