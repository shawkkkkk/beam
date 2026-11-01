//! Bodies only, after exact context admission. Parse/init, event, return and exit
//! are separate phases; these functions do not assert transaction commitment.
use crate::{*,runtime::*,arithmetic::*};
use crate::codec::constants::*;
pub struct DonationContext<'a> {
 pub tracker:&'a mut Tracker,pub debouncer:&'a mut Debouncer,pub whitelist:&'a Whitelist,
 pub epoch_bump:u8,pub debouncer_bump:u8,pub mint:Account,pub source:Account,
 pub destination:Account,pub from:Account,pub token_program:Account,pub system:Account,
}
fn donation_shared(rt:&mut impl Runtime,c:&mut DonationContext,a:&DonationArgs)->R<(u128,u64,u64)> {
 if a.message.len() >= 256 {
  return Err(Error::Owned(crate::generated_exit::exit_error::invalid_message_length()));
 }
 if a.tip_bps > 10000 {
  return Err(Error::Owned(crate::generated_exit::exit_error::invalid_bps()));
 }
 let owner=crate::generated_exit::AccountRuntime::owner(rt,&c.mint);
 require(owner==TOKEN || owner==TOKEN2022,6029)?;
 if c.mint.key!=NATIVE && c.mint.key!=NATIVE2022 && !c.whitelist.mints.contains(&c.mint.key) {
  require(safe_mint_106(rt,c.mint)?,6031)?;
 }
 if c.tracker.state==State::Uninitialized {
  *c.tracker=Tracker{bump:c.epoch_bump,state:State::Initialized,config_id:a.config_id,mint:c.mint.key,current_epoch:0};
 }
 if c.debouncer.state==State::Uninitialized {
  *c.debouncer=Debouncer{bump:c.debouncer_bump,state:State::Initialized,config_id:a.config_id,mint:c.mint.key,total_amount:0};
 }
 require(c.tracker.config_id==a.config_id,6002)?;
 require(c.debouncer.config_id==a.config_id,6002)?;
 require(c.debouncer.mint==c.mint.key,6006)?;
 require(c.debouncer.state==State::Initialized,6007)?;
 require(c.tracker.mint==c.mint.key,6006)?;
 require(c.tracker.state==State::Initialized,6007)?;
 let before=rt.token(c.destination)?.amount;
 let decimals=rt.mint(c.mint)?.decimals;
 transfer_checked(rt,c.token_program,c.source,c.mint,c.destination,c.from,a.amount,decimals,&vec![])?;
 let after=rt.token(c.destination)?.amount;
 let delta=after.checked_sub(before).ok_or_else(overflow)?;
 let product=delta.checked_mul(u64::from(a.tip_bps)).ok_or_else(overflow)?;
 let total=c.debouncer.total_amount.checked_add(delta).ok_or_else(overflow)?;
 c.debouncer.total_amount=total;
 Ok((c.tracker.current_epoch,delta,product/10000))
}
fn donation_body(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs,v:DonationVariant)->R<Event> {
 let native=matches!(v,DonationVariant::NativeBytes|DonationVariant::NativePubkey);
 if native {native_fund_sync(rt,c.system,c.token_program,c.from,c.source,a.amount)?;}
 let (epoch,gross,tip)=donation_shared(rt,c,&a)?;
 if native {close_token(rt,c.token_program,c.source,c.from,c.from,&vec![])?;}
 Ok(Event::Made{config_id:a.config_id,mint:c.mint.key,epoch,gross,tip,message:a.message,credited_to:a.credited_to})
}
pub fn donate_v1(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs)->R<Event>{donation_body(rt,c,a,DonationVariant::TokenBytes)}
pub fn donate_pubkey_config_id_v1(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs)->R<Event>{donation_body(rt,c,a,DonationVariant::TokenPubkey)}
pub fn donate_pubkey_config_id_with_payer_v1(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs)->R<Event>{donation_body(rt,c,a,DonationVariant::TokenPubkeyWithPayer)}
pub fn donate_native_v1(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs)->R<Event>{donation_body(rt,c,a,DonationVariant::NativeBytes)}
pub fn donate_native_pubkey_config_id_v1(rt:&mut impl Runtime,c:&mut DonationContext,a:DonationArgs)->R<Event>{donation_body(rt,c,a,DonationVariant::NativePubkey)}
fn authorize(signer:Account)->R<()> {require(signer.key==RELAYER,6015)}
pub fn upsert_mint_whitelist_v1(signer:Account,w:&mut Whitelist,bump:u8,mints:Vec<Key>)->R<()> {
 authorize(signer)?;require(mints.len()<=32,6032)?;w.mints=mints;w.bump=bump;Ok(())
}
pub struct CloseContext<'a> {
 pub signer:Account,pub tracker:&'a mut Tracker,pub debouncer:&'a mut Debouncer,
 pub distribution:&'a mut Distribution,pub distribution_bump:u8,
 pub source:Account,pub destination:Account,pub authority:Account,pub mint:MintView,pub token_program:Account,
}
pub fn close_donation_epoch_v1(rt:&mut impl Runtime,c:&mut CloseContext,_config_id:Key,epoch:u128)->R<Event> {
 authorize(c.signer)?;
 let gross=c.debouncer.total_amount;require(gross!=0,6012)?;
 require(c.tracker.current_epoch==epoch,6011)?;
 let old_epoch=c.tracker.current_epoch;
 *c.distribution=Distribution{bump:c.distribution_bump,state:State::Initialized,config_id:c.debouncer.config_id,mint:c.debouncer.mint,epoch:old_epoch,total_amount:gross};
 transfer_checked(rt,c.token_program,c.source,c.mint.account,c.destination,c.authority,gross,c.mint.decimals,&debouncer_seeds(c.debouncer))?;
 c.debouncer.total_amount=0;
 c.tracker.current_epoch=c.tracker.current_epoch.checked_add(1).ok_or_else(overflow)?;
 Ok(Event::Closed{config_id:c.debouncer.config_id,mint:c.debouncer.mint,epoch:old_epoch,gross})
}
// K+16=source, +64=mint, +112=program, +160=authority. Corrects prior prose inversion.
pub struct Settlement<'a> {pub source:&'a TokenView,pub mint:&'a MintView,pub program:&'a Account,pub authority:&'a Account,pub relayer:&'a Account}
fn distribution108(rt:&mut impl Runtime,s:&Settlement<'_>,d:&Distribution,shares:&[Share],recipients:&[Account],total:u64,expected_program:Key)->R<Vec<TransferReceipt>> {
 require(s.program.key==expected_program,6025)?;
 require(recipients.len()>=shares.len(),6030)?;
 let amounts=fair_amounts(total,shares); // admitted sum established by each caller
 for (i,share) in shares.iter().enumerate() {
  let t=rt.token(recipients[i])?;
  require(t.authority==share.authority,6014)?;
  require(t.mint==s.mint.account.key,6024)?;
  require(t.account.owner==expected_program,6025)?;
  if amounts[i]!=0 {transfer_checked(rt,*s.program,s.source.account,s.mint.account,recipients[i],*s.authority,amounts[i],s.mint.decimals,&distribution_seeds(d))?;}
 }
 Ok(shares.iter().cloned().zip(amounts).map(|(share,amount)|TransferReceipt{share,amount}).collect())
}
pub struct DistributionContext<'a> {pub distribution:&'a Distribution,pub current_epoch:u128,pub debouncer_mint:Key,pub settlement:Settlement<'a>}
fn distribute_body(rt:&mut impl Runtime,c:&DistributionContext,shares:&[Share],recipients:&[Account],receipt:Option<&SwapReceipt>)->R<Event> {
 let d=c.distribution;let s=&c.settlement;authorize(*s.relayer)?;
 require(d.epoch<c.current_epoch,6028)?;
 require(d.state==State::Initialized,6007)?;
 if receipt.is_some() {require(d.mint==c.debouncer_mint,6006)?;} else {
  require(d.mint==s.mint.account.key,6006)?;require(c.debouncer_mint==s.mint.account.key,6006)?;
 }
 admit_weights(shares)?;
 let amount=match receipt {Some(r)=>{require(r.output_amount>0,6012)?;r.output_amount},None=>d.total_amount};
 require(s.source.amount>=amount,6012)?;
 require(recipients.len()>=shares.len(),6030)?;
 let receipts=distribution108(rt,s,d,shares,recipients,amount,s.program.key)?;
 burn_then_close(rt,*s.program,s.source.account,s.mint.account,*s.authority,*s.relayer,&distribution_seeds(d))?;
 Ok(Event::Processed{config_id:d.config_id,mint:d.mint,epoch:d.epoch,amount,settlement:s.mint.account.key,receipts})
}
pub fn distribute_after_just_swapping_v1(rt:&mut impl Runtime,c:&DistributionContext,r:&SwapReceipt,shares:&[Share],remaining:&[Account])->R<Event> {
 // Parser146 MUST already bind r.distribution to the distribution account (6033).
 distribute_body(rt,c,shares,remaining,Some(r))
}
pub fn distribute_non_swap_output_mint_v1(rt:&mut impl Runtime,c:&DistributionContext,shares:&[Share],remaining:&[Account])->R<Event> {
 distribute_body(rt,c,shares,remaining,None)
}
pub struct SwapContext<'a> {pub distribution:&'a Distribution,pub current_epoch:u128,pub debouncer_mint:Key,pub input:Settlement<'a>,pub output:Settlement<'a>,pub jupiter:Account}
fn observed_swap(rt:&mut impl Runtime,c:&SwapContext,data:&[u8],accounts:&[Account])->R<u64> {
 let metas=accounts.iter().map(|a|Meta{key:a.key,writable:a.writable,signer:a.key==c.input.authority.key}).collect();
 // Amounts are cached context values BEFORE invocation. Reload input then output.
 let before_in=c.input.source.amount;let before_out=c.output.source.amount;
 rt.invoke_signed(Cpi{program:c.jupiter.key,metas,data:data.to_vec()},accounts,&distribution_seeds(c.distribution))?;
 let after_in=rt.token(c.input.source.account)?.amount;
 let after_out=rt.token(c.output.source.account)?.amount;
 let spent=before_in.checked_sub(after_in).ok_or_else(overflow)?;
 require(spent==c.distribution.total_amount,6021)?;
 let output=after_out.checked_sub(before_out).ok_or_else(overflow)?;
 require(output>0,6023)?;Ok(output)
}
pub fn just_swap_v1(rt:&mut impl Runtime,c:&SwapContext,receipt:&mut SwapReceipt,receipt_account:Account,receipt_bump:u8,data:&[u8],remaining:&[Account])->R<Event> {
 let d=c.distribution;authorize(*c.input.relayer)?;
 require(d.epoch<c.current_epoch,6028)?;require(d.state==State::Initialized,6007)?;
 require(c.debouncer_mint==d.mint,6006)?;require(d.mint!=SETTLEMENT,6006)?;
 let amount=observed_swap(rt,c,data,remaining)?;
 *receipt=SwapReceipt{bump:receipt_bump,distribution:c.input.authority.key,output_amount:amount};
 let s=&c.input;burn_then_close(rt,*s.program,s.source.account,s.mint.account,*s.authority,*s.relayer,&distribution_seeds(d))?;
 Ok(Event::Swapped{config_id:d.config_id,mint:d.mint,epoch:d.epoch,distribution:s.authority.key,receipt:receipt_account.key,output_vault:c.output.source.account.key,amount,settlement:c.output.mint.account.key})
}
pub fn swap_then_distribute_v1(rt:&mut impl Runtime,c:&SwapContext,data:&[u8],shares:&[Share],remaining:&[Account])->R<Event> {
 let d=c.distribution;authorize(*c.input.relayer)?;require(d.epoch<c.current_epoch,6028)?;
 // Do NOT transplant just_swap's three additional business checks here.
 admit_weights(shares)?;require(remaining.len()>=shares.len(),6030)?;
 let (recipients,jupiter)=remaining.split_at(shares.len());
 let amount=observed_swap(rt,c,data,jupiter)?;
 let receipts=distribution108(rt,&c.output,d,shares,recipients,amount,c.output.program.key)?;
 let s=&c.input;burn_then_close(rt,*s.program,s.source.account,s.mint.account,*s.authority,*s.relayer,&distribution_seeds(d))?;
 let o=&c.output;close_token(rt,*o.program,o.source.account,*o.relayer,*o.authority,&distribution_seeds(d))?;
 Ok(Event::Processed{config_id:d.config_id,mint:d.mint,epoch:d.epoch,amount,settlement:o.mint.account.key,receipts})
}
