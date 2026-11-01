use anchor_lang::prelude::*;
use crate::types::*;
use crate::donation::ConfigStatus;



#[event]
#[derive(Clone)]
pub struct UpdateFeeConfigEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub fee_tiers: Vec<FeeTier>, pub flat_fees: Fees,
}



#[event]
#[derive(Clone)]
pub struct UpdateStableFeeConfigEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey,
    pub stable_fee_tiers: Vec<FeeTier>, pub flat_fees: Fees,
}
