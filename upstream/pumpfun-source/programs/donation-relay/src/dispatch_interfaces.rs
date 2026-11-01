//! Static integration contracts, not runnable context/exit substitutes.
//! Implement with the separately sealed parser and exit source, real shared handles,
//! complete field mappings, and propagated framework errors. No default methods.
use crate::{Account,Key,Tracker,Debouncer,Distribution,SwapReceipt,Whitelist,DonationArgs,DonationVariant,Share};
use crate::{handlers as body,generated_exit as exit};
use crate::runtime::Runtime;
use crate::dispatch::{D,Failure};

#[derive(Clone,Copy)]
pub struct EventContext {pub program:Account,pub authority:Account,pub bump:u8}
pub trait EventFrame {fn event(&self)->EventContext;}
pub trait DonationFrame<A>:EventFrame {
 fn body(&mut self)->body::DonationContext<'_>;
 fn exit(&self)->exit::DonationExit<'_,A>;
}
pub trait EpochFrame<A>:EventFrame {
 fn body(&mut self)->body::CloseContext<'_>;
 fn exit(&self)->exit::EpochExit<'_,A>;
}
pub struct AfterView<'a> {pub body:body::DistributionContext<'a>,pub receipt:&'a SwapReceipt,pub remaining:&'a [Account]}
pub trait AfterFrame<A>:EventFrame {fn body(&self)->AfterView<'_>;fn exit(&self)->exit::AfterExit<'_,A>;}
pub struct NonSwapView<'a> {pub body:body::DistributionContext<'a>,pub remaining:&'a [Account]}
pub trait NonSwapFrame<A>:EventFrame {fn body(&self)->NonSwapView<'_>;fn exit(&self)->exit::NonSwapExit<'_,A>;}
pub struct JustView<'a> {
 pub body:body::SwapContext<'a>,pub receipt:&'a mut SwapReceipt,pub receipt_account:Account,
 pub receipt_bump:u8,pub remaining:&'a [Account],
}
pub trait JustFrame<A>:EventFrame {fn body(&mut self)->JustView<'_>;fn exit(&self)->exit::JustSwapExit<'_,A>;}
pub struct CombinedView<'a> {pub body:body::SwapContext<'a>,pub remaining:&'a [Account]}
pub trait CombinedFrame<A>:EventFrame {fn body(&self)->CombinedView<'_>;fn exit(&self)->exit::CombinedExit<'_,A>;}
pub struct WhitelistView<'a> {pub signer:Account,pub state:&'a mut Whitelist,pub canonical_bump:u8}
pub trait WhitelistFrame<A> {fn body(&mut self)->WhitelistView<'_>;fn exit(&self)->exit::StateAccount<'_,A,Whitelist>;}

/// These methods must call the actual ordered parser implementations. They may
/// not manufacture states, skip initialization, preflight all account counts,
/// accept arbitrary caller bumps, or erase framework errors/compared values.
/// `raw_args` is the ENTIRE suffix after the selector, including trailing bytes.
/// Returned views share live state and aliases; exit sees mutations made by body.
pub trait Admission<R:Runtime+exit::AccountRuntime> {
 type Donation:DonationFrame<R::A>;
 type Epoch:EpochFrame<R::A>;
 type After:AfterFrame<R::A>;
 type NonSwap:NonSwapFrame<R::A>;
 type Just:JustFrame<R::A>;
 type Combined:CombinedFrame<R::A>;
 type Whitelist:WhitelistFrame<R::A>;
 fn donation(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],kind:DonationVariant,args:&DonationArgs)->D<Self::Donation>;
 fn epoch(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],config:Key,epoch:u128)->D<Self::Epoch>;
 fn after(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],shares:&[Share])->D<Self::After>;
 fn non_swap(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],shares:&[Share])->D<Self::NonSwap>;
 fn just_swap(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],swap_data:&[u8])->D<Self::Just>;
 fn combined(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],swap_data:&[u8],shares:&[Share])->D<Self::Combined>;
 fn whitelist(&mut self,rt:&mut R,program:&Key,accounts:&[Account],raw_args:&[u8],mints:&[Key])->D<Self::Whitelist>;
}

/// Low-level effects only. Application admission and exit code are separate.
pub trait DispatchRuntime:Runtime+exit::AccountRuntime {
 fn log_instruction(&mut self,text:&str); // actual sol_log_, not log_data
 fn log_event_data(&mut self,data:&[u8]); // one slice, actual sol_log_data
 // The original341 syscall failure panics; never return a fabricated address.
 fn find_program_address(&mut self,seeds:&[&[u8]],program:&Key)->(Key,u8);
 // Original169 calls286 only on Err, then290 lowers it to ProgramError. This
 // adapter MUST preserve all rich metadata. No error swallowing or success.
 fn report_error(&mut self,error:&Failure);
}
