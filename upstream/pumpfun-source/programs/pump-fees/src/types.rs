//! IDL argument types needed for instruction deserialization.
use anchor_lang::prelude::*;

pub use crate::{FeeTier, Fees};












#[derive(AnchorSerialize, Clone)]
pub struct OptionBool(pub bool);

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct Shareholder {
    pub address: Pubkey,
    pub share_bps: u16,
}

impl AnchorDeserialize for OptionBool {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut flag = [0u8];
        match reader.read(&mut flag)? {
            0 => Ok(Self(false)),
            _ => match flag[0] { 0 => Ok(Self(false)), 1 => Ok(Self(true)),
                b => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid bool representation: {}", b))) },
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeeConfig {
    pub bump: u8,
    pub admin: Pubkey,
    pub flat_fees: Fees,
    pub fee_tiers: Vec<FeeTier>,
    pub stable_fee_tiers: Vec<FeeTier>,
    pub exotic_flat_fees: Fees,
}
impl FeeConfig {
    fn read_legacy(buf: &mut &[u8]) -> Result<Self> {
        let bump = u8::deserialize(buf)?;
        let admin = Pubkey::deserialize(buf)?;
        let flat_fees = Fees::deserialize(buf)?;
        let fee_tiers = Vec::<FeeTier>::deserialize(buf)?;
        Ok(Self { bump, admin, flat_fees, fee_tiers, stable_fee_tiers: vec![],
            exotic_flat_fees: Fees { lp_fee_bps: 0, protocol_fee_bps: 0, creator_fee_bps: 0 } })
    }
    fn read_stable(buf: &mut &[u8]) -> Result<Self> {
        let bump = u8::deserialize(buf)?;
        let admin = Pubkey::deserialize(buf)?;
        let flat_fees = Fees::deserialize(buf)?;
        let fee_tiers = Vec::<FeeTier>::deserialize(buf)?;
        let stable_fee_tiers = Vec::<FeeTier>::deserialize(buf)?;
        Ok(Self { bump, admin, flat_fees, fee_tiers, stable_fee_tiers,
            exotic_flat_fees: Fees { lp_fee_bps: 0, protocol_fee_bps: 0, creator_fee_bps: 0 } })
    }
    fn read_full(buf: &mut &[u8]) -> Result<Self> {
        let bump = u8::deserialize(buf)?;
        let admin = Pubkey::deserialize(buf)?;
        let flat_fees = Fees::deserialize(buf)?;
        let fee_tiers = Vec::<FeeTier>::deserialize(buf)?;
        let stable_fee_tiers = Vec::<FeeTier>::deserialize(buf)?;
        let exotic_flat_fees = Fees::deserialize(buf)?;
        Ok(Self { bump, admin, flat_fees, fee_tiers, stable_fee_tiers, exotic_flat_fees })
    }
}
impl Owner for FeeConfig { fn owner() -> Pubkey { crate::ID } }
impl anchor_lang::Discriminator for FeeConfig {
    const DISCRIMINATOR: &'static [u8] = &[143, 52, 146, 187, 219, 123, 76, 155];
}
impl AccountSerialize for FeeConfig {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        use anchor_lang::Discriminator;
        writer.write_all(&Self::DISCRIMINATOR)?;
        self.serialize(writer)?;
        Ok(())
    }
}
impl AccountDeserialize for FeeConfig {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        use anchor_lang::Discriminator;
        if buf.len() < 8 { return Err(ProgramError::InvalidAccountData.into()); }
        if &buf[..8] != Self::DISCRIMINATOR { return Err(ProgramError::InvalidAccountData.into()); }
        *buf = &buf[8..];
        decode_fee_config_payload(buf)
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 { return Err(ProgramError::InvalidAccountData.into()); }
        *buf = &buf[8..];
        decode_fee_config_payload(buf)

    }
}

pub use crate::state::FeeProgramGlobal;
#[event]
pub struct SetAuthorityEvent { pub timestamp: i64, pub old_authority: Pubkey, pub new_authority: Pubkey }
#[event]
pub struct SetSocialClaimAuthorityEvent { pub timestamp: i64, pub social_claim_authority: Pubkey }
#[event]
pub struct SetDisableFlagsEvent { pub timestamp: i64, pub disable_flags: u8 }
#[event]
pub struct SetClaimRateLimitEvent { pub timestamp: i64, pub claim_rate_limit: u64 }

#[event]
pub struct InitializeFeeProgramGlobalEvent {
    pub timestamp: i64, pub authority: Pubkey, pub social_claim_authority: Pubkey,
    pub disable_flags: u8, pub claim_rate_limit: u64,
}

#[event]
#[derive(Clone)]
pub struct SetExoticFlatFeesEvent {
    pub timestamp: i64, pub admin: Pubkey, pub fee_config: Pubkey, pub exotic_flat_fees: Fees,
}
pub use crate::instructions::admin::update_fee_config::UpdateFeeConfigEvent;





pub use crate::instructions::admin::update_fee_config::UpdateStableFeeConfigEvent;






#[event]
pub struct ExtendFeeConfigEvent {
    pub fee_config: Pubkey, pub user: Pubkey, pub current_size: u64,
    pub new_size: u64, pub timestamp: i64,
}

pub use crate::instructions::admin::upsert_fee_tiers::UpsertFeeTiersEvent;





pub use crate::instructions::admin::upsert_fee_tiers::UpsertStableFeeTiersEvent;






#[event]
pub struct UpdateAdminEvent { pub timestamp: i64, pub old_admin: Pubkey, pub new_admin: Pubkey }

#[event]
pub struct InitializeFeeConfigEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub fee_config: Pubkey,
}

#[inline(never)]
fn decode_fee_config_payload(buf: &mut &[u8]) -> Result<FeeConfig> {
    if buf.len() < 2504 { return Err(ProgramError::InvalidAccountData.into()); }
    if buf.len() < 4065 { return FeeConfig::read_legacy(buf); }
    if buf.len() < 4089 { return FeeConfig::read_stable(buf); }
    FeeConfig::read_full(buf)
}
