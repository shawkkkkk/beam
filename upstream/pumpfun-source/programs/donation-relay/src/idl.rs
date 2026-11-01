//! Captured relay IDL170 application reconstruction; static, UNCOMPILED.
//! Requires the real aliased AccountInfo adapter in idl_runtime.rs. No IDL body,
//! parser, serialization or generated exit is delegated to a success callback.
use crate::{Account, Key, Cpi, Meta};
use crate::codec::constants::RELAY;
use crate::dispatch_interfaces::DispatchRuntime;
use crate::idl_runtime::IdlRuntime;

pub const DISC: [u8; 8] = [24, 70, 98, 191, 58, 144, 123, 158];
const SYSTEM: Key = [0; 32];
const HEADER: usize = 44;
// Original error-origin data is explicitly reconstructed in the owned3002
// constructor. This does not attribute this new source file to that location.

pub type ProgramFailure = solana_program::program_error::ProgramError;
pub type IdlError = crate::generated_exit::exit_error::Error;
use crate::generated_exit::exit_error::ErrorCode;
pub type I<T> = Result<T, IdlError>;
fn code(n: ErrorCode) -> IdlError { n.into() }
pub fn program_error(error: ProgramFailure) -> IdlError { error.into() }
fn named<T>(r: I<T>, name: &'static str) -> I<T> {
    r.map_err(|error| error.with_account_name(name))
}
fn require(ok: bool, n: ErrorCode, name: &'static str) -> I<()> {
    named(if ok { Ok(()) } else { Err(code(n)) }, name)
}
fn require_keys(left: Key, right: Key, n: ErrorCode, name: &'static str) -> I<()> {
    if left == right { return Ok(()); }
    let error = code(n).with_pubkeys((
        solana_program::pubkey::Pubkey::new_from_array(left),
        solana_program::pubkey::Pubkey::new_from_array(right),
    ));
    named(Err(error), name)
}

