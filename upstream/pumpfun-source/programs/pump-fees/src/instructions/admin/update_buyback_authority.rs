use anchor_lang::prelude::*;
use crate::FeeError;
use crate::buyback::UpdateBuyback;
































 pub fn update_buyback_authority(ctx:Context<UpdateBuyback>,index:u8,new_authority:Pubkey)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);ctx.accounts.buyback_vault.authority=new_authority;Ok(())}










 pub fn update_buyback_claim_rate_limit(ctx:Context<UpdateBuyback>,index:u8,claim_rate_limit:i64)->Result<()>{require_gt!(8u8,index,FeeError::InvalidBuybackIndex);ctx.accounts.buyback_vault.claim_rate_limit=claim_rate_limit;Ok(())}
