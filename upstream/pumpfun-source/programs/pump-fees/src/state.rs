use anchor_lang::prelude::*;
















































#[account]
pub struct BuybackVault{pub authority:Pubkey,pub total_claimed:u64,pub total_claimed_token1:u64,pub total_claimed_token2:u64,pub last_claimed:i64,pub claim_rate_limit:i64,pub _reserved:[u8;128]}












impl BuybackVault {
    pub fn claim(&mut self) -> core::result::Result<(), crate::FeeError> {

        let now = Clock::get().unwrap().unix_timestamp;




        if self.claim_rate_limit != 0 {
        if now - self.last_claimed <= self.claim_rate_limit {
            return Err(crate::FeeError::ClaimRateLimitExceeded);
        }
        }
        self.last_claimed = now;
        Ok(())
    }
}
































#[account]
pub struct SocialFeePda{pub bump:u8,pub version:u8,pub user_id:String,pub platform:u8,pub total_claimed:u64,pub last_claimed:u64,pub total_stable_claimed:u64,pub _reserved:[u8;120]}




















#[account]
pub struct DonationFeePda {pub bump:u8,pub version:u8,pub config_id:Pubkey,pub base_mint:Pubkey,pub quote_mint:Pubkey,pub creator:Pubkey,pub total_donated:u64,pub last_crank_ts:i64,pub reserved:[u8;64]}

















#[account]
pub struct FeeProgramGlobal {
    pub bump: u8,
    pub authority: Pubkey,
    pub disable_flags: u8,
    pub social_claim_authority: Pubkey,
    pub claim_rate_limit: u64,
    pub _reserved: [u8; 256],
}







































































































































































































impl crate::types::FeeConfig {
    #[inline(never)]
    pub fn set_flat_fees(&mut self, flat_fees: crate::types::Fees) -> Result<()> {
        require!(flat_fees.is_valid(), crate::FeeError::InvalidFeeTotal);
        self.flat_fees = flat_fees;
        Ok(())
    }

    #[inline(never)]
    pub fn set_exotic_flat_fees(&mut self, exotic_flat_fees: crate::types::Fees) -> Result<()> {
        require!(exotic_flat_fees.is_valid(), crate::FeeError::InvalidFeeTotal);
        self.exotic_flat_fees = exotic_flat_fees;
        Ok(())
    }
}



    #[inline(never)]
pub fn replace_normal_tiers(current: &mut Vec<crate::types::FeeTier>, incoming: &[crate::types::FeeTier], offset: usize, append: bool) -> Result<()> {

    require!(!incoming.is_empty(), crate::FeeError::NoFeeTiers);

    require_gte!(current.len(), offset, crate::FeeError::OffsetNotContinuous);

    let new_tiers = if append {
        let mut merged = current.clone();
        let end = offset.saturating_add(incoming.len());
        if end > merged.len() {
            merged.resize(end, crate::types::FeeTier { market_cap_lamports_threshold: 0, fees: crate::types::Fees { lp_fee_bps: 0, protocol_fee_bps: 0, creator_fee_bps: 0 } });
        }





            merged[offset..end].clone_from_slice(incoming);
        merged
    } else {
        incoming.to_vec()
    };


    require_gte!(50usize, new_tiers.len(), crate::FeeError::TooManyFeeTiers);





    require!(new_tiers.iter().all(|tier| tier.fees.is_valid()), crate::FeeError::InvalidFeeTotal);






    require!(new_tiers.windows(2).all(|pair| pair[0].market_cap_lamports_threshold < pair[1].market_cap_lamports_threshold), crate::FeeError::FeeTiersNotSorted);
    *current = new_tiers;
    Ok(())
}











    #[inline(never)]
pub fn replace_stable_tiers(config: &mut crate::types::FeeConfig, incoming: &[crate::types::FeeTier], offset: usize, append: bool) -> Result<()> {
    let current = &mut config.stable_fee_tiers;
    require!(!incoming.is_empty(), crate::FeeError::NoFeeTiers);

    require_gte!(current.len(), offset, crate::FeeError::OffsetNotContinuous);

    let new_tiers = if append {
        let mut merged = current.clone();
        let end = offset.saturating_add(incoming.len());
        if end > merged.len() {
            merged.resize(end, crate::types::FeeTier { market_cap_lamports_threshold: 0, fees: crate::types::Fees { lp_fee_bps: 0, protocol_fee_bps: 0, creator_fee_bps: 0 } });
        }





            merged[offset..end].clone_from_slice(incoming);
        merged
    } else {
        incoming.to_vec()
    };


    require_gte!(50usize, new_tiers.len(), crate::FeeError::TooManyFeeTiers);





    require!(new_tiers.iter().all(|tier| tier.fees.is_valid()), crate::FeeError::InvalidFeeTotal);






    require!(new_tiers.windows(2).all(|pair| pair[0].market_cap_lamports_threshold < pair[1].market_cap_lamports_threshold), crate::FeeError::FeeTiersNotSorted);
    *current = new_tiers;
    Ok(())
}

impl BuybackVault {
    pub(crate) fn add_total_claimed(&mut self, amount: u64) -> Result<()> { self.total_claimed = self.total_claimed.checked_add(amount).ok_or(crate::FeeError::MathOverflow)?; Ok(()) }
    pub(crate) fn add_total_claimed_token1(&mut self, amount: u64) -> Result<()> { self.total_claimed_token1 = self.total_claimed_token1.checked_add(amount).ok_or(crate::FeeError::MathOverflow)?; Ok(()) }
    pub(crate) fn add_total_claimed_token2(&mut self, amount: u64) -> Result<()> { self.total_claimed_token2 = self.total_claimed_token2.checked_add(amount).ok_or(crate::FeeError::MathOverflow)?; Ok(()) }
}
