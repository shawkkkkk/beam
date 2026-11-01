//! Observable rich-error adapter, with explicit incomplete diagnostic provenance.
//! The message-length failure carries explicitly reconstructed diagnostic data.
//! This is not attribution of the historical Rust source layout.
//! Body errors lacking origin/compared metadata remain a documented fidelity gap.
use solana_program::{program_error::ProgramError,pubkey::Pubkey,log::sol_log};
use crate::{Error,Key,dispatch::Failure};
#[path="error_catalog.rs"] mod catalog;
fn keys(pair:Option<(Key,Key)>) {
    if let Some((left,right))=pair {
        sol_log("Left:");Pubkey::new_from_array(left).log();
        sol_log("Right:");Pubkey::new_from_array(right).log();
    }
}
fn values(pair:Option<(u64,u64)>) {
    if let Some((left,right))=pair {
        sol_log(&format!("Left: {}",left));sol_log(&format!("Right: {}",right));
    }
}
fn framework(code:u32,account:Option<&str>,origin:Option<(&str,u32)>,compared:Option<(Key,Key)>) {
    let (name,message)=catalog::catalog(code);
    // The captured union has a single origin. Decorating with an account name
    // replaces Source, rather than logging both. Account has precedence here.
    if let Some(account)=account {
        sol_log(&format!("AnchorError caused by account: {}. Error Code: {}. Error Number: {}. Error Message: {}.",account,name,code,message));
    } else if let Some((file,line))=origin {
        sol_log(&format!("AnchorError thrown in {}:{}. Error Code: {}. Error Number: {}. Error Message: {}.",file,line,name,code,message));
    } else {
        sol_log(&format!("AnchorError occurred. Error Code: {}. Error Number: {}. Error Message: {}.",name,code,message));
    }
    keys(compared);
}
fn program(error:&ProgramError,account:Option<&str>) {
    let number=u64::from(error.clone());
    if let Some(account)=account {
        sol_log(&format!("ProgramError caused by account: {}. Error Code: {:?}. Error Number: {}. Error Message: {}.",account,error,number,error));
    } else {
        sol_log(&format!("ProgramError occurred. Error Code: {:?}. Error Number: {}. Error Message: {}.",error,number,error));
    }
}
fn body(error:&Error) {
    match error {
        Error::Code(code)=>framework(*code,None,None,None),
        Error::Owned(error)=>error.log(),
        Error::IncorrectProgramId=>program(&ProgramError::IncorrectProgramId,None),
        Error::Program(error)=>program(error,None),
        // An analysis boundary is not converted to success or an invented error.
        Error::Incomplete(reason)=>panic!("unimplemented source boundary: {}",reason),
    }
}
pub fn report(error:&Failure) {
    match error {
        Failure::Framework(e)=>framework(e.code,e.account,e.origin.as_ref().map(|o|(o.file.as_str(),o.line)),e.compared_keys),
        Failure::ProgramNotEnoughAccountKeys=>program(&ProgramError::NotEnoughAccountKeys,None),
        Failure::Program{error,account}=>program(error,*account),
        Failure::Body(e)=>body(e),
        Failure::Exit(e)=>e.log(),
        Failure::Idl(error)=>error.log(),
        Failure::Unrecovered(reason)=>panic!("unimplemented source boundary: {}",reason),
    }
}
/// SDK ProgramError is retained intact through reporting and lowered by the
/// real entrypoint macro. Private historical enum tags are never returned raw.
pub fn lower(error:Failure)->ProgramError {
    match error {
        Failure::Framework(e)=>ProgramError::Custom(e.code),
        Failure::ProgramNotEnoughAccountKeys=>ProgramError::NotEnoughAccountKeys,
        Failure::Program{error,..}=>error,
        Failure::Body(e)=>match e {
            Error::Code(code)=>ProgramError::Custom(code),
            Error::Owned(error)=>error.into(),
            Error::IncorrectProgramId=>ProgramError::IncorrectProgramId,
            Error::Program(error)=>error,
            Error::Incomplete(reason)=>panic!("unimplemented source boundary: {}",reason),
        },
        Failure::Exit(e)=>e.into(),
        Failure::Idl(error)=>error.into(),
        Failure::Unrecovered(reason)=>panic!("unimplemented source boundary: {}",reason),
    }
}
