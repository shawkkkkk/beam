use anchor_lang::prelude::*;
use crate::types::{FeeConfig, FeeProgramGlobal};
use crate::FeeError;


#[event_cpi]
#[derive(Accounts)]
pub struct InitializeFeeProgramGlobal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"global"], bump, seeds::program = crate::pump_global::PUMP_ID,
              has_one = authority @ FeeError::NotAuthorized)]
    pub pump_global: Account<'info, crate::pump_global::Global>,
    #[account(init, payer = authority, space = 338, seeds = [b"fee-program-global"], bump)]
    pub fee_program_global: Account<'info, FeeProgramGlobal>,
    pub system_program: Program<'info, System>,
}
