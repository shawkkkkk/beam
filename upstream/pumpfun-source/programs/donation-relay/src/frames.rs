//! Frames own actual decoded states; both views borrow these exact fields.
use crate::{*,admission::Loaded,handlers as body,generated_exit as exit};
use crate::dispatch_interfaces::*;
pub struct Epoch {
 pub(crate) distribution:Loaded<Distribution>,
 pub(crate) distribution_token_account:Loaded<TokenView>,
 pub(crate) epoch_tracker:Loaded<Tracker>,
 pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) debouncer_token_account:Loaded<TokenView>,
 pub(crate) mint:Loaded<MintView>,
 pub(crate) token_program:Account,
 pub(crate) signer:Account,
 pub(crate) associated_token_program:Account,
 pub(crate) system_program:Account,
 pub(crate) event_authority:Account,
 pub(crate) program:Account,
 pub(crate) event_bump:u8,
 pub(crate) distribution_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for Epoch {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
pub struct After {
 pub(crate) epoch_tracker:Loaded<Tracker>,
 pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) distribution:Loaded<Distribution>,
 pub(crate) swap_receipt:Loaded<SwapReceipt>,
 pub(crate) output_distribution_token_account:Loaded<TokenView>,
 pub(crate) output_mint:Loaded<MintView>,
 pub(crate) output_mint_program:Account,
 pub(crate) signer:Account,
 pub(crate) associated_token_program:Account,
 pub(crate) system_program:Account,
 pub(crate) event_authority:Account,
 pub(crate) program:Account,
 pub(crate) event_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for After {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
pub struct NonSwap {
 pub(crate) epoch_tracker:Loaded<Tracker>,
 pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) distribution:Loaded<Distribution>,
 pub(crate) distribution_token_account:Loaded<TokenView>,
 pub(crate) mint:Loaded<MintView>,
 pub(crate) signer:Account,
 pub(crate) token_program:Account,
 pub(crate) associated_token_program:Account,
 pub(crate) system_program:Account,
 pub(crate) event_authority:Account,
 pub(crate) program:Account,
 pub(crate) event_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for NonSwap {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
pub struct Just {
 pub(crate) epoch_tracker:Loaded<Tracker>,
 pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) distribution:Loaded<Distribution>,
 pub(crate) swap_receipt:Loaded<SwapReceipt>,
 pub(crate) distribution_token_account:Loaded<TokenView>,
 pub(crate) output_distribution_token_account:Loaded<TokenView>,
 pub(crate) mint:Loaded<MintView>,
 pub(crate) output_mint:Loaded<MintView>,
 pub(crate) output_mint_program:Account,
 pub(crate) signer:Account,
 pub(crate) token_program:Account,
 pub(crate) jupiter_program:Account,
 pub(crate) associated_token_program:Account,
 pub(crate) system_program:Account,
 pub(crate) event_authority:Account,
 pub(crate) program:Account,
 pub(crate) event_bump:u8,
 pub(crate) receipt_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for Just {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
pub struct Combined {
 pub(crate) epoch_tracker:Loaded<Tracker>,
 pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) distribution:Loaded<Distribution>,
 pub(crate) distribution_token_account:Loaded<TokenView>,
 pub(crate) output_distribution_token_account:Loaded<TokenView>,
 pub(crate) mint:Loaded<MintView>,
 pub(crate) output_mint:Loaded<MintView>,
 pub(crate) output_mint_program:Account,
 pub(crate) signer:Account,
 pub(crate) token_program:Account,
 pub(crate) jupiter_program:Account,
 pub(crate) associated_token_program:Account,
 pub(crate) system_program:Account,
 pub(crate) event_authority:Account,
 pub(crate) program:Account,
 pub(crate) event_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for Combined {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
pub struct WhitelistFrameData {
 pub(crate) mint_whitelist:Loaded<Whitelist>,
 pub(crate) signer:Account,
 pub(crate) system_program:Account,
 pub(crate) whitelist_bump:u8,
 pub(crate) remaining:Vec<Account>,
}
pub struct Donation {
 pub(crate) epoch_tracker:Loaded<Tracker>,pub(crate) debouncer:Loaded<Debouncer>,
 pub(crate) destination:Loaded<TokenView>,pub(crate) mint:Loaded<MintView>,pub(crate) source:Loaded<TokenView>,
 pub(crate) from:Account,pub(crate) payer:Account,pub(crate) whitelist:Loaded<Whitelist>,
 pub(crate) token_program:Account,pub(crate) system:Account,pub(crate) associated:Account,
 pub(crate) event_authority:Account,pub(crate) program:Account,
 pub(crate) epoch_bump:u8,pub(crate) debouncer_bump:u8,pub(crate) event_bump:u8,pub(crate) temp_bump:Option<u8>,
 pub(crate) remaining:Vec<Account>,
}
impl EventFrame for Donation {fn event(&self)->EventContext {EventContext{program:self.program,authority:self.event_authority,bump:self.event_bump}}}
impl DonationFrame<Account> for Donation {
 fn body(&mut self)->body::DonationContext<'_> {body::DonationContext{
  tracker:&mut self.epoch_tracker.value,debouncer:&mut self.debouncer.value,whitelist:&self.whitelist.value,
  epoch_bump:self.epoch_bump,debouncer_bump:self.debouncer_bump,mint:self.mint.info,source:self.source.info,
  destination:self.destination.info,from:self.from,token_program:self.token_program,system:self.system}}
 fn exit(&self)->exit::DonationExit<'_,Account> {exit::DonationExit{tracker:self.epoch_tracker.state(),debouncer:self.debouncer.state(),vault:self.destination.interface(),mint:self.mint.interface(),from_token:self.source.interface()}}
}
impl EpochFrame<Account> for Epoch {
 fn body(&mut self)->body::CloseContext<'_> {body::CloseContext{
  signer:self.signer,tracker:&mut self.epoch_tracker.value,debouncer:&mut self.debouncer.value,
  distribution:&mut self.distribution.value,distribution_bump:self.distribution_bump,
  source:self.debouncer_token_account.info,destination:self.distribution_token_account.info,
  authority:self.debouncer.info,mint:self.mint.value,token_program:self.token_program}}
 fn exit(&self)->exit::EpochExit<'_,Account> {exit::EpochExit{
  distribution:self.distribution.state(),distribution_token:self.distribution_token_account.interface(),
  tracker:self.epoch_tracker.state(),debouncer:self.debouncer.state(),debouncer_token:self.debouncer_token_account.interface(),mint:self.mint.interface()}}
}
// Borrow the admitted cached views; these references do not borrow the live
// AccountInfo RefCell data/lamports and do not alter CPI reload behavior.
fn settlement<'a>(source:&'a Loaded<TokenView>,mint:&'a Loaded<MintView>,program:&'a Account,authority:&'a Account,relayer:&'a Account)->body::Settlement<'a> {
 body::Settlement{source:&source.value,mint:&mint.value,program,authority,relayer}
}
impl AfterFrame<Account> for After {
 fn body(&self)->AfterView<'_> {AfterView{body:body::DistributionContext{
  distribution:&self.distribution.value,current_epoch:self.epoch_tracker.value.current_epoch,
  debouncer_mint:self.debouncer.value.mint,
  settlement:settlement(&self.output_distribution_token_account,&self.output_mint,&self.output_mint_program,&self.distribution.info,&self.signer)},
  receipt:&self.swap_receipt.value,remaining:&self.remaining}}
 fn exit(&self)->exit::AfterExit<'_,Account> {exit::AfterExit{distribution:&self.distribution.info,receipt:&self.swap_receipt.info,signer:&self.signer,output_token:self.output_distribution_token_account.interface(),output_mint:self.output_mint.interface()}}
}
impl NonSwapFrame<Account> for NonSwap {
 fn body(&self)->NonSwapView<'_> {NonSwapView{body:body::DistributionContext{
  distribution:&self.distribution.value,current_epoch:self.epoch_tracker.value.current_epoch,
  debouncer_mint:self.debouncer.value.mint,
  settlement:settlement(&self.distribution_token_account,&self.mint,&self.token_program,&self.distribution.info,&self.signer)},remaining:&self.remaining}}
 fn exit(&self)->exit::NonSwapExit<'_,Account> {exit::NonSwapExit{distribution:&self.distribution.info,signer:&self.signer,token:self.distribution_token_account.interface(),mint:self.mint.interface()}}
}
impl JustFrame<Account> for Just {
 fn body(&mut self)->JustView<'_> {JustView{body:body::SwapContext{
  distribution:&self.distribution.value,current_epoch:self.epoch_tracker.value.current_epoch,debouncer_mint:self.debouncer.value.mint,
  input:settlement(&self.distribution_token_account,&self.mint,&self.token_program,&self.distribution.info,&self.signer),
  output:settlement(&self.output_distribution_token_account,&self.output_mint,&self.output_mint_program,&self.distribution.info,&self.signer),jupiter:self.jupiter_program},
  receipt:&mut self.swap_receipt.value,receipt_account:self.swap_receipt.info,receipt_bump:self.receipt_bump,remaining:&self.remaining}}
 fn exit(&self)->exit::JustSwapExit<'_,Account> {exit::JustSwapExit{distribution:self.distribution.state(),receipt:self.swap_receipt.state(),input_token:self.distribution_token_account.interface(),output_token:self.output_distribution_token_account.interface(),mint:self.mint.interface(),output_mint:self.output_mint.interface()}}
}
impl CombinedFrame<Account> for Combined {
 fn body(&self)->CombinedView<'_> {CombinedView{body:body::SwapContext{
  distribution:&self.distribution.value,current_epoch:self.epoch_tracker.value.current_epoch,debouncer_mint:self.debouncer.value.mint,
  input:settlement(&self.distribution_token_account,&self.mint,&self.token_program,&self.distribution.info,&self.signer),
  output:settlement(&self.output_distribution_token_account,&self.output_mint,&self.output_mint_program,&self.distribution.info,&self.signer),jupiter:self.jupiter_program},remaining:&self.remaining}}
 fn exit(&self)->exit::CombinedExit<'_,Account> {exit::CombinedExit{distribution:&self.distribution.info,signer:&self.signer,input_token:self.distribution_token_account.interface(),output_token:self.output_distribution_token_account.interface(),mint:self.mint.interface(),output_mint:self.output_mint.interface()}}
}
impl WhitelistFrame<Account> for WhitelistFrameData {
 fn body(&mut self)->WhitelistView<'_> {WhitelistView{signer:self.signer,state:&mut self.mint_whitelist.value,canonical_bump:self.whitelist_bump}}
 fn exit(&self)->exit::StateAccount<'_,Account,Whitelist>{self.mint_whitelist.state()}
}
