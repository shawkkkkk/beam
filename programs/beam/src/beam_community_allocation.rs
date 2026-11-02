//! Disabled community-allocation model.
//!
//! The maintainer decided the curve charge is 50 bps all-in. Creator payouts
//! come out of the 42 bps community share. An additional creator charge is
//! rejected. How many of those 42 bps go to creators is unset until a caller
//! sets it, and the value must be <= 42.
//!
//! The flagship coin is $BEAM, launched later on this launchpad. Its mint is
//! a required config pubkey. This module hard-codes no mint address.
//! Holder rewards are a later airdrop of that mint. Flagship proceeds are a
//! later buyback whose acquired tokens are burned. Hold is rejected.
//!
//! Nothing here collects, transfers, burns, or submits an order.
//!
//! # Creator amount
//!
//! For a set `bps` in `0..=42` and community integer `c`:
//!
//! ```text
//! creator = floor(c * bps / 42)
//! available = c - creator
//! ```
//!
//! `creator <= c` because `bps <= 42`. The holder/flagship parameters then
//! divide only `available`.
//!
//! # Remainder rounding
//!
//! Let `D = 10_000` and `h + f = D`. For available amount `a`,
//! `a - floor(a*h/D) - floor(a*f/D)` is 0 or 1. `DustAssignment` gives that
//! lamport to one leg. A 0-share leg receives 0.
//!
//! # Conservation
//!
//! `community = creator + holder_rewards + flagship_buyback`.
//! With [`crate::beam_economics::allocate`], `total = treasury + founder + community`.
//! Treasury and founder are not used to fund the creator, holders, or buyback.
//!
//! The separate 10_000_000 whole-token Slop contributor allocation is not part
//! of this sum.
use anchor_lang::prelude::*;

/// Parts of the available community amount. The two shares sum to this value.
pub const SPLIT_BPS_DENOMINATOR: u32 = 10_000;

/// Nominal community component of the 42/5/3 proposal, in trade basis points.
/// Creator bps are a subset of this value. This does not change
/// [`crate::beam_economics`].
pub const COMMUNITY_TRADE_BPS: u16 = 42;

/// Provisional test setting: half of the available community amount.
/// This is 5_000 / 10_000 of that amount, a different scale from trade bps.
pub const PROVISIONAL_AVAILABLE_SHARE_BPS: u16 = 5_000;

/// Whole $BEAM tokens reserved for Slop Cash contributors. This is a supply
/// allocation, not a fee. Mint decimals are unset, so this is not a base-unit
/// amount. It is not funded and it is not distributed.
pub const SLOP_CONTRIBUTOR_WHOLE_TOKENS: u64 = 10_000_000;

/// First custom error for this module. Beam policy errors use 7000.
/// Draft fee-policy code uses 7100, so these start at 7200.
pub const ERROR_OFFSET: u32 = 7200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationMode {
    /// Default. [`divide`] returns [`CommunityAllocationError::Disabled`].
    Disabled,
    /// [`divide`] may return integers. The mode has no collection path.
    ArithmeticOnly,
}

/// The only creator-funding route stored on a model.
/// An additional creator charge is not a variant; [`creator_funding`] rejects it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatorFunding {
    /// Take the creator payout from the community integer.
    /// `None` means the size is unset. `Some` is trade basis points and must
    /// be <= [`COMMUNITY_TRADE_BPS`].
    FromCommunity { bps_of_trade: Option<u16> },
}

/// Input to [`creator_funding`]. `AdditionalCharge` is the removed route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatorFundingRequest {
    FromCommunity { bps_of_trade: Option<u16> },
    AdditionalCharge,
}

/// Holder-rewards leg. The lamports are a budget for a later airdrop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HolderRewardUse {
    /// Paid in the flagship mint. No airdrop instruction exists here.
    FlagshipMintAirdrop,
}

/// Flagship leg. The lamports are a budget for a later buyback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagshipProceedsUse {
    /// Buy the flagship mint later. No order instruction exists here.
    Buyback,
}

/// What happens to flagship tokens acquired by that buyback.
/// `Hold` is not a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcquiredFlagshipDisposition {
    Burn,
}

/// A requested disposition. [`accept_flagship_disposition`] rejects `Hold`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestedFlagshipDisposition {
    Burn,
    Hold,
}

/// Which leg receives the 0-or-1 lamport left by flooring both shares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DustAssignment {
    /// Holder rewards receive `available - floor(available * flagship_bps / 10_000)`.
    RemainderToHolderRewards,
    /// Flagship buyback receives `available - floor(available * holder_bps / 10_000)`.
    RemainderToFlagshipBuyback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitParameters {
    pub holder_rewards_bps: u16,
    pub flagship_buyback_bps: u16,
    pub dust: DustAssignment,
}

