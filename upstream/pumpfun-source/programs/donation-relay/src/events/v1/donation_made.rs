use anchor_lang::prelude::*; use crate::anchor_framework::{DaasEventDiscriminator};



#[event]
pub struct DonationMadeV1Event {
 pub daas_event_discriminator:DaasEventDiscriminator,pub config_id:[u8;32],pub mint:Pubkey,pub epoch:u128,
 pub gross_amount:u64,pub tip:u64,pub message:String,pub credited_to:Pubkey,
}
