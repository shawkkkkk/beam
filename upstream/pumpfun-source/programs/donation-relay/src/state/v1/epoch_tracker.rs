use anchor_lang::prelude::*;
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
// Historical source text preceding the account macro is unknown.
#[account]
pub struct EpochTrackerV1 {pub bump:u8,pub state:EpochTrackerV1State,pub config_id:[u8;32],pub mint:Pubkey,pub current_epoch:u128}
#[derive(AnchorSerialize,AnchorDeserialize,Clone,PartialEq,Eq)]
pub enum EpochTrackerV1State {Uninitialized,Initialized}

impl EpochTrackerV1 {pub fn increment(&mut self)->anchor_lang::Result<()> {self.current_epoch=self.current_epoch.checked_add(1).ok_or(crate::business_error_codes::ErrorCode::ArithmeticOverflow)?;Ok(())}}
