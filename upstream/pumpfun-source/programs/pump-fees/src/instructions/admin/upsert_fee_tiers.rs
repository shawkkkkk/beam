use anchor_lang::prelude::*;
use crate::types::*;
use crate::donation::ConfigStatus;


#[event]
#[derive(Clone)]
pub struct UpsertFeeTiersEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub fee_tiers: Vec<FeeTier>, pub offset: u8,
}



#[event]
#[derive(Clone)]
pub struct UpsertStableFeeTiersEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub stable_fee_tiers: Vec<FeeTier>, pub offset: u8,
}
