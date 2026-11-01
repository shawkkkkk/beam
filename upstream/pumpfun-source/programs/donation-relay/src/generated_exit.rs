//! Natural generated-exit adapter for the integrated hypothesis; static, uncompiled.
//! The runtime trait contains AccountInfo primitives only, never an application exit.
use crate::{Key, State, Tracker, Debouncer, Distribution, SwapReceipt, Whitelist, codec};
use std::io::Write;
pub(crate) mod bpf_writer { pub use anchor_lang::__private::BpfWriter; }
use bpf_writer::BpfWriter;
use codec::constants::{RELAY, WHITELIST_DISC, TRACKER_DISC, DEBOUNCER_DISC, DISTRIBUTION_DISC, RECEIPT_DISC};

#[path="exit_error.rs"] pub(crate) mod exit_error;
pub use exit_error::Error as ExitError;
pub type X<T> = Result<T, ExitError>;
fn write_error() -> ExitError { exit_error::account_did_not_serialize() }
fn named<T>(r:X<T>, name:&'static str)->X<T> {
    r.map_err(|e| e.with_account_name(name))
}
// Runtime implementation requirements:
// - A=Account is a trusted invocation arena capability preserving aliases.
//   Arena retains original Rc backing throughout. Exact clone/drop overflow and
//   destructor timing remain a normal-resource premise, not claimed equivalent.
// - owner reads the LIVE owner; cached interface owner is a separate input below.
// - data_empty_panicking uses an immutable data borrow; conflicts abort/panic.
// - with_data_mut uses try_borrow_mut_data: conflicts -> actual AccountBorrowFailed;
//   the captured private enum tag is NOT the external ProgramError u64 ABI,
//   and releases the guard on both Ok and Err. No account-name origin at this layer.
// - lamports_panicking/set_lamports_panicking use borrow()/borrow_mut(), not try;
//   conflicts panic, and each call releases its guard before returning.
// - assign_system writes owner; resize_zero tries a mutable data borrow, sets both
//   serialized and visible length to zero, leaves allocation bytes intact, and
//   returns ProgramError on borrow failure. It does not clear the data tail.
// - adequate allocation, valid slices, Rc storage and valid AccountInfo ABI are
//   explicit premises; panic text and full runtime ABI equivalence are not established.
pub trait AccountRuntime {
    type A: Clone;
    fn owner(&self, a:&Self::A)->Key;
    fn data_empty_panicking(&self,a:&Self::A)->bool;
    fn with_data_mut<T,F>(&mut self,a:&Self::A,f:F)->X<T>
        where F:FnOnce(&mut [u8])->X<T>;
    fn lamports_panicking(&self,a:&Self::A)->u64;
    fn set_lamports_panicking(&mut self,a:&Self::A,value:u64);
    fn assign_system(&mut self,a:&Self::A);
    fn resize_zero(&mut self,a:&Self::A)->X<()>;
    fn set_return_data(&mut self,bytes:&[u8]);
}
fn closed<R:AccountRuntime>(rt:&R,a:&R::A)->bool {
    // W283: owner==System && data_is_empty, NOT a zero-lamport test.
    rt.owner(a)==[0;32] && rt.data_empty_panicking(a)
}
pub struct StateAccount<'a,A,T> { pub info:&'a A, pub value:&'a T }
pub struct Interface<'a,A> { pub info:&'a A, pub cached_owner:Key }
fn persist<R:AccountRuntime,F>(rt:&mut R,pid:&Key,a:&R::A,name:&'static str,write:F)->X<()>
where F:FnOnce(&mut [u8])->std::io::Result<()> {
    // W17/18/20 compare the TYPE's declared owner, not current AccountInfo.owner.
    if *pid!=RELAY || closed(rt,a) {return Ok(());}
    named(rt.with_data_mut(a,|data| {
        // Stream the fixed fields after acquiring the mutable borrow, as in
        // original writers17/18/20 and the inline receipt writer in153.
        // Short writes preserve their copied prefix; no temporary Vec or tail wipe.
        write(data).map_err(|error| {drop(error);write_error()})
    }),name)
}
fn interface<R:AccountRuntime>(rt:&mut R,pid:&Key,a:&Interface<R::A>,name:&'static str)->X<()> {
    if a.cached_owner!=*pid || closed(rt,a.info) {return Ok(());}
    // W125 etc: successful branch has no writer call or data store. The SPL
    // interface serialization is zero bytes, but its MUTABLE BORROW is fallible.
    // It must remain after the owner/closed gates and before the next field.
    named(rt.with_data_mut(a.info,|_data|Ok(())),name)
}
fn tracker<R:AccountRuntime>(r:&mut R,p:&Key,a:&StateAccount<R::A,Tracker>)->X<()> {
    persist(r,p,a.info,"epoch_tracker",|data| {
        let mut writer=BpfWriter::new(data);
        writer.write_all(&TRACKER_DISC)?;
        writer.write_all(&[a.value.bump])?;
        writer.write_all(&[match a.value.state {State::Uninitialized=>0,State::Initialized=>1}])?;
        writer.write_all(&a.value.config_id)?;
        writer.write_all(&a.value.mint)?;
        writer.write_all(&a.value.current_epoch.to_le_bytes())
    })
}
fn debouncer<R:AccountRuntime>(r:&mut R,p:&Key,a:&StateAccount<R::A,Debouncer>)->X<()> {
    persist(r,p,a.info,"debouncer",|data| {
        let mut writer=BpfWriter::new(data);
        writer.write_all(&DEBOUNCER_DISC)?;
        writer.write_all(&[a.value.bump])?;
        writer.write_all(&[match a.value.state {State::Uninitialized=>0,State::Initialized=>1}])?;
        writer.write_all(&a.value.config_id)?;
        writer.write_all(&a.value.mint)?;
        writer.write_all(&a.value.total_amount.to_le_bytes())
    })
}
fn distribution<R:AccountRuntime>(r:&mut R,p:&Key,a:&StateAccount<R::A,Distribution>)->X<()> {
    persist(r,p,a.info,"distribution",|data| {
        let mut writer=BpfWriter::new(data);
        writer.write_all(&DISTRIBUTION_DISC)?;
        writer.write_all(&[a.value.bump])?;
        writer.write_all(&[match a.value.state {State::Uninitialized=>0,State::Initialized=>1}])?;
        writer.write_all(&a.value.config_id)?;
        writer.write_all(&a.value.mint)?;
        writer.write_all(&a.value.epoch.to_le_bytes())?;
        writer.write_all(&a.value.total_amount.to_le_bytes())
    })
}
fn receipt<R:AccountRuntime>(r:&mut R,p:&Key,a:&StateAccount<R::A,SwapReceipt>)->X<()> {
    persist(r,p,a.info,"swap_receipt",|data| {
        let mut writer=BpfWriter::new(data);
        writer.write_all(&RECEIPT_DISC)?;
        writer.write_all(&[a.value.bump])?;
        writer.write_all(&a.value.distribution)?;
        writer.write_all(&a.value.output_amount.to_le_bytes())
    })
}
fn close282<R:AccountRuntime>(rt:&mut R,source:R::A,destination:R::A)->X<()> {
    let destination_balance=rt.lamports_panicking(&destination);
    let source_balance=rt.lamports_panicking(&source);
    let total=destination_balance.checked_add(source_balance)
        .expect("framework close lamport overflow"); // panic, not relay6008
    rt.set_lamports_panicking(&destination,total); // guard released here
    rt.set_lamports_panicking(&source,0);          // a new guard, alias-sensitive
    rt.assign_system(&source);
    let result=rt.resize_zero(&source); // may fail AFTER lamport/owner effects
    drop(destination); drop(source);   // W282 releases destination before source
    result
}
fn close_named<R:AccountRuntime>(rt:&mut R,source:&R::A,destination:&R::A,name:&'static str)->X<()> {
    // Callers clone destination first, source second. No new key inequality or
    // owner/closed check: already parsed context; W282 itself has no such gates.
    let destination=destination.clone();
    let source=source.clone();
    named(close282(rt,source,destination),name)
}
pub struct DonationExit<'a,A> {
    pub tracker:StateAccount<'a,A,Tracker>, pub debouncer:StateAccount<'a,A,Debouncer>,
    pub vault:Interface<'a,A>, pub mint:Interface<'a,A>, pub from_token:Interface<'a,A>,
}
fn donation<R:AccountRuntime>(rt:&mut R,pid:&Key,c:&DonationExit<R::A>,last:&'static str)->X<()> {
    tracker(rt,pid,&c.tracker)?;
    debouncer(rt,pid,&c.debouncer)?;
    interface(rt,pid,&c.vault,"debouncer_token_account")?;
    interface(rt,pid,&c.mint,"mint")?;
    interface(rt,pid,&c.from_token,last)
}
pub fn account_exit125<R:AccountRuntime>(r:&mut R,p:&Key,c:&DonationExit<R::A>)->X<()> {
    donation(r,p,c,"from_token_account")
}
pub fn account_exit130<R:AccountRuntime>(r:&mut R,p:&Key,c:&DonationExit<R::A>)->X<()> {
    donation(r,p,c,"temp_wsol_token_account")
}
pub fn account_exit141<R:AccountRuntime>(r:&mut R,p:&Key,c:&DonationExit<R::A>)->X<()> {
    donation(r,p,c,"from_token_account")
}
pub struct EpochExit<'a,A> {
    pub distribution:StateAccount<'a,A,Distribution>, pub distribution_token:Interface<'a,A>,
    pub tracker:StateAccount<'a,A,Tracker>, pub debouncer:StateAccount<'a,A,Debouncer>,
    pub debouncer_token:Interface<'a,A>, pub mint:Interface<'a,A>,
}
pub fn account_exit145<R:AccountRuntime>(r:&mut R,p:&Key,c:&EpochExit<R::A>)->X<()> {
    distribution(r,p,&c.distribution)?;
    interface(r,p,&c.distribution_token,"distribution_token_account")?;
    tracker(r,p,&c.tracker)?;
    debouncer(r,p,&c.debouncer)?;
    interface(r,p,&c.debouncer_token,"debouncer_token_account")?;
    interface(r,p,&c.mint,"mint")
}
pub struct AfterExit<'a,A> {
    pub distribution:&'a A,pub receipt:&'a A,pub signer:&'a A,
    pub output_token:Interface<'a,A>,pub output_mint:Interface<'a,A>,
}
pub fn account_exit147<R:AccountRuntime>(r:&mut R,p:&Key,c:&AfterExit<R::A>)->X<()> {
    close_named(r,c.distribution,c.signer,"distribution")?;
    close_named(r,c.receipt,c.signer,"swap_receipt")?;
    interface(r,p,&c.output_token,"output_distribution_token_account")?;
    interface(r,p,&c.output_mint,"output_mint")
}
pub struct NonSwapExit<'a,A> {
    pub distribution:&'a A,pub signer:&'a A,
    pub token:Interface<'a,A>,pub mint:Interface<'a,A>,
}
pub fn account_exit149<R:AccountRuntime>(r:&mut R,p:&Key,c:&NonSwapExit<R::A>)->X<()> {
    close_named(r,c.distribution,c.signer,"distribution")?;
    interface(r,p,&c.token,"distribution_token_account")?;
    interface(r,p,&c.mint,"mint")
}
pub struct JustSwapExit<'a,A> {
    pub distribution:StateAccount<'a,A,Distribution>,pub receipt:StateAccount<'a,A,SwapReceipt>,
    pub input_token:Interface<'a,A>,pub output_token:Interface<'a,A>,
    pub mint:Interface<'a,A>,pub output_mint:Interface<'a,A>,
}
pub fn account_exit153<R:AccountRuntime>(r:&mut R,p:&Key,c:&JustSwapExit<R::A>)->X<()> {
    distribution(r,p,&c.distribution)?;
    receipt(r,p,&c.receipt)?;
    interface(r,p,&c.input_token,"distribution_token_account")?;
    interface(r,p,&c.output_token,"output_distribution_token_account")?;
    interface(r,p,&c.mint,"mint")?;
    interface(r,p,&c.output_mint,"output_mint")
}
pub struct CombinedExit<'a,A> {
    pub distribution:&'a A,pub signer:&'a A,
    pub input_token:Interface<'a,A>,pub output_token:Interface<'a,A>,
    pub mint:Interface<'a,A>,pub output_mint:Interface<'a,A>,
}
pub fn account_exit155<R:AccountRuntime>(r:&mut R,p:&Key,c:&CombinedExit<R::A>)->X<()> {
    close_named(r,c.distribution,c.signer,"distribution")?;
    interface(r,p,&c.input_token,"distribution_token_account")?;
    interface(r,p,&c.output_token,"output_distribution_token_account")?;
    interface(r,p,&c.mint,"mint")?;
    interface(r,p,&c.output_mint,"output_mint")
}
pub fn account_exit158<R:AccountRuntime>(r:&mut R,p:&Key,c:&StateAccount<R::A,Whitelist>)->X<()> {
    if *p!=RELAY || closed(r,c.info) {return Ok(());}
    named(r.with_data_mut(c.info,|data| {
        // Preserve the shared writer's I/O error construction and drop it
        // before creating the owned framework serialization error.
        let serialize = || -> std::io::Result<()> {
            let mut writer=BpfWriter::new(data);
            writer.write_all(&WHITELIST_DISC)?;
            writer.write_all(&[c.value.bump])?;
            let n=u32::try_from(c.value.mints.len())
                .map_err(|_|std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
            writer.write_all(&n.to_le_bytes())?;
            for key in &c.value.mints {writer.write_all(key)?;}
            Ok(())
        };
        serialize().map_err(|error| {drop(error);write_error()})
    }),"mint_whitelist")
}
// Dispatch owns event, return-data and each explicit account_exit call.
// No second wrapper or generic callback is supplied by this module.
