use anchor_lang::prelude::*; use anchor_spl::{associated_token::AssociatedToken,token_interface::{Mint,TokenAccount,TokenInterface}}; use crate::anchor_framework::*;
#[event_cpi]
#[derive(Accounts)]
pub struct DistributeNonSwapOutputMintV1<'info> {
 #[account(seeds=[b"epoch_tracker_v1",distribution.config_id.as_ref(),mint.key().as_ref()],bump=epoch_tracker.bump)]
 pub epoch_tracker:Account<'info,EpochTrackerV1>,
 #[account(seeds=[b"debouncer_v1",distribution.config_id.as_ref(),mint.key().as_ref()],bump=debouncer.bump)]
 pub debouncer:Account<'info,DebouncerV1>,
 #[account(mut,close=signer,seeds=[b"distribution_v1",distribution.config_id.as_ref(),distribution.mint.as_ref(),&distribution.epoch.to_le_bytes()],bump=distribution.bump)]
 pub distribution:Account<'info,DistributionV1>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=distribution,associated_token::token_program=token_program)]
 pub distribution_token_account:InterfaceAccount<'info,TokenAccount>,
 #[account(mut,mint::token_program=token_program,address=Pubkey::new_from_array(crate::codec::constants::SETTLEMENT) @ crate::business_error_codes::ErrorCode::InvalidMint)]
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,address=Pubkey::new_from_array(crate::codec::constants::RELAYER))]
 pub signer:Signer<'info>,
 pub token_program:Interface<'info,TokenInterface>,
 pub associated_token_program:Program<'info,AssociatedToken>,
 pub system_program:Program<'info,System>,
}