/// Wallets that would be credited if a later design collects funds.
/// The all-zero pubkey is unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipients {
    pub holder_rewards: Pubkey,
    pub flagship_buyback: Pubkey,
    pub creator: Pubkey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocationModel {
    pub mode: AllocationMode,
    pub creator_funding: CreatorFunding,
    pub split: SplitParameters,
    pub recipients: Recipients,
    /// Required for arithmetic. All-zero is unset. No mint is hard-coded.
    pub flagship_mint: Pubkey,
    pub holder_reward_use: HolderRewardUse,
    pub flagship_proceeds_use: FlagshipProceedsUse,
    pub acquired_disposition: AcquiredFlagshipDisposition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunityDivision {
    pub community_lamports: u64,
    pub creator_bps_of_trade: u16,
    pub creator_lamports: u64,
    pub available_lamports: u64,
    pub holder_rewards: u64,
    pub flagship_buyback: u64,
    /// `available - floor(holder share) - floor(flagship share)`, 0 or 1.
    pub rounding_remainder: u64,
    pub flagship_mint: Pubkey,
    pub holder_reward_use: HolderRewardUse,
    pub acquired_disposition: AcquiredFlagshipDisposition,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProposedCommunityLeg {
    pub fee: crate::beam_economics::FeeAllocation,
    pub community: CommunityDivision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupplyAllocationMode {
    /// Default for the Slop contributor allocation.
    Disabled,
    /// Integer evaluation only. Does not mint, fund, or claim.
    ArithmeticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupplyDivision {
    pub allocation_whole_tokens: u64,
    pub total_points: u128,
    pub assigned_whole_tokens: u64,
    pub unassigned_whole_tokens: u64,
}

#[error_code(offset = 7200)]
pub enum CommunityAllocationError {
    #[msg("Community allocation is disabled")]
    Disabled,
    #[msg("Creator basis points within the community share are unset")]
    CreatorBpsUnset,
    #[msg("Holder and flagship shares must sum to 10000")]
    InvalidSplit,
    #[msg("A configured community recipient is unset")]
    UnsetRecipient,
    #[msg("Creator basis points exceed the 42 bps community share")]
    CreatorBpsAboveCommunityShare,
    #[msg(
        "An additional creator charge is rejected; creator payouts come from the community share"
    )]
    AdditionalChargeRejected,
    #[msg("Community allocation arithmetic overflowed")]
    Overflow,
    #[msg("Flagship mint is unset")]
    UnsetFlagshipMint,
    #[msg("Acquired flagship tokens are burned; hold is rejected")]
    HoldDispositionRejected,
    #[msg("Slop contributor supply allocation is disabled")]
    SupplyAllocationDisabled,
    #[msg("Points total is zero")]
    ZeroPoints,
    #[msg("Points and share lengths differ")]
    PointsLength,
}

impl AllocationModel {
    /// Disabled, with creator-from-community selected and the creator bps unset.
    /// The flagship mint is unset. The declared uses are airdrop and burn.
    pub fn disabled() -> Self {
        Self {
            mode: AllocationMode::Disabled,
            creator_funding: CreatorFunding::FromCommunity { bps_of_trade: None },
            split: SplitParameters {
                holder_rewards_bps: 0,
                flagship_buyback_bps: 0,
                dust: DustAssignment::RemainderToHolderRewards,
            },
            recipients: Recipients {
                holder_rewards: Pubkey::default(),
                flagship_buyback: Pubkey::default(),
                creator: Pubkey::default(),
            },
            flagship_mint: Pubkey::default(),
            holder_reward_use: HolderRewardUse::FlagshipMintAirdrop,
            flagship_proceeds_use: FlagshipProceedsUse::Buyback,
            acquired_disposition: AcquiredFlagshipDisposition::Burn,
        }
    }

    /// Parameters that [`divide`] may evaluate. `FromCommunity { None }` and
    /// an all-zero flagship mint still fail inside `divide`.
    pub fn arithmetic_only(
        creator_funding: CreatorFunding,
        split: SplitParameters,
        recipients: Recipients,
        flagship_mint: Pubkey,
    ) -> Self {
        Self {
            mode: AllocationMode::ArithmeticOnly,
            creator_funding,
            split,
            recipients,
            flagship_mint,
            holder_reward_use: HolderRewardUse::FlagshipMintAirdrop,
            flagship_proceeds_use: FlagshipProceedsUse::Buyback,
            acquired_disposition: AcquiredFlagshipDisposition::Burn,
        }
    }
}

/// Accept the from-community route. Reject an additional creator charge.
pub fn creator_funding(request: CreatorFundingRequest) -> Result<CreatorFunding> {
    match request {
        CreatorFundingRequest::AdditionalCharge => {
            err!(CommunityAllocationError::AdditionalChargeRejected)
        }
        CreatorFundingRequest::FromCommunity { bps_of_trade } => {
            if let Some(bps) = bps_of_trade {
                require!(
                    bps <= COMMUNITY_TRADE_BPS,
                    CommunityAllocationError::CreatorBpsAboveCommunityShare
                );
            }
            Ok(CreatorFunding::FromCommunity { bps_of_trade })
        }
    }
}

/// Burn is the only accepted disposition of acquired flagship tokens.
pub fn accept_flagship_disposition(
    requested: RequestedFlagshipDisposition,
) -> Result<AcquiredFlagshipDisposition> {
    match requested {
        RequestedFlagshipDisposition::Burn => Ok(AcquiredFlagshipDisposition::Burn),
        RequestedFlagshipDisposition::Hold => {
            err!(CommunityAllocationError::HoldDispositionRejected)
        }
    }
}

/// Provisional 50/50 test parameters. `dust` is selected by the caller.
pub fn provisional_even_split(dust: DustAssignment) -> SplitParameters {
    SplitParameters {
        holder_rewards_bps: PROVISIONAL_AVAILABLE_SHARE_BPS,
        flagship_buyback_bps: PROVISIONAL_AVAILABLE_SHARE_BPS,
        dust,
    }
}

/// Divide `community_lamports` under `model`.
pub fn divide(model: &AllocationModel, community_lamports: u64) -> Result<CommunityDivision> {
    if model.mode == AllocationMode::Disabled {
        return err!(CommunityAllocationError::Disabled);
    }
    require!(
        is_set(model.flagship_mint),
        CommunityAllocationError::UnsetFlagshipMint
    );
    require_declared_uses(model)?;
    let creator_bps = creator_bps(model.creator_funding)?;
    validate_split(model.split)?;
    require_recipients(model, creator_bps)?;
    let creator_taken = creator_lamports(community_lamports, creator_bps)?;
    let available = community_lamports
        .checked_sub(creator_taken)
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    let (holder_rewards, flagship_buyback, rounding_remainder) =
        split_available(available, model.split)?;
    let attributed = creator_taken
        .checked_add(holder_rewards)
        .and_then(|sum| sum.checked_add(flagship_buyback))
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    require!(
        attributed == community_lamports,
        CommunityAllocationError::Overflow
    );
    Ok(CommunityDivision {
        community_lamports,
        creator_bps_of_trade: creator_bps,
        creator_lamports: creator_taken,
        available_lamports: available,
        holder_rewards,
        flagship_buyback,
        rounding_remainder,
        flagship_mint: model.flagship_mint,
        holder_reward_use: model.holder_reward_use,
        acquired_disposition: model.acquired_disposition,
    })
}

/// Divide the community integer from [`crate::beam_economics::allocate`].
/// Treasury and founder values are returned unchanged.
pub fn divide_proposed_community_leg(
    model: &AllocationModel,
    quote_amount: u64,
) -> Result<ProposedCommunityLeg> {
    let fee = crate::beam_economics::allocate(quote_amount)?;
    let community = divide(model, fee.community)?;
    require!(
        community.community_lamports == fee.community,
        CommunityAllocationError::Overflow
    );
    Ok(ProposedCommunityLeg { fee, community })
}

/// The Slop contributor supply entrypoint. It stays disabled.
pub fn divide_slop_contributor_supply(
    points: &[u64],
    out_shares: &mut [u64],
) -> Result<SupplyDivision> {
    let _ = (points, out_shares);
    err!(CommunityAllocationError::SupplyAllocationDisabled)
}

/// Pro-rata whole tokens by points. Each share is
/// `floor(allocation * points_i / total_points)`. The unassigned remainder
/// stays in the allocation. `Disabled` performs no split.
pub fn divide_supply_by_points(
    mode: SupplyAllocationMode,
    allocation_whole_tokens: u64,
    points: &[u64],
    out_shares: &mut [u64],
) -> Result<SupplyDivision> {
    if mode == SupplyAllocationMode::Disabled {
        return err!(CommunityAllocationError::SupplyAllocationDisabled);
    }
    require!(
        points.len() == out_shares.len(),
        CommunityAllocationError::PointsLength
    );
    let mut total_points: u128 = 0;
    for points_i in points {
        total_points = total_points
            .checked_add(u128::from(*points_i))
            .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    }
    require!(total_points > 0, CommunityAllocationError::ZeroPoints);
    let mut assigned: u128 = 0;
    for (index, points_i) in points.iter().enumerate() {
        let share = u128::from(allocation_whole_tokens)
            .checked_mul(u128::from(*points_i))
            .ok_or_else(|| error!(CommunityAllocationError::Overflow))?
            / total_points;
        let share = u64::try_from(share).map_err(|_| error!(CommunityAllocationError::Overflow))?;
        out_shares[index] = share;
        assigned = assigned
            .checked_add(u128::from(share))
            .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    }
    let allocation = u128::from(allocation_whole_tokens);
    require!(assigned <= allocation, CommunityAllocationError::Overflow);
    let unassigned = u64::try_from(allocation - assigned)
        .map_err(|_| error!(CommunityAllocationError::Overflow))?;
    require!(
        points.is_empty() || unassigned < points.len() as u64,
        CommunityAllocationError::Overflow
    );
    Ok(SupplyDivision {
        allocation_whole_tokens,
        total_points,
        assigned_whole_tokens: u64::try_from(assigned)
            .map_err(|_| error!(CommunityAllocationError::Overflow))?,
        unassigned_whole_tokens: unassigned,
    })
}

fn require_declared_uses(model: &AllocationModel) -> Result<()> {
    match model.holder_reward_use {
        HolderRewardUse::FlagshipMintAirdrop => {}
    }
    match model.flagship_proceeds_use {
        FlagshipProceedsUse::Buyback => {}
    }
    match model.acquired_disposition {
        AcquiredFlagshipDisposition::Burn => {}
    }
    Ok(())
}

fn creator_bps(funding: CreatorFunding) -> Result<u16> {
    match funding {
        CreatorFunding::FromCommunity { bps_of_trade: None } => {
            err!(CommunityAllocationError::CreatorBpsUnset)
        }
        CreatorFunding::FromCommunity {
            bps_of_trade: Some(bps),
        } => {
            require!(
                bps <= COMMUNITY_TRADE_BPS,
                CommunityAllocationError::CreatorBpsAboveCommunityShare
            );
            Ok(bps)
        }
    }
}

fn creator_lamports(community: u64, bps: u16) -> Result<u64> {
    let product = u128::from(community)
        .checked_mul(u128::from(bps))
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    let share = product / u128::from(COMMUNITY_TRADE_BPS);
    let lamports = u64::try_from(share).map_err(|_| error!(CommunityAllocationError::Overflow))?;
    require!(lamports <= community, CommunityAllocationError::Overflow);
    Ok(lamports)
}

fn validate_split(split: SplitParameters) -> Result<()> {
    let sum = u32::from(split.holder_rewards_bps) + u32::from(split.flagship_buyback_bps);
    require!(
        sum == SPLIT_BPS_DENOMINATOR,
        CommunityAllocationError::InvalidSplit
    );
    Ok(())
}

fn require_recipients(model: &AllocationModel, creator_bps: u16) -> Result<()> {
    if creator_bps > 0 {
        require!(
            is_set(model.recipients.creator),
            CommunityAllocationError::UnsetRecipient
        );
    }
    if model.split.holder_rewards_bps > 0 {
        require!(
            is_set(model.recipients.holder_rewards),
            CommunityAllocationError::UnsetRecipient
        );
    }
    if model.split.flagship_buyback_bps > 0 {
        require!(
            is_set(model.recipients.flagship_buyback),
            CommunityAllocationError::UnsetRecipient
        );
    }
    Ok(())
}

fn is_set(key: Pubkey) -> bool {
    key != Pubkey::default()
}

fn floor_share(amount: u64, bps: u16) -> Result<u64> {
    let product = u128::from(amount)
        .checked_mul(u128::from(bps))
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    let share = product / u128::from(SPLIT_BPS_DENOMINATOR);
    u64::try_from(share).map_err(|_| error!(CommunityAllocationError::Overflow))
}

fn split_available(available: u64, split: SplitParameters) -> Result<(u64, u64, u64)> {
    let holder_floor = floor_share(available, split.holder_rewards_bps)?;
    let flagship_floor = floor_share(available, split.flagship_buyback_bps)?;
    let rounding_remainder = available
        .checked_sub(holder_floor)
        .and_then(|rest| rest.checked_sub(flagship_floor))
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    require!(rounding_remainder <= 1, CommunityAllocationError::Overflow);
    let (holder_rewards, flagship_buyback) = match split.dust {
        DustAssignment::RemainderToHolderRewards => {
            let holder = available
                .checked_sub(flagship_floor)
                .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
            (holder, flagship_floor)
        }
        DustAssignment::RemainderToFlagshipBuyback => {
            let flagship = available
                .checked_sub(holder_floor)
                .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
            (holder_floor, flagship)
        }
    };
    let sum = holder_rewards
        .checked_add(flagship_buyback)
        .ok_or_else(|| error!(CommunityAllocationError::Overflow))?;
    require!(sum == available, CommunityAllocationError::Overflow);
    Ok((holder_rewards, flagship_buyback, rounding_remainder))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_code(error: anchor_lang::error::Error) -> u32 {
        match error {
            anchor_lang::error::Error::AnchorError(inner) => inner.error_code_number,
            other => panic!("expected community allocation error, got {other:?}"),
        }
    }

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn recipients(holder: bool, flagship: bool, creator: bool) -> Recipients {
        Recipients {
            holder_rewards: if holder { key(1) } else { Pubkey::default() },
            flagship_buyback: if flagship { key(2) } else { Pubkey::default() },
            creator: if creator { key(3) } else { Pubkey::default() },
        }
    }

    fn full_recipients() -> Recipients {
        recipients(true, true, true)
    }

    fn model(
        creator_bps: u16,
        holder_bps: u16,
        flagship_bps: u16,
        dust: DustAssignment,
        recipients: Recipients,
    ) -> AllocationModel {
        AllocationModel::arithmetic_only(
            CreatorFunding::FromCommunity {
                bps_of_trade: Some(creator_bps),
            },
            SplitParameters {
                holder_rewards_bps: holder_bps,
                flagship_buyback_bps: flagship_bps,
                dust,
            },
            recipients,
            key(9),
        )
    }

    fn floor_bps(amount: u64, bps: u16) -> u64 {
        let whole = (u128::from(amount) / 10_000) * u128::from(bps);
        let remainder = (u128::from(amount % 10_000) * u128::from(bps)) / 10_000;
        u64::try_from(whole + remainder).unwrap()
    }

    fn creator_take(community: u64, bps: u16) -> u64 {
        u64::try_from(u128::from(community) * u128::from(bps) / u128::from(COMMUNITY_TRADE_BPS))
            .unwrap()
    }

    fn assert_division(spec: &AllocationModel, community: u64) {
        let got = divide(spec, community).unwrap_or_else(|error| {
            panic!("divide failed for community={community} model={spec:?}: {error:?}")
        });
        let bps = match spec.creator_funding {
            CreatorFunding::FromCommunity {
                bps_of_trade: Some(bps),
            } => bps,
            other => panic!("expected a set creator bps, got {other:?}"),
        };
        let creator = creator_take(community, bps);
        let available = community - creator;
        let holder_floor = floor_bps(available, spec.split.holder_rewards_bps);
        let flagship_floor = floor_bps(available, spec.split.flagship_buyback_bps);
        let rounding = available - holder_floor - flagship_floor;
        assert!(
            rounding <= 1,
            "remainder {rounding} for available={available}"
        );
        let (holder, flagship) = match spec.split.dust {
            DustAssignment::RemainderToHolderRewards => {
                (available - flagship_floor, flagship_floor)
            }
            DustAssignment::RemainderToFlagshipBuyback => (holder_floor, available - holder_floor),
        };
        assert_eq!(got.community_lamports, community);
        assert_eq!(got.creator_bps_of_trade, bps);
        assert_eq!(got.creator_lamports, creator);
        assert!(got.creator_lamports <= community);
        assert_eq!(got.available_lamports, available);
        assert_eq!(got.holder_rewards, holder);
        assert_eq!(got.flagship_buyback, flagship);
        assert_eq!(got.rounding_remainder, rounding);
        assert_eq!(
            got.creator_lamports + got.holder_rewards + got.flagship_buyback,
            community
        );
        assert_eq!(got.flagship_mint, spec.flagship_mint);
        assert_ne!(got.flagship_mint, Pubkey::default());
        assert_eq!(got.holder_reward_use, HolderRewardUse::FlagshipMintAirdrop);
        assert_eq!(got.acquired_disposition, AcquiredFlagshipDisposition::Burn);
        if spec.split.holder_rewards_bps == 0 {
            assert_eq!(got.holder_rewards, 0);
        }
        if spec.split.flagship_buyback_bps == 0 {
            assert_eq!(got.flagship_buyback, 0);
        }
    }

    #[test]
    fn proposed_curve_fee_numbers_are_unchanged() {
        assert_eq!(crate::beam_economics::TOTAL_FEE_BPS, 50);
        assert_eq!(crate::beam_economics::COMMUNITY_SHARE_BPS, 8_400);
        assert_eq!(crate::beam_economics::TREASURY_SHARE_BPS, 1_000);
        assert_eq!(crate::beam_economics::FOUNDER_SHARE_BPS, 600);
        assert_eq!(
            crate::beam_economics::TOTAL_FEE_BPS * crate::beam_economics::COMMUNITY_SHARE_BPS
                / 10_000,
            u64::from(COMMUNITY_TRADE_BPS)
        );
    }

    #[test]
    fn disabled_default_selects_from_community_and_rejects_arithmetic() {
        let disabled = AllocationModel::disabled();
        assert_eq!(disabled.mode, AllocationMode::Disabled);
        assert_eq!(
            disabled.creator_funding,
            CreatorFunding::FromCommunity { bps_of_trade: None }
        );
        assert_eq!(disabled.flagship_mint, Pubkey::default());
        assert_eq!(
            disabled.holder_reward_use,
            HolderRewardUse::FlagshipMintAirdrop
        );
        assert_eq!(
            disabled.acquired_disposition,
            AcquiredFlagshipDisposition::Burn
        );
        assert_eq!(
            error_code(divide(&disabled, 420).unwrap_err()),
            ERROR_OFFSET
        );
        assert_eq!(
            error_code(divide(&disabled, u64::MAX).unwrap_err()),
            ERROR_OFFSET
        );
    }

    #[test]
    fn unset_creator_bps_and_unset_mint_and_extra_charge_are_rejected() {
        let mut unset_bps = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        unset_bps.creator_funding = CreatorFunding::FromCommunity { bps_of_trade: None };
        assert_eq!(
            error_code(divide(&unset_bps, 10).unwrap_err()),
            ERROR_OFFSET + 1
        );

        let mut unset_mint = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        unset_mint.flagship_mint = Pubkey::default();
        unset_mint.creator_funding = CreatorFunding::FromCommunity { bps_of_trade: None };
        assert_eq!(
            error_code(divide(&unset_mint, 10).unwrap_err()),
            ERROR_OFFSET + 7
        );

        assert_eq!(
            error_code(creator_funding(CreatorFundingRequest::AdditionalCharge).unwrap_err()),
            ERROR_OFFSET + 5
        );
        assert_eq!(
            error_code(
                creator_funding(CreatorFundingRequest::FromCommunity {
                    bps_of_trade: Some(43),
                })
                .unwrap_err()
            ),
            ERROR_OFFSET + 4
        );
        assert!(creator_funding(CreatorFundingRequest::FromCommunity {
            bps_of_trade: Some(42),
        })
        .is_ok());
        let mut too_high = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        too_high.creator_funding = CreatorFunding::FromCommunity {
            bps_of_trade: Some(43),
        };
        assert_eq!(
            error_code(divide(&too_high, 42).unwrap_err()),
            ERROR_OFFSET + 4
        );
    }

    #[test]
    fn flagship_mint_is_the_caller_supplied_pubkey() {
        let mut first = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        let mut second = first;
        first.flagship_mint = key(4);
        second.flagship_mint = key(5);
        assert_ne!(first.flagship_mint, second.flagship_mint);
        assert_eq!(divide(&first, 8).unwrap().flagship_mint, key(4));
        assert_eq!(divide(&second, 8).unwrap().flagship_mint, key(5));
    }

    #[test]
    fn acquired_flagship_tokens_are_burned_and_hold_is_rejected() {
        assert_eq!(
            accept_flagship_disposition(RequestedFlagshipDisposition::Burn).unwrap(),
            AcquiredFlagshipDisposition::Burn
        );
        assert_eq!(
            error_code(
                accept_flagship_disposition(RequestedFlagshipDisposition::Hold).unwrap_err()
            ),
            ERROR_OFFSET + 8
        );
        match AcquiredFlagshipDisposition::Burn {
            AcquiredFlagshipDisposition::Burn => {}
        }
        match HolderRewardUse::FlagshipMintAirdrop {
            HolderRewardUse::FlagshipMintAirdrop => {}
        }
        match FlagshipProceedsUse::Buyback {
            FlagshipProceedsUse::Buyback => {}
        }
    }

    #[test]
    fn shares_must_sum_to_the_denominator() {
        for (holder, flagship) in [
            (0, 0),
            (5_000, 5_001),
            (10_001, 0),
            (6_000, 6_000),
            (u16::MAX, u16::MAX),
        ] {
            let spec = model(
                0,
                holder,
                flagship,
                DustAssignment::RemainderToFlagshipBuyback,
                full_recipients(),
            );
            assert_eq!(
                error_code(divide(&spec, 100).unwrap_err()),
                ERROR_OFFSET + 2,
                "{holder}+{flagship}"
            );
        }
    }

    #[test]
    fn unset_recipients_are_rejected_for_configured_legs() {
        let missing_holder = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            recipients(false, true, false),
        );
        assert_eq!(
            error_code(divide(&missing_holder, 0).unwrap_err()),
            ERROR_OFFSET + 3
        );
        let missing_creator = model(
            1,
            0,
            10_000,
            DustAssignment::RemainderToFlagshipBuyback,
            recipients(false, true, false),
        );
        assert_eq!(
            error_code(divide(&missing_creator, 0).unwrap_err()),
            ERROR_OFFSET + 3
        );
    }

    #[test]
    fn zero_creator_bps_may_omit_the_creator_wallet() {
        let spec = model(
            0,
            0,
            10_000,
            DustAssignment::RemainderToFlagshipBuyback,
            recipients(false, true, false),
        );
        let got = divide(&spec, 25).unwrap();
        assert_eq!(got.creator_lamports, 0);
        assert_eq!(got.holder_rewards, 0);
        assert_eq!(got.flagship_buyback, 25);
    }

    #[test]
    fn creator_bps_are_taken_from_the_community_integer_before_the_split() {
        let spec = model(
            21,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        let got = divide(&spec, 42).unwrap();
        assert_eq!(got.creator_lamports, 21);
        assert_eq!(got.available_lamports, 21);
        assert_eq!(got.holder_rewards, 11);
        assert_eq!(got.flagship_buyback, 10);
        assert_eq!(
            got.creator_lamports + got.holder_rewards + got.flagship_buyback,
            42
        );
        let all = model(
            42,
            5_000,
            5_000,
            DustAssignment::RemainderToFlagshipBuyback,
            full_recipients(),
        );
        let got = divide(&all, 42).unwrap();
        assert_eq!(got.creator_lamports, 42);
        assert_eq!(got.holder_rewards, 0);
        assert_eq!(got.flagship_buyback, 0);
    }

    #[test]
    fn one_lamport_follows_the_configured_dust_leg() {
        let holder_dust = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        let flagship_dust = model(
            0,
            5_000,
            5_000,
            DustAssignment::RemainderToFlagshipBuyback,
            full_recipients(),
        );
        assert_eq!(divide(&holder_dust, 1).unwrap().holder_rewards, 1);
        assert_eq!(divide(&holder_dust, 1).unwrap().flagship_buyback, 0);
        assert_eq!(divide(&flagship_dust, 1).unwrap().holder_rewards, 0);
        assert_eq!(divide(&flagship_dust, 1).unwrap().flagship_buyback, 1);
    }

    #[test]
    fn provisional_even_split_and_100_sol_component_conserve_the_curve_total() {
        let quote = 100_000_000_000;
        let none = AllocationModel::arithmetic_only(
            CreatorFunding::FromCommunity {
                bps_of_trade: Some(0),
            },
            provisional_even_split(DustAssignment::RemainderToHolderRewards),
            full_recipients(),
            key(9),
        );
        let plain = divide_proposed_community_leg(&none, quote).unwrap();
        assert_eq!(
            plain.fee,
            crate::beam_economics::FeeAllocation {
                total: 500_000_000,
                community: 420_000_000,
                treasury: 50_000_000,
                founder: 30_000_000,
            }
        );
        assert_eq!(plain.community.creator_lamports, 0);
        assert_eq!(plain.community.holder_rewards, 210_000_000);
        assert_eq!(plain.community.flagship_buyback, 210_000_000);

        // 21 of 42 is a fixture for half the community integer, not a chosen rate.
        let half = AllocationModel::arithmetic_only(
            CreatorFunding::FromCommunity {
                bps_of_trade: Some(21),
            },
            provisional_even_split(DustAssignment::RemainderToFlagshipBuyback),
            full_recipients(),
            key(9),
        );
        let with_creator = divide_proposed_community_leg(&half, quote).unwrap();
        assert_eq!(with_creator.fee.treasury, 50_000_000);
        assert_eq!(with_creator.fee.founder, 30_000_000);
        assert_eq!(with_creator.community.creator_lamports, 210_000_000);
        assert_eq!(with_creator.community.holder_rewards, 105_000_000);
        assert_eq!(with_creator.community.flagship_buyback, 105_000_000);
        assert_eq!(
            with_creator.fee.total,
            with_creator.fee.treasury
                + with_creator.fee.founder
                + with_creator.community.creator_lamports
                + with_creator.community.holder_rewards
                + with_creator.community.flagship_buyback
        );
    }

    #[test]
    fn economics_composition_conserves_creator_bps_fixtures() {
        for bps in [0_u16, 21, 42] {
            let spec = model(
                bps,
                1,
                9_999,
                DustAssignment::RemainderToFlagshipBuyback,
                full_recipients(),
            );
            for quote in [0, 1, 199, 1_801, 10_000, 100_000_000_000, u64::MAX] {
                let fee = crate::beam_economics::allocate(quote).unwrap();
                let got = divide_proposed_community_leg(&spec, quote).unwrap();
                assert_eq!(got.fee.treasury, fee.treasury);
                assert_eq!(got.fee.founder, fee.founder);
                assert_eq!(
                    got.community.creator_lamports
                        + got.community.holder_rewards
                        + got.community.flagship_buyback,
                    fee.community
                );
                assert_eq!(
                    fee.total,
                    fee.treasury
                        + fee.founder
                        + got.community.creator_lamports
                        + got.community.holder_rewards
                        + got.community.flagship_buyback
                );
            }
        }
    }

    #[test]
    fn sweep_conserves_parameterized_splits_including_extremes() {
        let shares = [
            (5_000, 5_000),
            (1, 9_999),
            (9_999, 1),
            (0, 10_000),
            (10_000, 0),
            (2_500, 7_500),
            (3_333, 6_667),
            (8_400, 1_600),
        ];
        let amounts = [
            0,
            1,
            2,
            9,
            10,
            11,
            42,
            99,
            100,
            999,
            1_801,
            10_000,
            420_000_000,
            u64::MAX - 1,
            u64::MAX,
        ];
        let dust_rules = [
            DustAssignment::RemainderToHolderRewards,
            DustAssignment::RemainderToFlagshipBuyback,
        ];
        for community in amounts {
            for &(holder_bps, flagship_bps) in &shares {
                for dust in dust_rules {
                    for creator_bps in [0_u16, 1, 21, 42] {
                        let spec = model(
                            creator_bps,
                            holder_bps,
                            flagship_bps,
                            dust,
                            recipients(holder_bps > 0, flagship_bps > 0, creator_bps > 0),
                        );
                        assert_division(&spec, community);
                    }
                }
            }
        }
    }

    #[test]
    fn dust_rules_differ_only_by_the_single_remainder_lamport() {
        for community in [0_u64, 1, 21, 42, 100, 420_000_000, u64::MAX] {
            for creator_bps in [0_u16, 21] {
                let holder_dust = model(
                    creator_bps,
                    3_333,
                    6_667,
                    DustAssignment::RemainderToHolderRewards,
                    full_recipients(),
                );
                let flagship_dust = model(
                    creator_bps,
                    3_333,
                    6_667,
                    DustAssignment::RemainderToFlagshipBuyback,
                    full_recipients(),
                );
                let left = divide(&holder_dust, community).unwrap();
                let right = divide(&flagship_dust, community).unwrap();
                assert_eq!(left.rounding_remainder, right.rounding_remainder);
                assert!(left.rounding_remainder <= 1);
                assert_eq!(
                    left.holder_rewards - right.holder_rewards,
                    left.rounding_remainder
                );
                assert_eq!(
                    right.flagship_buyback - left.flagship_buyback,
                    right.rounding_remainder
                );
            }
        }
    }

    #[test]
    fn slop_supply_is_disabled_and_pro_rata_conserves_when_evaluated() {
        assert_eq!(SLOP_CONTRIBUTOR_WHOLE_TOKENS, 10_000_000);
        let mut untouched = [7_u64, 7];
        assert_eq!(
            error_code(divide_slop_contributor_supply(&[1, 1], &mut untouched).unwrap_err()),
            ERROR_OFFSET + 9
        );
        assert_eq!(untouched, [7, 7]);
        assert_eq!(
            error_code(
                divide_supply_by_points(
                    SupplyAllocationMode::Disabled,
                    SLOP_CONTRIBUTOR_WHOLE_TOKENS,
                    &[1, 1],
                    &mut [0, 0],
                )
                .unwrap_err()
            ),
            ERROR_OFFSET + 9
        );

        let fee = crate::beam_economics::allocate(100_000_000_000).unwrap();
        let curve = divide(
            &model(
                0,
                5_000,
                5_000,
                DustAssignment::RemainderToHolderRewards,
                full_recipients(),
            ),
            fee.community,
        )
        .unwrap();
        assert_eq!(curve.holder_rewards + curve.flagship_buyback, fee.community);
        assert_ne!(curve.community_lamports, SLOP_CONTRIBUTOR_WHOLE_TOKENS);

        fn check(allocation: u64, points: &[u64]) {
            let mut out = vec![0_u64; points.len()];
            let got = divide_supply_by_points(
                SupplyAllocationMode::ArithmeticOnly,
                allocation,
                points,
                &mut out,
            )
            .unwrap();
            let total = points
                .iter()
                .fold(0_u128, |sum, item| sum + u128::from(*item));
            let mut assigned = 0_u128;
            for (index, item) in points.iter().enumerate() {
                let expect =
                    u64::try_from(u128::from(allocation) * u128::from(*item) / total).unwrap();
                assert_eq!(out[index], expect);
                assigned += u128::from(out[index]);
            }
            assert_eq!(
                assigned + u128::from(got.unassigned_whole_tokens),
                u128::from(allocation)
            );
            assert_eq!(got.assigned_whole_tokens as u128, assigned);
            assert!(got.unassigned_whole_tokens < points.len() as u64);
        }

        check(SLOP_CONTRIBUTOR_WHOLE_TOKENS, &[1, 1, 1, 1]);
        let mut even = [0_u64; 4];
        divide_supply_by_points(
            SupplyAllocationMode::ArithmeticOnly,
            SLOP_CONTRIBUTOR_WHOLE_TOKENS,
            &[1, 1, 1, 1],
            &mut even,
        )
        .unwrap();
        assert_eq!(even, [2_500_000, 2_500_000, 2_500_000, 2_500_000]);
        check(10, &[1, 2, 3]);
        let mut dust = [0_u64; 3];
        let got = divide_supply_by_points(
            SupplyAllocationMode::ArithmeticOnly,
            10,
            &[1, 2, 3],
            &mut dust,
        )
        .unwrap();
        assert_eq!(dust, [1, 3, 5]);
        assert_eq!(got.unassigned_whole_tokens, 1);
        check(SLOP_CONTRIBUTOR_WHOLE_TOKENS, &[1, 0]);
        check(1, &[1, 1]);
        check(u64::MAX, &[1, 1, 1]);
        assert_eq!(
            error_code(
                divide_supply_by_points(
                    SupplyAllocationMode::ArithmeticOnly,
                    10,
                    &[0, 0],
                    &mut [0, 0]
                )
                .unwrap_err()
            ),
            ERROR_OFFSET + 10
        );
        assert_eq!(
            error_code(
                divide_supply_by_points(
                    SupplyAllocationMode::ArithmeticOnly,
                    10,
                    &[1],
                    &mut [0, 0]
                )
                .unwrap_err()
            ),
            ERROR_OFFSET + 11
        );
    }
}
