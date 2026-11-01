//! Natural relay dispatch/wrapper proposal. Uncompiled; actual API bridge is separate.
//! Names/phase edges bind original169,178..189 and190..204. See contract ledger.
use crate::{Account,Key,Event,DonationVariant,codec,handlers as body,runtime::Runtime};
use crate::codec::constants::RELAY;
use crate::generated_exit as exit;
use crate::dispatch_interfaces::*;

#[derive(Debug)]
pub struct SourceOrigin {pub file:String,pub line:u32}
#[derive(Debug)]
pub struct FrameworkFailure {
 pub code:u32,pub account:Option<&'static str>,pub compared_keys:Option<(Key,Key)>,
 pub origin:Option<SourceOrigin>,
 // Original297 constructor on dispatch/event failures has no source origin.
 // Exact error-name/message allocation and heap representation remain a premise.
}
#[derive(Debug)]
pub enum Failure {
 Framework(FrameworkFailure),ProgramNotEnoughAccountKeys,
 Program {error:solana_program::program_error::ProgramError,account:Option<&'static str>},
 Body(crate::Error),Exit(exit::ExitError),Idl(crate::idl::IdlError),
 // An analysis stop, never a relay/Anchor error or a successful instruction.
 Unrecovered(&'static str),
}
pub type D<T> = Result<T,Failure>;
fn framework(code:u32)->Failure {Failure::Framework(FrameworkFailure{code,account:None,compared_keys:None,origin:None})}
fn named(code:u32,compared_keys:Option<(Key,Key)>)->Failure {
 Failure::Framework(FrameworkFailure{code,account:Some("event_authority"),compared_keys,origin:None})
}
const IDL_SELECTOR:[u8;8]=[64,244,188,120,167,233,105,10];
const EVENT_SELECTOR:[u8;8]=[228,69,165,46,81,203,154,29];

// No payload parsing/logging, exact-account-count check, return-data clear or
// generated exit occurs on this branch. The first real AccountInfo must sign.
fn event_recursion<R:DispatchRuntime>(rt:&mut R,program:&Key,accounts:&[Account])->D<()> {
 let authority=accounts.first().ok_or(Failure::ProgramNotEnoughAccountKeys)?;
 if !authority.signer {return Err(named(2002,None));}
 let (expected,_bump)=rt.find_program_address(&[b"__event_authority"],program);
 if authority.key!=expected {return Err(named(2006,Some((authority.key,expected))));}
 Ok(())
}

fn emit_and_return<R:DispatchRuntime>(rt:&mut R,event:&Event,context:EventContext)->D<()> {
 // First serialization for log. Dynamic serialization failure unwraps/panics;
 // allocation/panic object identity is not established by this natural source.
 let (disc,bytes)=codec::event_bytes(event).unwrap();
 let mut logged=disc.to_vec();logged.extend(bytes);
 rt.log_event_data(&logged);
 // SECOND serialization for self-CPI, after the log. Do not reuse logged bytes.
 let (disc,bytes)=codec::event_bytes(event).unwrap();
 let mut data=EVENT_SELECTOR.to_vec();data.extend(disc);data.extend(bytes);
 let ix=crate::Cpi{program:RELAY,metas:vec![crate::Meta{key:context.authority.key,writable:false,signer:true}],data};
 // `program` is retained from the admitted context but is not an extra meta or
 // an invented executable/address constraint. Original uses fixed relay ID.
 let _program=context.program;
 rt.invoke_signed(ix,&[context.authority],&vec![b"__event_authority".to_vec(),vec![context.bump]])
   .map_err(Failure::Body)?;
 // THIRD serialization: bare enum-tagged return payload, no event discriminator.
 // Set-return-data is a syscall with no ordinary Result; generated exit follows.
 let (_,returned)=codec::event_bytes(event).unwrap();
 exit::AccountRuntime::set_return_data(rt,&returned);
 Ok(())
}

/// Logical Result layer of169. The surrounding loader ABI, rich error reporting
/// and lowering are implemented by the candidate API bridge. Input accounts come
/// from the real Solana entrypoint loader; historical binary identity is not claimed.
pub fn dispatch<R:crate::idl_runtime::IdlRuntime,C:Admission<R>>(
 rt:&mut R,contexts:&mut C,program:Key,accounts:&[Account],data:&[u8]
)->D<()> {
 // Declared-program check precedes even the input-length check.
 if program!=RELAY {return Err(framework(4100));}
 if data.len()<8 {return Err(framework(101));}
 let selector:[u8;8]=data[..8].try_into().unwrap();
 let raw_args=&data[8..];
 // Original169 routes to eleven distinct business wrappers178..188.
 match selector {
  _=>Err(framework(101)),
 }
}

// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


// Instruction-specific wrapper: log, decode, admission, body, event/return, exit.


/// Report propagated ordinary errors once, as169 calls286 before290 lowering.
/// This is not the raw Solana u64 ABI: rich error encoding/lowering still needs
/// the actual framework adapter. Unrecovered is never reported as a real error.
pub fn process<R:crate::idl_runtime::IdlRuntime,C:Admission<R>>(rt:&mut R,c:&mut C,program:Key,accounts:&[Account],data:&[u8])->D<()> {
 let result=dispatch(rt,c,program,accounts,data);
 if let Err(error)=&result {if !matches!(error,Failure::Unrecovered(_)) {rt.report_error(error);}}
 result
}