#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug)]
pub struct Header { pub authority: solana_program::pubkey::Pubkey, pub data_len: u32 }
struct Typed { info: Account, value: Header }
enum Instruction {
    Create(u64), CreateBuffer, Write(Vec<u8>), SetBuffer,
    SetAuthority(Key), Close, Resize(u64),
}
// W170 enum decoder consumes a prefix, not the entire suffix. Allocation/OOM
// details of W77's Vec reader remain a stated framework premise.
fn decode(data: &[u8]) -> I<Instruction> {
    let (&tag, tail) = data.split_first().ok_or_else(||code(ErrorCode::InstructionDidNotDeserialize))?;
    let u64_arg = || -> I<u64> {
        Ok(u64::from_le_bytes(tail.get(..8).ok_or_else(||code(ErrorCode::InstructionDidNotDeserialize))?.try_into().unwrap()))
    };
    Ok(match tag {
        0 => Instruction::Create(u64_arg()?),
        1 => Instruction::CreateBuffer,
        2 => {
            let len = u32::from_le_bytes(tail.get(..4).ok_or_else(||code(ErrorCode::InstructionDidNotDeserialize))?.try_into().unwrap()) as usize;
            let bytes = tail.get(4..).and_then(|s|s.get(..len)).ok_or_else(||code(ErrorCode::InstructionDidNotDeserialize))?;
            Instruction::Write(bytes.to_vec())
        }
        3 => Instruction::SetBuffer,
        4 => Instruction::SetAuthority(tail.get(..32).ok_or_else(||code(ErrorCode::InstructionDidNotDeserialize))?.try_into().unwrap()),
        5 => Instruction::Close,
        6 => Instruction::Resize(u64_arg()?),
        _ => return Err(code(ErrorCode::InstructionDidNotDeserialize)),
    })
}
// W218: skip eight bytes even unchecked; <8 is a slice panic, 8..43 ->3003.
// Neither checked nor unchecked decode validates data_len against capacity.
fn unchecked_header(data: &mut &[u8]) -> I<Header> {
    let mut tail = &data[8..];
    <Header as borsh::BorshDeserialize>::deserialize(&mut tail).map_err(|error| {
        let owned = crate::generated_exit::exit_error::account_did_not_deserialize();
        drop(error);
        owned
    })
}
fn checked_header(data: &mut &[u8]) -> I<Header> {
    if data.len() < 8 {
        return Err(crate::generated_exit::exit_error::account_discriminator_not_found());
    }
    if data[..8] != DISC {
        let owned = crate::generated_exit::exit_error::account_discriminator_mismatch()
            .with_account_name("IdlAccount");
        return Err(owned); // enclosing generated parser replaces and drops the owned account name.
    }
    unchecked_header(data)
}
struct Cursor<'a> { accounts: &'a [Account], at: usize }
impl<'a> Cursor<'a> {
    fn new(accounts: &'a [Account]) -> Self { Self { accounts, at: 0 } }
    fn raw(&mut self) -> I<Account> {
        let a = self.accounts.get(self.at).copied().ok_or_else(||code(ErrorCode::AccountNotEnoughKeys))?;
        self.at += 1; Ok(a)
    }
    fn raw_named(&mut self, name: &'static str) -> I<Account> { named(self.raw(), name) }
    fn signer(&mut self, name: &'static str) -> I<Account> {
        let a = self.raw_named(name)?;
        require(a.signer, ErrorCode::AccountNotSigner, name)?; Ok(a) // W80 ->W280, not constraint2002.
    }
    fn system<R: IdlRuntime>(&mut self, rt: &R) -> I<Account> {
        let a = self.raw_named("system_program")?;
        require_keys(a.key, SYSTEM, ErrorCode::InvalidProgramId, "system_program")?;
        require(rt.idl_executable(a), ErrorCode::InvalidProgramExecutable, "system_program")?; Ok(a)
    }
    fn typed<R: IdlRuntime>(&mut self, rt: &R, name: &'static str) -> I<Typed> {
        let a = self.raw_named(name)?;
        named(typed(rt, a, true), name)
    }
}
fn typed<R: IdlRuntime>(rt: &R, info: Account, checked: bool) -> I<Typed> {
    let owner = rt.idl_owner(info);
    if owner == SYSTEM && rt.idl_lamports_panicking(info) == 0 { return Err(code(ErrorCode::AccountNotInitialized)); }
    if owner != RELAY {
        return Err(code(ErrorCode::AccountOwnedByWrongProgram).with_pubkeys((
            solana_program::pubkey::Pubkey::new_from_array(owner),
            solana_program::pubkey::Pubkey::new_from_array(RELAY),
        )));
    }
    let bytes = rt.idl_data(info)?;
    let value = if checked { checked_header(&mut &bytes[..])? } else { unchecked_header(&mut &bytes[..])? };
    drop(bytes); Ok(Typed { info, value })
}
fn writable(a: Account, name: &'static str) -> I<()> { require(a.writable, ErrorCode::ConstraintMut, name) }
fn authorized(idl: &Typed, authority: Account, name: &'static str) -> I<()> {
    require_keys(idl.value.authority.to_bytes(), authority.key, ErrorCode::ConstraintHasOne, name)
}
fn active_authority(a: Account) -> I<()> { require(a.key != SYSTEM, ErrorCode::ConstraintRaw, "authority") }
// W213: typed idl, signer, then idl mut/has_one, then non-erased authority.
fn admit_write<R: IdlRuntime>(rt: &R, c: &mut Cursor<'_>) -> I<Typed> {
    let idl = c.typed(rt, "idl")?;
    let authority = c.signer("authority")?;
    writable(idl.info, "idl")?; authorized(&idl, authority, "idl")?;
    active_authority(authority)?; Ok(idl)
}
// W211: System program typing precedes all derived constraints. Authority is
// writable here (payer), unlike Write/SetAuthority/SetBuffer/Close.
fn admit_resize<R: IdlRuntime>(rt: &R, c: &mut Cursor<'_>) -> I<(Typed, Account, Account)> {
    let idl = c.typed(rt, "idl")?;
    let authority = c.signer("authority")?;
    let system = c.system(rt)?;
    writable(idl.info, "idl")?; authorized(&idl, authority, "idl")?;
    writable(authority, "authority")?; active_authority(authority)?;
    Ok((idl, authority, system))
}
// W207: buffer's raw constraint compares BOTH STORED authorities; only then
// does the destination's has_one compare against the actual Signer.
fn admit_set_buffer<R: IdlRuntime>(rt: &R, c: &mut Cursor<'_>) -> I<(Typed, Typed)> {
    let buffer = c.typed(rt, "buffer")?;
    let idl = c.typed(rt, "idl")?;
    let authority = c.signer("authority")?;
    writable(buffer.info, "buffer")?;
    require(buffer.value.authority == idl.value.authority, ErrorCode::ConstraintRaw, "buffer")?;
    writable(idl.info, "idl")?; authorized(&idl, authority, "idl")?;
    active_authority(authority)?; Ok((buffer, idl))
}
// W205: generated close-key inequality precedes authority raw constraint and
// destination mutability. No extra authority writable or destination owner test.
fn admit_close<R: IdlRuntime>(rt: &R, c: &mut Cursor<'_>) -> I<(Typed, Account)> {
    let account = c.typed(rt, "account")?;
    let authority = c.signer("authority")?;
    let destination = c.raw_named("sol_destination")?;
    writable(account.info, "account")?; authorized(&account, authority, "account")?;
    require(account.info.key != destination.key, ErrorCode::ConstraintClose, "account")?;
    active_authority(authority)?; writable(destination, "sol_destination")?;
    Ok((account, destination))
}
fn admit_buffer<R: IdlRuntime>(rt: &R, c: &mut Cursor<'_>) -> I<(Typed, Account)> {
    let info = c.raw()?; // W209 initial missing buffer is unnamed3005.
    let authority = c.signer("authority")?;
    let rent = rt.idl_rent()?;
    let data = rt.idl_data(info)?; // initial borrow errors also unnamed.
    if data[..8] != [0; 8] { return named(Err(code(ErrorCode::ConstraintZero)), "buffer"); }
    // Original holds the first immutable guard through the unchecked typed load.
    let buffer = named(typed(rt, info, false), "buffer")?;
    drop(data);
    writable(info, "buffer")?;
    let balance = rt.idl_lamports_panicking(info);
    let size = rt.idl_data(info)?.len();
    require(balance >= rent.minimum_balance(size), ErrorCode::ConstraintRentExempt, "buffer")?;
    active_authority(authority)?;
    Ok((buffer, authority))
}
fn admit_create<R: IdlRuntime>(rt: &mut R, c: &mut Cursor<'_>, program: &Key) -> I<(Account, Account, Account, Account, Account)> {
    let from = c.raw_named("from")?;
    let to = c.raw_named("to")?;
    let base = c.raw_named("base")?;
    let system = c.system(rt)?;
    let executable = c.raw_named("program")?;
    require(from.signer, ErrorCode::ConstraintSigner, "from")?;
    writable(to, "to")?;
    let (expected, _bump) = rt.find_program_address(&[], program);
    require_keys(base.key, expected, ErrorCode::ConstraintSeeds, "base")?;
    require(rt.idl_executable(executable), ErrorCode::ConstraintExecutable, "program")?;
    Ok((from, to, base, system, executable))
}

// W281/19/216 stream each field. A short account has its available prefix
// overwritten before3004; no upfront capacity check or trailing-byte erasure.
fn write_header(data: &mut [u8], h: Header) -> I<()> {
    use std::io::Write;
    let serialize = || -> std::io::Result<()> {
        let mut writer = crate::generated_exit::bpf_writer::BpfWriter::new(data);
        writer.write_all(&DISC)?;
        writer.write_all(h.authority.as_ref())?;
        writer.write_all(&h.data_len.to_le_bytes())
    };
    serialize().map_err(|error| {
        drop(error);
        crate::generated_exit::exit_error::account_did_not_serialize()
    })
}
fn exit_header<R: IdlRuntime>(rt: &R, program: &Key, idl: &Typed, name: &'static str) -> I<()> {
    if *program != RELAY { return Ok(()); } // TYPE owner comparison, not live owner.
    if rt.idl_owner(idl.info) == SYSTEM && rt.idl_data(idl.info).unwrap().is_empty() { return Ok(()); }
    named((|| { let mut bytes = rt.idl_data_mut(idl.info)?; write_header(&mut bytes, idl.value.clone()) })(), name)
}
fn create<R: IdlRuntime>(rt: &mut R, program: &Key, from: Account, to: Account,
    base_info: Account, system: Account, executable: Account, requested_payload: u64) -> I<()> {
    rt.log_instruction("Instruction: IdlCreateAccount");
    if *program != executable.key { return Err(code(ErrorCode::IdlInstructionInvalidProgram)); }
    let (base, bump) = rt.find_program_address(&[], program); // recomputed by171.
    let address = rt.idl_create_with_seed(&base, b"anchor:idl", &executable.key).unwrap();
    let requested = requested_payload.checked_add(44).expect("IDL create space overflow");
    let rent = rt.idl_rent()?;
    let space = requested.min(10_000);
    let lamports = rent.minimum_balance(space as usize);
    // W320 ->W307: System bincode tag3, base32, seed length u64, seed bytes,
    // lamports u64, space u64, owner32. `to` is NOT pre-compared to address;
    // the real System CPI matches the derived writable meta to supplied infos.
    let mut data = 3u32.to_le_bytes().to_vec();
    data.extend_from_slice(&base); data.extend_from_slice(&10u64.to_le_bytes());
    data.extend_from_slice(b"anchor:idl"); data.extend_from_slice(&lamports.to_le_bytes());
    data.extend_from_slice(&space.to_le_bytes()); data.extend_from_slice(&executable.key);
    let ix = Cpi { program: SYSTEM, metas: vec![
        Meta {key:from.key, writable:true, signer:true},
        Meta {key:address, writable:true, signer:false},
        Meta {key:base, writable:false, signer:true}], data };
    rt.idl_invoke_signed(ix, &[from,to,base_info,system], &[vec![vec![bump]]])?;
    let bytes = rt.idl_data(to)?;
    let mut h = unchecked_header(&mut &bytes[..])?; drop(bytes);
    h.authority = solana_program::pubkey::Pubkey::new_from_array(from.key); // preserve decoded data_len; genuine new account is zeroed.
    let mut bytes = rt.idl_data_mut(to)?;
    write_header(&mut bytes, h) // W216 directly, no extra typed owner/closed gate.
}
fn resize<R: IdlRuntime>(rt: &mut R, idl: &Typed, authority: Account, _system: Account, requested_total: u64) -> I<()> {
    rt.log_instruction("Instruction: IdlResizeAccount");
    if idl.value.data_len != 0 { return Err(code(ErrorCode::IdlAccountNotEmpty)); }
    let current = rt.idl_data(idl.info).unwrap().len();
    let second = rt.idl_data(idl.info).unwrap().len();
    let delta = requested_total.checked_sub(second as u64)
        .expect("data_len should always be >= the current account space").min(10_000);
    let new_len = current.checked_add(delta as usize).expect("IDL resize space overflow");
    if new_len <= rt.idl_data(idl.info).unwrap().len() { return Ok(()); }
    let rent = rt.idl_rent()?;
    let minimum = rent.minimum_balance(new_len);
    let balance = rt.idl_lamports_panicking(idl.info);
    let amount = minimum.checked_sub(balance).expect("IDL resize rent subtraction overflow");
    let mut data = 2u32.to_le_bytes().to_vec(); data.extend_from_slice(&amount.to_le_bytes());
    let ix = Cpi {program:SYSTEM, metas:vec![Meta {key:authority.key,writable:true,signer:true},
        Meta {key:idl.info.key,writable:true,signer:false}], data};
    rt.idl_invoke_signed(ix, &[authority,idl.info], &[])?; // also when amount==0.
    rt.idl_realloc_zero_filled(idl.info, new_len) // W332 always zeros growth.
}
fn write<R: IdlRuntime>(rt: &mut R, idl: &mut Typed, input: &[u8]) -> I<()> {
    rt.log_instruction("Instruction: IdlWrite");
    let old = idl.value.data_len as usize;
    let end = old.checked_add(input.len()).expect("IDL write usize overflow");
    let n = u32::try_from(input.len()).unwrap();
    idl.value.data_len = idl.value.data_len.checked_add(n).expect("IDL write u32 overflow");
    let mut data = rt.idl_data_mut(idl.info).unwrap(); // panic, not propagated borrow error.
    data[HEADER..][old..end].copy_from_slice(input);
    Ok(())
}
fn set_buffer<R: IdlRuntime>(rt: &mut R, buffer: &Typed, idl: &mut Typed) -> I<()> {
    rt.log_instruction("Instruction: IdlSetBuffer");
    idl.value.data_len = buffer.value.data_len; // BEFORE both borrows and capacity checks.
    let mut dst_data = rt.idl_data_mut(idl.info).unwrap();
    let dst = &mut dst_data[HEADER..];
    let src_data = rt.idl_data(buffer.info).unwrap();
    let src = &src_data[HEADER..][..buffer.value.data_len as usize];
    let result = if dst.len() < src.len() {
        Err(code(ErrorCode::RequireGteViolated).with_values((dst.len() as u64, src.len() as u64)))
    } else { dst[..src.len()].copy_from_slice(src); Ok(()) };
    drop(src_data); drop(dst_data); result
}
fn close<R: IdlRuntime>(rt: &mut R, source: Account, destination: Account) -> I<()> {
    // W173 is only this log; W206 then invokes282. Real adapter pins/releases
    // aliased Rc handles as destination first, source second; no byte cloning.
    rt.log_instruction("Instruction: IdlCloseAccount");
    let destination_balance = rt.idl_lamports_panicking(destination);
    let source_balance = rt.idl_lamports_panicking(source);
    let total = destination_balance.checked_add(source_balance).expect("IDL close lamport overflow");
    rt.idl_set_lamports_panicking(destination, total);
    rt.idl_set_lamports_panicking(source, 0);
    rt.idl_assign_system(source);
    named(rt.idl_realloc_zero_filled(source, 0), "account")
}

pub fn dispatch170<R: IdlRuntime>(rt: &mut R, program: &Key, accounts: &[Account], raw_args: &[u8]) -> I<()> {
    let instruction = decode(raw_args)?;
    let mut c = Cursor::new(accounts); // extra accounts are not rejected.
    match instruction {
        Instruction::Create(size) => {
            let (from,to,base,system,executable) = admit_create(rt,&mut c,program)?;
            create(rt,program,from,to,base,system,executable,size)
        }
        Instruction::CreateBuffer => {
            let (mut buffer,authority) = admit_buffer(rt,&mut c)?;
            rt.log_instruction("Instruction: IdlCreateBuffer");
            buffer.value.authority = solana_program::pubkey::Pubkey::new_from_array(authority.key); // no forced data_len reset.
            exit_header(rt,program,&buffer,"buffer")
        }
        Instruction::Write(bytes) => {
            let mut idl = admit_write(rt,&mut c)?; write(rt,&mut idl,&bytes)?;
            exit_header(rt,program,&idl,"idl")
        }
        Instruction::SetBuffer => {
            let (buffer,mut idl) = admit_set_buffer(rt,&mut c)?; set_buffer(rt,&buffer,&mut idl)?;
            exit_header(rt,program,&buffer,"buffer")?;
            exit_header(rt,program,&idl,"idl") // W208 persists buffer BEFORE idl.
        }
        Instruction::SetAuthority(key) => {
            let mut idl = admit_write(rt,&mut c)?;
            rt.log_instruction("Instruction: IdlSetAuthority"); idl.value.authority = solana_program::pubkey::Pubkey::new_from_array(key);
            exit_header(rt,program,&idl,"idl") // zero key is permitted (erases authority).
        }
        Instruction::Close => {
            let (account,destination) = admit_close(rt,&mut c)?; close(rt,account.info,destination)
        }
        Instruction::Resize(size) => {
            let (idl,authority,system) = admit_resize(rt,&mut c)?; resize(rt,&idl,authority,system,size)?;
            exit_header(rt,program,&idl,"idl")
        }
    }
    // No event/log_data/self-CPI event/return-data serialization in170.
}
