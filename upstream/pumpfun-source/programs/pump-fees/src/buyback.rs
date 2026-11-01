//! Partial buyback reconstruction from signed local differential evidence.
use anchor_lang::prelude::*;
use crate::FeeError;
use crate::types::FeeProgramGlobal;
pub use crate::instructions::buyback::initialize_buyback;
pub use crate::instructions::admin::update_buyback_authority::update_buyback_authority;
pub use crate::instructions::admin::update_buyback_authority::update_buyback_claim_rate_limit;
pub use crate::instructions::buyback::sweep_buyback;













pub use crate::instructions::buyback::*;












#[derive(Accounts)]
#[instruction(index:u8)]
pub struct UpdateBuyback<'info>{
 #[account(mut)]pub authority:Signer<'info>,
 #[account(mut,seeds=[b"fee-program-global"],bump=fee_program_global.bump,has_one=authority @ FeeError::NotAuthorized)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 #[account(mut,seeds=[b"buyback-vault".as_ref(),&[index]],bump)]pub buyback_vault:Box<Account<'info,BuybackVault>>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: event target.
 pub program:AccountInfo<'info>,
}
#[derive(Accounts)]
#[instruction(index:u8)]
pub struct SweepBuyback<'info>{
 /// CHECK: arbitrary destination.
 #[account(mut)]pub destination:UncheckedAccount<'info>,
 #[account(mut)]pub authority:Signer<'info>,
 #[account(mut,seeds=[b"buyback-vault".as_ref(),&[index]],bump,has_one=authority @ FeeError::NotAuthorized)]pub buyback_vault:Box<Account<'info,BuybackVault>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=buyback_vault,associated_token::token_program=token_program)]pub buyback_vault_ata:Box<InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=destination,associated_token::token_program=token_program)]pub destination_ata:Box<InterfaceAccount<'info,anchor_spl::token_interface::TokenAccount>>,
 pub system_program:Program<'info,System>,pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: event target.
 pub program:AccountInfo<'info>,
}
pub use crate::state::BuybackVault;
#[event]
pub struct SweepBuybackEvent{pub index:u8,pub sol_amount:u64,pub token_amount:u64,pub destination:Pubkey,pub buyback_vault:Pubkey,pub mint:Pubkey}
