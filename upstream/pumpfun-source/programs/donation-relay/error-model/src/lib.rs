//! Owned error representation for generated account exits.
//! Adapted from the authenticated Anchor 0.31.1 error model (Apache-2.0).
//! Reconstructed external framework crate boundary; historical crate identity unresolved.
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    AnchorError(Box<AnchorError>),
    ProgramError(Box<ProgramErrorWithOrigin>),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AnchorError(ae) => Display::fmt(&ae, f),
            Error::ProgramError(pe) => Display::fmt(&pe, f),
        }
    }
}

impl From<AnchorError> for Error {
    fn from(ae: AnchorError) -> Self {
        Self::AnchorError(Box::new(ae))
    }
}

impl From<ProgramError> for Error {
    fn from(program_error: ProgramError) -> Self {
        Self::ProgramError(Box::new(program_error.into()))
    }
}


impl From<ProgramErrorWithOrigin> for Error {
    fn from(pe: ProgramErrorWithOrigin) -> Self {
        Self::ProgramError(Box::new(pe))
    }
}



impl Error {
    pub fn log(&self) {
        match self {
            Error::ProgramError(program_error) => program_error.log(),
            Error::AnchorError(anchor_error) => anchor_error.log(),
        }
    }

    pub fn with_account_name(mut self, account_name: impl ToString) -> Self {
        match &mut self {
            Error::AnchorError(ae) => {
                ae.error_origin = Some(ErrorOrigin::AccountName(account_name.to_string()));
            }
            Error::ProgramError(pe) => {
                pe.error_origin = Some(ErrorOrigin::AccountName(account_name.to_string()));
            }
        };
        self
    }

    pub fn with_source(mut self, source: Source) -> Self {
        match &mut self {
            Error::AnchorError(ae) => {
                ae.error_origin = Some(ErrorOrigin::Source(source));
            }
            Error::ProgramError(pe) => {
                pe.error_origin = Some(ErrorOrigin::Source(source));
            }
        };
        self
    }

    pub fn with_pubkeys(mut self, pubkeys: (Pubkey, Pubkey)) -> Self {
        let pubkeys = Some(ComparedValues::Pubkeys((pubkeys.0, pubkeys.1)));
        match &mut self {
            Error::AnchorError(ae) => ae.compared_values = pubkeys,
            Error::ProgramError(pe) => pe.compared_values = pubkeys,
        };
        self
    }

    pub fn with_values(mut self, values: (impl ToString, impl ToString)) -> Self {
        match &mut self {
            Error::AnchorError(ae) => {
                ae.compared_values = Some(ComparedValues::Values((
                    values.0.to_string(),
                    values.1.to_string(),
                )))
            }
            Error::ProgramError(pe) => {
                pe.compared_values = Some(ComparedValues::Values((
                    values.0.to_string(),
                    values.1.to_string(),
                )))
            }
        };
        self
    }
}

#[derive(Debug)]
pub struct ProgramErrorWithOrigin {
    pub program_error: ProgramError,
    pub error_origin: Option<ErrorOrigin>,
    pub compared_values: Option<ComparedValues>,
}

// Two ProgramErrors are equal when they have the same error code
impl PartialEq for ProgramErrorWithOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.program_error == other.program_error
    }
}
impl Eq for ProgramErrorWithOrigin {}

impl Display for ProgramErrorWithOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.program_error, f)
    }
}

impl ProgramErrorWithOrigin {
    pub fn log(&self) {
        match &self.error_origin {
            None => {
                solana_program::msg!(
                    "ProgramError occurred. Error Code: {:?}. Error Number: {}. Error Message: {}.",
                    self.program_error,
                    u64::from(self.program_error.clone()),
                    self.program_error
                );
            }
            Some(ErrorOrigin::Source(source)) => {
                solana_program::msg!(
                    "ProgramError thrown in {}:{}. Error Code: {:?}. Error Number: {}. Error Message: {}.",
                    source.filename,
                    source.line,
                    self.program_error,
                    u64::from(self.program_error.clone()),
                    self.program_error
                );
            }
            Some(ErrorOrigin::AccountName(account_name)) => {
                // using sol_log because msg! wrongly interprets 5 inputs as u64
                solana_program::log::sol_log(&format!(
                    "ProgramError caused by account: {}. Error Code: {:?}. Error Number: {}. Error Message: {}.",
                    account_name,
                    self.program_error,
                    u64::from(self.program_error.clone()),
                    self.program_error
                ));
            }
        }
        match &self.compared_values {
            Some(ComparedValues::Pubkeys((left, right))) => {
                solana_program::msg!("Left:");
                left.log();
                solana_program::msg!("Right:");
                right.log();
            }
            Some(ComparedValues::Values((left, right))) => {
                solana_program::msg!("Left: {}", left);
                solana_program::msg!("Right: {}", right);
            }
            None => (),
        }
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.error_origin = Some(ErrorOrigin::Source(source));
        self
    }

