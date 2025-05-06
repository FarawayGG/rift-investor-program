use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer};

use crate::{error::ErrorCode, Agreement, Investor, AGREEMENT_SEED, INVESTOR_SEED, PAYMENT_SEED};

#[derive(Accounts)]
#[instruction(params: DepositStablecoinsParams)]
pub struct DepositStablecoins<'info> {
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
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct DepositStablecoinsParams {
    pub amount: u64,
}

impl<'info> DepositStablecoins<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        params: DepositStablecoinsParams,
    ) -> Result<()> {
        if params.amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        if !ctx.accounts.agreement.seller_deposited {
            return Err(ErrorCode::SellerMustDepositFirst.into());
        }
        if ctx.accounts.agreement.token_seller == *ctx.accounts.payer.key {
            return Err(ErrorCode::SellerCannotDepositStablecoins.into());
        }

        if let Some(investor) = &mut ctx.accounts.investor {
            if params.amount != investor.required_amount {
                return Err(ErrorCode::InvalidAmount.into());
            }
            if investor.investor_deposited {
                return Err(ErrorCode::AlreadyDeposited.into());
            }
            if ctx.accounts.agreement.total_invested + params.amount
                > ctx.accounts.agreement.expected_payment
            {
                return Err(ErrorCode::DepositExceedsExpectedPayment.into());
            }

            let (expected_destination_token_account, _) = Pubkey::find_program_address(
                &[
                    PAYMENT_SEED.as_bytes(),
                    ctx.accounts.agreement.agreement_id.to_le_bytes().as_ref(),
                ],
                ctx.program_id,
            );
            if expected_destination_token_account != ctx.accounts.destination_token_account.key() {
                return Err(ErrorCode::InvalidDestination.into());
            }

            ctx.accounts.agreement.total_invested += params.amount;
            if ctx.accounts.agreement.total_invested == ctx.accounts.agreement.expected_payment
                && ctx.accounts.agreement.hold_duration_start == 0
            {
                ctx.accounts.agreement.hold_duration_start = ctx.accounts.clock.unix_timestamp;
            }

            investor.investor_deposited = true;
        } else {
            if ctx.accounts.destination_token_account.owner.key()
                != ctx.accounts.agreement.company_wallet
                || ctx.accounts.destination_token_account.mint
                    != ctx.accounts.agreement.payment_token_mint
            {
                return Err(ErrorCode::InvalidDestination.into());
            }
        }

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.destination_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            ),
            params.amount,
        )?;

        Ok(())
    }
}
