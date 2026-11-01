use anchor_lang::prelude::*; use anchor_spl::{associated_token::AssociatedToken,token_interface::{Mint,TokenAccount,TokenInterface}}; use crate::anchor_framework::*;
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
// Preserve the independently decoded macro coordinate; prior historical text unknown.
#[event_cpi]
#[derive(Accounts)]
#[instruction(amount: u64, config_id: Pubkey, tip_bps: u16, message: String, credited_to: Pubkey)]
pub struct DonatePubkeyConfigIdV1<'info> {
    #[account(seeds = [b"epoch_tracker_v1", config_id.as_ref(), mint.key().as_ref()], bump, init_if_needed, payer = from, space = 90)]
    pub epoch_tracker: Account<'info,EpochTrackerV1>,
    #[account(seeds = [b"debouncer_v1", config_id.as_ref(), mint.key().as_ref()], bump, init_if_needed, payer = from, space = 82)]
    pub debouncer: Account<'info,DebouncerV1>,
    #[account(init_if_needed, payer = from, associated_token::mint = mint, associated_token::authority = debouncer, associated_token::token_program = token_program)]
    pub debouncer_token_account: InterfaceAccount<'info,TokenAccount>,
    #[account(mut, mint::token_program = token_program)]
    pub mint: Box<InterfaceAccount<'info,Mint>>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = from, associated_token::token_program = token_program)]
    pub from_token_account: InterfaceAccount<'info,TokenAccount>,
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(seeds = [b"mint_whitelist_v1"], bump = mint_whitelist.bump)]
    pub mint_whitelist: Account<'info,MintWhitelistV1>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub token_program: Interface<'info,TokenInterface>,
    pub system_program: Program<'info,System>,
}
