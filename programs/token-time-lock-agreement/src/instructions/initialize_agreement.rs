use anchor_lang::{
    prelude::*,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::token::{initialize_account, InitializeAccount, Mint, Token, TokenAccount};

use std::mem::size_of;

use crate::{
    error::ErrorCode, Agreement, Settings, AGREEMENT_SEED, PAYMENT_SEED, PROJECT_SEED,
    SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: InitializeAgreementParams)]
pub struct InitializeAgreement<'info> {
    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump
    )]
    pub settings: Account<'info, Settings>,
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<Agreement>(),
        seeds = [AGREEMENT_SEED.as_bytes(), params.agreement_id.to_le_bytes().as_ref()],
        bump
    )]
    pub agreement: Account<'info, Agreement>,
    pub payment_token_mint: Account<'info, Mint>,
    pub project_token_mint: Account<'info, Mint>,
    /// CHECK: Valid PDA, will be initialized.
    #[account(mut,
        seeds = [PAYMENT_SEED.as_bytes(), params.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub payment_token_account: UncheckedAccount<'info>,
    /// CHECK: Valid PDA, will be initialized.
    #[account(mut,
        seeds = [PROJECT_SEED.as_bytes(), params.agreement_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub project_token_account: UncheckedAccount<'info>,
    /// CHECK: can be arbitrary account
    pub company_wallet: UncheckedAccount<'info>,
    /// CHECK: can be arbitrary account
    pub token_seller: AccountInfo<'info>,
    #[account(mut,
        address = settings.owner @ ErrorCode::NotAuthorized,
    )]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeAgreementParams {
    pub agreement_id: u64,
    pub expected_payment: u64,
    pub expected_tokens: u64,
    pub hold_duration: i64,
}

impl<'info> InitializeAgreement<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        params: InitializeAgreementParams,
    ) -> Result<()> {
        if params.expected_payment == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        if params.expected_tokens == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }
        if params.hold_duration <= 0 {
            return Err(ErrorCode::InvalidHoldDuration.into());
        }

        // Create token accounts for payment and project tokens
        for (mint, token, seeds) in &[
            (
                &ctx.accounts.payment_token_mint,
                &ctx.accounts.payment_token_account,
                [
                    PAYMENT_SEED.as_bytes(),
                    params.agreement_id.to_le_bytes().as_ref(),
                    &[ctx.bumps.payment_token_account],
                ],
            ),
            (
                &ctx.accounts.project_token_mint,
                &ctx.accounts.project_token_account,
                [
                    PROJECT_SEED.as_bytes(),
                    params.agreement_id.to_le_bytes().as_ref(),
                    &[ctx.bumps.project_token_account],
                ],
            ),
        ] {
            create_account(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    CreateAccount {
                        from: ctx.accounts.payer.to_account_info(),
                        to: token.to_account_info(),
                    },
                    &[seeds],
                ),
                ctx.accounts.rent.minimum_balance(TokenAccount::LEN),
                TokenAccount::LEN as u64,
                ctx.accounts.token_program.key,
            )?;

            initialize_account(CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                InitializeAccount {
                    account: token.to_account_info(),
                    mint: mint.to_account_info(),
                    authority: ctx.accounts.agreement.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[seeds],
            ))?;
        }

        let agreement = &mut ctx.accounts.agreement;
        agreement.agreement_id = params.agreement_id;
        agreement.payment_token_mint = *ctx.accounts.payment_token_mint.to_account_info().key;
        agreement.project_token_mint = *ctx.accounts.project_token_mint.to_account_info().key;
        agreement.company_wallet = ctx.accounts.company_wallet.key();
        agreement.token_seller = *ctx.accounts.token_seller.key;
        agreement.expected_payment = params.expected_payment;
        agreement.expected_tokens = params.expected_tokens;
        agreement.hold_duration = params.hold_duration;
        agreement.bump = ctx.bumps.agreement;
        agreement.owner = *ctx.accounts.payer.key;

        Ok(())
    }
}
