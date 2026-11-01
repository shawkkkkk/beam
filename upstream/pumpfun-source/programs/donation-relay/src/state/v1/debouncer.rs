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
pub struct DebouncerV1 {pub bump:u8,pub state:DebouncerV1State,pub config_id:[u8;32],pub mint:Pubkey,pub total_amount:u64}
#[derive(AnchorSerialize,AnchorDeserialize,Clone,PartialEq,Eq)]
pub enum DebouncerV1State {Uninitialized,Initialized}
