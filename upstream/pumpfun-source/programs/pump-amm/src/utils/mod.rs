pub mod quote;

/// Canonical Pump authority for pools created from a bonding curve.
pub fn pool_authority(base_mint: &anchor_lang::prelude::Pubkey) -> anchor_lang::prelude::Pubkey {
    let seeds: &[&[u8]] = &[&crate::utils::POOL_AUTHORITY_SEED, base_mint.as_ref()];
    anchor_lang::prelude::Pubkey::find_program_address(seeds, &crate::trade::PUMP).0
}

/// Pump bonding curve account associated with a base mint.
pub fn bonding_curve(base_mint: &anchor_lang::prelude::Pubkey) -> anchor_lang::prelude::Pubkey {
    let seeds: &[&[u8]] = &[&crate::utils::BONDING_CURVE_SEED, base_mint.as_ref()];
    anchor_lang::prelude::Pubkey::find_program_address(seeds, &crate::trade::PUMP).0
}

pub mod math;
pub mod token;

pub mod transfer_fee;

pub(crate) static POOL_AUTHORITY_SEED: [u8; 14] = *b"pool-authority";
pub(crate) static BONDING_CURVE_SEED: [u8; 13] = *b"bonding-curve";
pub(crate) static HOLDER_REWARDS_SEED: [u8; 14] = *b"holder-rewards";
