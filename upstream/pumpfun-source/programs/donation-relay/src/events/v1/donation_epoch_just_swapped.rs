use anchor_lang::prelude::*; use crate::anchor_framework::{DaasEventDiscriminator};



#[event]
pub struct DonationEpochJustSwappedV1Event {pub daas_event_discriminator:DaasEventDiscriminator,pub config_id:[u8;32],pub mint:Pubkey,pub epoch:u128,pub distribution:Pubkey,pub swap_receipt:Pubkey,pub output_token_account:Pubkey,pub swap_output_amount:u64,pub settlement_mint:Pubkey}
