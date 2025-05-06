pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("93KzY6AXgrgxL5T1MwLJWa7YbJgFKbZhWHDP62URuY9c");

#[program]
pub mod token_time_lock_agreement {
    use super::*;

    pub fn initialize<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, Initialize<'info>>,
        params: InitializeParams,
    ) -> Result<()> {
        Initialize::handle(&mut ctx, params)
    }

    pub fn initialize_agreement<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, InitializeAgreement<'info>>,
        params: InitializeAgreementParams,
    ) -> Result<()> {
        InitializeAgreement::handle(&mut ctx, params)
    }

    pub fn add_investors<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, AddInvestors<'info>>,
        params: AddInvestorsParams,
    ) -> Result<()> {
        AddInvestors::handle(&mut ctx, params)
    }

    pub fn deposit_stablecoins<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, DepositStablecoins<'info>>,
        params: DepositStablecoinsParams,
    ) -> Result<()> {
        DepositStablecoins::handle(&mut ctx, params)
    }

    pub fn process_token_deposit<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, ProcessTokenDeposit<'info>>,
        params: ProcessTokenDepositParams,
    ) -> Result<()> {
        ProcessTokenDeposit::handle(&mut ctx, params)
    }

    pub fn withdraw_excess_tokens<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawExcessTokens<'info>>,
        params: WithdrawExcessTokensParams,
    ) -> Result<()> {
        WithdrawExcessTokens::handle(&mut ctx, params)
    }

    pub fn withdraw_funds<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawFunds<'info>>,
        params: WithdrawFundsParams,
    ) -> Result<()> {
        WithdrawFunds::handle(&mut ctx, params)
    }

    pub fn withdraw_tokens<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawTokens<'info>>,
        params: WithdrawTokensParams,
    ) -> Result<()> {
        WithdrawTokens::handle(&mut ctx, params)
    }

    pub fn withdraw_tokens_batch<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawTokensBatch<'info>>,
        params: WithdrawTokensBatchParams,
    ) -> Result<()> {
        WithdrawTokensBatch::handle(&mut ctx, params)
    }

    pub fn cancel_agreement<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CancelAgreement<'info>>,
        params: CancelAgreementParams,
    ) -> Result<()> {
        CancelAgreement::handle(&mut ctx, params)
    }

    pub fn withdraw_cancelled_funds<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawCancelledFunds<'info>>,
        params: WithdrawCancelledFundsParams,
    ) -> Result<()> {
        WithdrawCancelledFunds::handle(&mut ctx, params)
    }

    pub fn withdraw_cancelled_funds_batch<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, WithdrawCancelledFundsBatch<'info>>,
        params: WithdrawCancelledFundsBatchParams,
    ) -> Result<()> {
        WithdrawCancelledFundsBatch::handle(&mut ctx, params)
    }
}