    pub fn with_account_name(mut self, account_name: impl ToString) -> Self {
        self.error_origin = Some(ErrorOrigin::AccountName(account_name.to_string()));
        self
    }
}

impl From<ProgramError> for ProgramErrorWithOrigin {
    fn from(program_error: ProgramError) -> Self {
        Self {
            program_error,
            error_origin: None,
            compared_values: None,
        }
    }
}

#[derive(Debug)]
pub enum ComparedValues {
    Values((String, String)),
    Pubkeys((Pubkey, Pubkey)),
}

#[derive(Debug)]
pub enum ErrorOrigin {
    Source(Source),
    AccountName(String),
}

#[derive(Debug)]
pub struct AnchorError {
    pub error_name: String,
    pub error_code_number: u32,
    pub error_msg: String,
    pub error_origin: Option<ErrorOrigin>,
    pub compared_values: Option<ComparedValues>,
}

impl AnchorError {
    pub fn log(&self) {
        match &self.error_origin {
            None => {
                solana_program::log::sol_log(&format!(
                    "AnchorError occurred. Error Code: {}. Error Number: {}. Error Message: {}.",
                    self.error_name, self.error_code_number, self.error_msg
                ));
            }
            Some(ErrorOrigin::Source(source)) => {
                solana_program::msg!(
                    "AnchorError thrown in {}:{}. Error Code: {}. Error Number: {}. Error Message: {}.",
                    source.filename,
                    source.line,
                    self.error_name,
                    self.error_code_number,
                    self.error_msg
                );
            }
            Some(ErrorOrigin::AccountName(account_name)) => {
                solana_program::log::sol_log(&format!(
                    "AnchorError caused by account: {}. Error Code: {}. Error Number: {}. Error Message: {}.",
                    account_name,
                    self.error_name,
                    self.error_code_number,
                    self.error_msg
                ));
            }
        }
        match &self.compared_values {
            Some(ComparedValues::Pubkeys((left, right))) => {
                solana_program::msg!("Left:");
                left.log();
                solana_program::msg!("Right:");
                right.log();
            }
            Some(ComparedValues::Values((left, right))) => {
                solana_program::msg!("Left: {}", left);
                solana_program::msg!("Right: {}", right);
            }
            None => (),
        }
    }

    pub fn with_source(mut self, source: Source) -> Self {
        self.error_origin = Some(ErrorOrigin::Source(source));
        self
    }

    pub fn with_account_name(mut self, account_name: impl ToString) -> Self {
        self.error_origin = Some(ErrorOrigin::AccountName(account_name.to_string()));
        self
    }
}

impl Display for AnchorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

/// Two `AnchorError`s are equal when they have the same error code
impl PartialEq for AnchorError {
    fn eq(&self, other: &Self) -> bool {
        self.error_code_number == other.error_code_number
    }
}

impl Eq for AnchorError {}

impl std::convert::From<Error> for solana_program::program_error::ProgramError {
    fn from(e: Error) -> solana_program::program_error::ProgramError {
        match e {
            Error::AnchorError(error) => {
                solana_program::program_error::ProgramError::Custom(
                    error.error_code_number,
                )
            }
            Error::ProgramError(program_error) => program_error.program_error,
        }
    }
}

#[derive(Debug)]
pub struct Source {
    pub filename: &'static str,
    pub line: u32,
}


mod framework_codes;
pub use framework_codes::ErrorCode;
