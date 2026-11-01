use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::{Instruction,AccountMeta},program::invoke_signed,system_instruction};
use anchor_spl::{token_interface::{Mint,TokenAccount,TokenInterface},associated_token::AssociatedToken};
use crate::{Global,PumpError};
#[event_cpi]
#[derive(Accounts)]
pub struct DistributeFeeToHolders<'info>{
 #[account(seeds=[b"global"],bump,has_one=holder_reward_claim_authority @ PumpError::NotAuthorized)]pub global:Box<Account<'info,Global>>,
 #[account(mut)]pub holder_reward_claim_authority:Signer<'info>,
 /// CHECK: mint key identifies the rewards PDA.
 pub mint:UncheckedAccount<'info>,
 /// CHECK: System transfers enforce the source account ownership.
 #[account(mut,seeds=[b"holder-rewards",mint.key().as_ref()],bump)]pub holder_rewards:UncheckedAccount<'info>,
 #[account(mut,token::authority=holder_rewards,token::mint=quote_mint,token::token_program=quote_token_program)]pub holder_rewards_token_account:Option<Box<InterfaceAccount<'info,TokenAccount>>>,
 pub quote_mint:Box<InterfaceAccount<'info,Mint>>,
 pub quote_token_program:Interface<'info,TokenInterface>,
 pub associated_token_program:Program<'info,AssociatedToken>,
 pub system_program:Program<'info,System>,
}
#[event]#[derive(Clone)]pub struct DistributeFeeToHoldersEvent{pub timestamp:i64,pub mint:Pubkey,pub quote_mint:Pubkey,pub recipients:u64,pub total:u64}
fn transfer_tokens<'info>(token_program:&Interface<'info,TokenInterface>,from:impl ToAccountInfo<'info>,mint:&InterfaceAccount<'info,Mint>,to:&AccountInfo<'info>,authority:impl ToAccountInfo<'info>,amount:u64,signer_seeds:&[&[&[u8]]])->Result<()>{ if amount==0{return Ok(())} anchor_spl::token_2022::transfer_checked(CpiContext::new_with_signer(token_program.to_account_info(),anchor_spl::token_2022::TransferChecked{from:from.to_account_info(),to:to.clone(),authority:authority.to_account_info(),mint:mint.to_account_info()},signer_seeds),amount,mint.decimals)}
pub fn execute<'info>(ctx:Context<'_, '_, '_, 'info,DistributeFeeToHolders<'info>>,amounts:Vec<u64>)->Result<()>{
  let a=&ctx.accounts;let remaining_accounts=ctx.remaining_accounts;let holder_rewards=&a.holder_rewards;
// layout-pad(58): aligns the next Source site with the original line

























































 require!(ctx.remaining_accounts.len()==amounts.len().saturating_mul(2),PumpError::HolderRewardRecipientsMismatch);
 let mint=a.mint.key();let seeds:&[&[u8]]=&[b"holder-rewards",mint.as_ref(),&[ctx.bumps.holder_rewards]];
 let native=a.quote_mint.key()==anchor_spl::token::spl_token::native_mint::ID;
 if native {
  if let Some(source)=&a.holder_rewards_token_account {anchor_spl::token_2022::close_account(CpiContext::new_with_signer(a.quote_token_program.to_account_info(),anchor_spl::token_2022::CloseAccount{account:source.to_account_info(),destination:holder_rewards.to_account_info(),authority:holder_rewards.to_account_info()},&[seeds]))?;}
 }
 let mut total=0u64;
// layout-pad(23): aligns the next Source site with the original line






















 for (accounts,amount) in remaining_accounts.chunks(2).zip(amounts.iter()){
  let amount=*amount;
    let recipient = &accounts[0];
  if native {
   let signer=&[seeds];crate::migrate::transfer_native(a.system_program.as_ref(),holder_rewards,recipient,amount,signer)?;
  }else{
// layout-pad(2): aligns the next Source site with the original line

   let source=a.holder_rewards_token_account.as_ref().ok_or_else(||error!(PumpError::HolderRewardTokenAccountMissing))?;
   let recipient_associated_token_account=&accounts[1];
   require_keys_eq!(recipient_associated_token_account.key(),anchor_spl::associated_token::get_associated_token_address_with_program_id(&recipient.key(),&a.quote_mint.key(),&a.quote_token_program.key()),PumpError::HolderRewardRecipientsMismatch);
   if recipient_associated_token_account.data_is_empty(){
    anchor_spl::associated_token::create(CpiContext::new(a.associated_token_program.to_account_info(),anchor_spl::associated_token::Create{payer:a.holder_reward_claim_authority.to_account_info(),associated_token:recipient_associated_token_account.clone(),authority:recipient.clone(),mint:a.quote_mint.to_account_info(),system_program:a.system_program.to_account_info(),token_program:a.quote_token_program.to_account_info()}))?;
   }
   transfer_tokens(&a.quote_token_program,source.to_account_info(),&a.quote_mint,recipient_associated_token_account,holder_rewards,amount,&[seeds])?;
  }
// layout-pad(29): aligns the next Source site with the original line




























  total=total.checked_add(amount).ok_or_else(||error!(PumpError::Overflow))?;
 }
// layout-pad(5): aligns the next Source site with the original line




 if native {let balance=holder_rewards.lamports();require!(balance==0||balance>=Rent::get()?.minimum_balance(0),PumpError::HolderRewardsRentFloor);}
 let e=DistributeFeeToHoldersEvent{timestamp:Clock::get()?.unix_timestamp,mint,quote_mint:if native {Pubkey::default()}else{a.quote_mint.key()},recipients:amounts.len() as u64,total};emit!(e);emit_cpi!(e);Ok(())
}
