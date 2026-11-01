//! Recovered referral withdrawal; original executable remains the comparison target.
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::FeeError;
pub use crate::instructions::withdraw::execute;

















#[derive(Accounts)]
pub struct Withdraw<'info>{
 #[account(mut)]pub user:Signer<'info>,
 #[account(mut,seeds=[b"referral-vault",user.key().as_ref()],bump)]pub referral_vault:SystemAccount<'info>,
 pub quote_mint:Option<InterfaceAccount<'info,Mint>>,
 #[account(mut,token::mint=quote_mint,token::authority=referral_vault,token::token_program=token_program)]pub vault_ata:Option<InterfaceAccount<'info,TokenAccount>>,
 #[account(mut,token::mint=quote_mint,token::authority=user,token::token_program=token_program)]pub user_ata:Option<InterfaceAccount<'info,TokenAccount>>,
 pub token_program:Option<Interface<'info,TokenInterface>>,
 pub system_program:Program<'info,System>,
 /// CHECK: canonical event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: CPI receives this account; no typed program constraint observed.
 pub program:AccountInfo<'info>,
}
#[event]
pub struct WithdrawEvent{pub timestamp:i64,pub user:Pubkey,pub quote_mint:Pubkey,pub amount:u64}
