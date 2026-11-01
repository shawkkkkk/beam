use anchor_lang::prelude::*; use anchor_spl::{associated_token::AssociatedToken,token_interface::{Mint,TokenAccount,TokenInterface}}; use crate::anchor_framework::*;
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.

#[event_cpi]
#[derive(Accounts)]
#[instruction(config_id: [u8;32], epoch: u128)]
pub struct CloseDonationEpochV1<'info> {
    #[account(init, payer = signer, space = 98, seeds = [b"distribution_v1",config_id.as_ref(),mint.key().as_ref(),&epoch.to_le_bytes()], bump)]
    pub distribution: Account<'info,DistributionV1>,
    #[account(init, payer = signer, associated_token::mint = mint, associated_token::authority = distribution, associated_token::token_program = token_program)]
    pub distribution_token_account: InterfaceAccount<'info,TokenAccount>,
    #[account(mut, seeds = [b"epoch_tracker_v1", config_id.as_ref(), mint.key().as_ref()], bump = epoch_tracker.bump)]
    pub epoch_tracker: Account<'info,EpochTrackerV1>,
    #[account(mut, seeds = [b"debouncer_v1", config_id.as_ref(), mint.key().as_ref()], bump = debouncer.bump)]
    pub debouncer: Account<'info,DebouncerV1>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = debouncer, associated_token::token_program = token_program)]
    pub debouncer_token_account: InterfaceAccount<'info,TokenAccount>,
    #[account(mut, mint::token_program = token_program, address = debouncer.mint)]
    pub mint: Box<InterfaceAccount<'info,Mint>>,
    pub token_program: Interface<'info,TokenInterface>,
    #[account(mut, address = Pubkey::new_from_array(crate::codec::constants::RELAYER))]
    pub signer: Signer<'info>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub system_program: Program<'info,System>,
}
