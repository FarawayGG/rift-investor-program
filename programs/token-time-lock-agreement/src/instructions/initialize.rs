use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::{Settings, SETTINGS_SEED};

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<Settings>(),
        seeds = [SETTINGS_SEED.as_bytes()],
        bump
    )]
    pub settings: Account<'info, Settings>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub cancel_timeout: i64,
    pub commission_basis_points: u16,
    pub owner: Pubkey,
}

impl<'info> Initialize<'info> {
    pub fn handle(
        ctx: &mut Context<'_, '_, 'info, 'info, Self>,
        params: InitializeParams,
    ) -> Result<()> {
        ctx.accounts.settings.cancel_timeout = params.cancel_timeout;
        ctx.accounts.settings.commission_basis_points = params.commission_basis_points;
        ctx.accounts.settings.owner = params.owner;
        Ok(())
    }
}
