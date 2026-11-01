//! Genuine SDK loader and return ABI. This proposal has not been built or run.
use solana_program::{account_info::AccountInfo,entrypoint::ProgramResult,pubkey::Pubkey};

pub fn process_instruction(program:&Pubkey,accounts:&[AccountInfo<'_>],data:&[u8])->ProgramResult {
    let (mut runtime,handles)=crate::solana_runtime::SolanaRuntime::from_loader(accounts);
    let mut contexts=crate::admission::Contexts;
    // The dispatch reports a returned error exactly once before this conversion.
    crate::dispatch::process(&mut runtime,&mut contexts,program.to_bytes(),&handles,data)
        .map_err(crate::error_report::lower)
}
