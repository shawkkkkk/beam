//! Genuine low-level primitive boundary for the uncompiled IDL application.
use std::cell::{Ref, RefMut};
use crate::{Account, Key, Cpi};
use crate::dispatch_interfaces::DispatchRuntime;
use crate::idl::I;

pub struct Rent { pub lamports_per_byte_year:u64, pub exemption_threshold:f64, pub burn_percent:u8 }
impl Rent {
    pub fn minimum_balance(&self, data_len:usize)->u64 {
        // W324/325 checked u64 overhead/multiply, f64 multiply, saturating cast.
        // Exact compiled soft-float bodies/NaN behavior are bounded runtime premises.
        let bytes=(data_len as u64).checked_add(128).expect("Rent length overflow");
        let annual=bytes.checked_mul(self.lamports_per_byte_year).expect("Rent multiplication overflow");
        (annual as f64 * self.exemption_threshold) as u64
    }
}
// No default implementations. Account.handle resolves the SAME live Rc-backed
// AccountInfo used by body/context/generated exits. Duplicate metas keep aliasing;
// no copied data, forged signer bit, substituted owner or guessed PDA.
pub trait IdlRuntime:DispatchRuntime {
    fn idl_owner(&self,a:Account)->Key;
    fn idl_executable(&self,a:Account)->bool;
    // W331 / mutable counterpart: try_borrow[_mut], AccountBorrowFailed on conflict,
    // guards share borrow state; releasing Ref/RefMut changes that same state.
    fn idl_data(&self,a:Account)->I<Ref<'_,[u8]>>;
    fn idl_data_mut(&self,a:Account)->I<RefMut<'_,[u8]>>;
    // W326 and282 use PANICKING lamport borrow/borrow_mut and release each guard.
    fn idl_lamports_panicking(&self,a:Account)->u64;
    fn idl_set_lamports_panicking(&mut self,a:Account,value:u64);
    fn idl_assign_system(&mut self,a:Account);
    // W332 checks new_len.saturating_sub(original_serialized_len)<=10240 after
    // acquiring mutable data, then updates serialized/visible length; growth
    // is zero-filled. Conflict ->ProgramFailure::AccountBorrowFailed;
    // excessive increase ->ProgramFailure::InvalidRealloc.
    fn idl_realloc_zero_filled(&mut self,a:Account,new_len:usize)->I<()>;
    // Real Rent::get syscall through the pinned candidate API; retain ProgramError.
    fn idl_rent(&self)->I<Rent>;
    // W340: reject seed>32 or owner suffix ProgramDerivedAddress; SHA256 of
    // base||seed||owner. Preserve failure so caller's unwrap panics as captured.
    fn idl_create_with_seed(&self,base:&Key,seed:&[u8],owner:&Key)->Result<Key,solana_program::program_error::ProgramError>;
    // Actual Solana invoke_signed, real System program, unchanged metas/data,
    // AccountInfo alias/borrow preflight W300 and runtime signer derivation.
    // Outer slice=signer groups, inner slice=seeds; empty groups means no PDA.
    // Preflight conflict ->AccountBorrowFailed. The real SDK preserves ordinary
    // ProgramError values; its current external u64 ABI is not a historical enum claim.
    fn idl_invoke_signed(&mut self,ix:Cpi,infos:&[Account],signers:&[Vec<Vec<u8>>])->I<()>;
}
