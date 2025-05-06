use std::mem::size_of;

use anchor_lang::{
    prelude::*,
    system_program::{self, create_account, CreateAccount},
};

use crate::{
    error::ErrorCode, Agreement, Investor, Settings, AGREEMENT_SEED, INVESTOR_SEED, SETTINGS_SEED,
};

#[derive(Accounts)]
#[instruction(params: AddInvestorsParams)]
pub struct AddInvestors<'info> {
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
    #[account(address = settings.owner @ ErrorCode::NotAuthorized)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct AddInvestorsParams {
    pub allocations: Vec<InvestorAllocation>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InvestorAllocation {
    pub amount: u64,
    pub token_amount: u64,
    pub wallet: Pubkey,
}

impl<'info> AddInvestors<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        params: AddInvestorsParams,
    ) -> Result<()> {
        let agreement = &mut ctx.accounts.agreement;
        let remaining_accounts = ctx.remaining_accounts;

        if ctx.remaining_accounts.len() != params.allocations.len() {
            return Err(ErrorCode::InvalidRemainingAccounts.into());
        }

        if agreement.seller_deposited {
            return Err(ErrorCode::SellerDeposited.into());
        }

        let mut total_new_allocations = 0;
        let mut total_new_payment_allocations = 0;

        for allocation in params.allocations.iter() {
            total_new_payment_allocations += allocation.amount;
            total_new_allocations += allocation.token_amount;
        }

        if agreement.total_required + total_new_payment_allocations > agreement.expected_payment {
            return Err(ErrorCode::TotalAllocationsExceedTarget.into());
        }

        if agreement.total_token_allocation + total_new_allocations > agreement.expected_tokens {
            return Err(ErrorCode::TotalTokenAllocationsExceedTarget.into());
        }

        // Process each investor
        // TODO: Check number of allocations
        let mut seen = std::collections::HashSet::new();
        for (i, allocation) in params.allocations.iter().enumerate() {
            let investor_account = &remaining_accounts[i];

            if !seen.insert(allocation.wallet) {
                return Err(ErrorCode::DuplicateInvestor.into());
            }

            // Generate PDA
            let (investor_pda, bump) = Pubkey::find_program_address(
                &[
                    INVESTOR_SEED.as_bytes(),
                    agreement.key().as_ref(),
                    allocation.wallet.as_ref(),
                ],
                ctx.program_id,
            );

            if *investor_account.owner != system_program::ID {
                return Err(ErrorCode::InvalidInvestorAccountOwner.into());
            }

            if investor_account.lamports() != 0 {
                return Err(ErrorCode::InvestorAlreadyExists.into());
            }

            if investor_pda != investor_account.key() {
                return Err(ErrorCode::InvalidInvestorAccount.into());
            }

            if allocation.amount == 0 {
                return Err(ErrorCode::InvalidAmount.into());
            }

            if allocation.token_amount == 0 {
                return Err(ErrorCode::InvalidAmount.into());
            }

            let investor_len = 8 + size_of::<Investor>();

            create_account(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    CreateAccount {
                        from: ctx.accounts.payer.to_account_info(),
                        to: investor_account.to_account_info(),
                    },
                    &[&[
                        INVESTOR_SEED.as_bytes(),
                        agreement.key().as_ref(),
                        allocation.wallet.as_ref(),
                        &[bump],
                    ]],
                ),
                Rent::get()?.minimum_balance(investor_len),
                investor_len as u64,
                ctx.program_id,
            )?;

            // Initialize investor account
            let mut investor = Investor::default();
            investor.agreement = agreement.key();
            investor.wallet = allocation.wallet;
            investor.token_allocation = allocation.token_amount;
            investor.required_amount = allocation.amount;
            investor.has_withdrawn_tokens = false;
            investor.investor_deposited = false;
            investor.bump = bump;

            investor.try_serialize(&mut *investor_account.try_borrow_mut_data()?)?;

            agreement.investors_count += 1;
            agreement.total_required += allocation.amount;
            agreement.total_token_allocation += allocation.token_amount;
        }

        Ok(())
    }
}
