use anchor_lang::prelude::*;
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.
// Historical source text preceding this macro is unknown.

#[account]
pub struct DistributionV1 {pub bump:u8,pub state:DistributionV1State,pub config_id:[u8;32],pub mint:Pubkey,pub epoch:u128,pub total_amount:u64}
#[derive(AnchorSerialize,AnchorDeserialize,Clone,PartialEq,Eq)]
pub enum DistributionV1State {Uninitialized,Initialized}
