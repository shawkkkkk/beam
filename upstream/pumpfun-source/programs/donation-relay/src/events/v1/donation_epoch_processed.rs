use anchor_lang::prelude::*; use crate::anchor_framework::{DaasEventDiscriminator,Receipt};



#[event]
pub struct DonationEpochProcessedV1Event {pub daas_event_discriminator:DaasEventDiscriminator,pub config_id:[u8;32],pub mint:Pubkey,pub epoch:u128,pub settled_amount:u64,pub settlement_mint:Pubkey,pub transfers:Vec<Receipt>}
