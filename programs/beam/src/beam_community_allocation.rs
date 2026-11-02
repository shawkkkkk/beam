//! Disabled community-allocation model for holder rewards and flagship buybacks.
//!
//! `beam_economics::allocate` proposes a 42/5/3 bps community/treasury/founder
//! split of one rounded Beam component. This module divides only that
//! community integer. Treasury and founder lamports stay outside the split.
//!
//! The default model is disabled. `CreatorFunding::Undecided` stays rejected
//! until a caller selects a config option. When the option is
//! `CommunityRemainder`, an exact creator amount is removed first and the
//! holder/flagship parameters apply only to what remains. When the option is
//! `SeparateCharge`, the whole community integer is divided and a creator
//! deduction is rejected.
//!
//! Share parameters are parts of the available community amount per 10_000.
//! They are a different scale from the 42/5/3 trade-component basis points.
//! `PROVISIONAL_AVAILABLE_SHARE_BPS` (5_000 and 5_000) is a test setting.
//!
//! # Rounding
//!
//! Let `D = 10_000` and let `h + f = D`. For an available amount `a`:
//!
//! ```text
//! H = floor(a * h / D)
//! F = floor(a * f / D)
//! ```
//!
//! Because `a * h / D + a * f / D = a`, the fractional parts sum to an integer
//! in `{0, 1}`. Therefore `a - H - F` is 0 or 1. `RemainderToFlagshipBuyback`
//! pays `H` to holder rewards and `a - H` to the flagship leg.
//! `RemainderToHolderRewards` pays `F` to the flagship leg and `a - F` to
//! holder rewards. Every available lamport is assigned to one of those two
//! legs. A leg configured at 0 share receives 0, including the rounding
//! lamport, because `floor(a * 10_000 / 10_000) = a`.
//!
//! # Conservation
//!
//! `SeparateCharge`: `community = holder_rewards + flagship_buyback`.
//!
//! `CommunityRemainder`, with `creator <= community`:
//! `community = creator + holder_rewards + flagship_buyback`.
//!
//! Composed with the proposed Beam component:
//! `total = treasury + founder + community`, and `community` expands as above.
//! Creator shortfall is rejected. This module does not take lamports from
//! treasury or founder.
//!
//! Arithmetic uses `u128` intermediates. For `a <= u64::MAX` and `h <= 10_000`,
//! `a * h` fits in `u128`, and `floor(a * h / D) <= a`.
//!
//! This module has no instruction, transfer, keeper, or market-order entrypoint.
//! Evaluating it records no claim to future fees, price, or execution.
use anchor_lang::prelude::*;

/// Parts of the available community amount. The two shares sum to this value.
pub const SPLIT_BPS_DENOMINATOR: u32 = 10_000;

/// Provisional test setting: half of the available community amount.
/// This is 5_000 / 10_000 of that amount, which is a different scale from the
/// 50 bps Beam component.
pub const PROVISIONAL_AVAILABLE_SHARE_BPS: u16 = 5_000;

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

/// Config option for the creator payout. The default does not select either
/// funding route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatorFunding {
    /// [`divide`] rejects the model until a route is selected.
    Undecided,
    /// The creator payout stays outside this allocation.
    SeparateCharge,
    /// Remove the creator payout from the community amount, then divide the remainder.
    CommunityRemainder,
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

/// Wallets that would be credited if a later, separate design collects funds.
/// The all-zero pubkey is unset. A positive configured share requires a set
/// recipient. A positive creator deduction requires a set creator wallet.
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunityDivision {
    pub community_lamports: u64,
    pub creator_lamports: u64,
    pub available_lamports: u64,
    pub holder_rewards: u64,
    pub flagship_buyback: u64,
    /// `available_lamports - floor(holder share) - floor(flagship share)`.
    /// Always 0 or 1 for a valid split, and included in the configured leg.
    pub rounding_remainder: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProposedCommunityLeg {
    pub fee: crate::beam_economics::FeeAllocation,
    pub community: CommunityDivision,
}

#[error_code(offset = 7200)]
pub enum CommunityAllocationError {
    #[msg("Community allocation is disabled")]
    Disabled,
    #[msg("Creator funding source is unset")]
    CreatorFundingUnset,
    #[msg("Holder and flagship shares must sum to 10000")]
    InvalidSplit,
    #[msg("A configured community recipient is unset")]
    UnsetRecipient,
    #[msg("Creator payout exceeds the community allocation")]
    CreatorExceedsCommunity,
    #[msg("Creator lamports are outside a separate-charge community split")]
    CreatorNotInCommunityAllocation,
    #[msg("Community allocation arithmetic overflowed")]
    Overflow,
}

impl AllocationModel {
    /// Disabled, with creator funding left unset and both shares at zero.
    /// The dust field is inert until [`AllocationMode::ArithmeticOnly`].
    pub fn disabled() -> Self {
        Self {
            mode: AllocationMode::Disabled,
            creator_funding: CreatorFunding::Undecided,
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
        }
    }

    /// Parameters that [`divide`] may evaluate. Passing
    /// [`CreatorFunding::Undecided`] still fails closed inside `divide`.
    pub fn arithmetic_only(
        creator_funding: CreatorFunding,
        split: SplitParameters,
        recipients: Recipients,
    ) -> Self {
        Self {
            mode: AllocationMode::ArithmeticOnly,
            creator_funding,
            split,
            recipients,
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
///
/// `creator_lamports` is an exact amount supplied by the caller. This function
/// calculates no creator fee schedule of its own.
pub fn divide(
    model: &AllocationModel,
    community_lamports: u64,
    creator_lamports: u64,
) -> Result<CommunityDivision> {
    if model.mode == AllocationMode::Disabled {
        return err!(CommunityAllocationError::Disabled);
    }
    let (available, creator_taken) =
        available_after_creator(model.creator_funding, community_lamports, creator_lamports)?;
    validate_split(model.split)?;
    require_recipients(model, creator_taken)?;
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
        creator_lamports: creator_taken,
        available_lamports: available,
        holder_rewards,
        flagship_buyback,
        rounding_remainder,
    })
}

/// Divide the community integer proposed by [`crate::beam_economics::allocate`].
/// Treasury and founder values are returned unchanged.
pub fn divide_proposed_community_leg(
    model: &AllocationModel,
    quote_amount: u64,
    creator_lamports: u64,
) -> Result<ProposedCommunityLeg> {
    let fee = crate::beam_economics::allocate(quote_amount)?;
    let community = divide(model, fee.community, creator_lamports)?;
    require!(
        community.community_lamports == fee.community,
        CommunityAllocationError::Overflow
    );
    Ok(ProposedCommunityLeg { fee, community })
}

fn available_after_creator(
    funding: CreatorFunding,
    community_lamports: u64,
    creator_lamports: u64,
) -> Result<(u64, u64)> {
    match funding {
        CreatorFunding::Undecided => err!(CommunityAllocationError::CreatorFundingUnset),
        CreatorFunding::SeparateCharge => {
            require!(
                creator_lamports == 0,
                CommunityAllocationError::CreatorNotInCommunityAllocation
            );
            Ok((community_lamports, 0))
        }
        CreatorFunding::CommunityRemainder => {
            let available = community_lamports
                .checked_sub(creator_lamports)
                .ok_or_else(|| error!(CommunityAllocationError::CreatorExceedsCommunity))?;
            Ok((available, creator_lamports))
        }
    }
}

fn validate_split(split: SplitParameters) -> Result<()> {
    let sum = u32::from(split.holder_rewards_bps) + u32::from(split.flagship_buyback_bps);
    require!(
        sum == SPLIT_BPS_DENOMINATOR,
        CommunityAllocationError::InvalidSplit
    );
    Ok(())
}

fn require_recipients(model: &AllocationModel, creator_taken: u64) -> Result<()> {
    if creator_taken > 0 {
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
        funding: CreatorFunding,
        holder_bps: u16,
        flagship_bps: u16,
        dust: DustAssignment,
        recipients: Recipients,
    ) -> AllocationModel {
        AllocationModel::arithmetic_only(
            funding,
            SplitParameters {
                holder_rewards_bps: holder_bps,
                flagship_buyback_bps: flagship_bps,
                dust,
            },
            recipients,
        )
    }

    /// Independent of `floor_share`: `amount = q*D + r` implies
    /// `floor(amount * bps / D) = q*bps + floor(r * bps / D)`.
    fn floor_bps(amount: u64, bps: u16) -> u64 {
        let whole = (u128::from(amount) / 10_000) * u128::from(bps);
        let remainder = (u128::from(amount % 10_000) * u128::from(bps)) / 10_000;
        u64::try_from(whole + remainder).unwrap()
    }

    fn assert_division(spec: &AllocationModel, community: u64, creator: u64) {
        let got = divide(spec, community, creator).unwrap_or_else(|error| {
            panic!("divide failed for community={community} creator={creator} model={spec:?}: {error:?}")
        });
        let (available, creator_taken) = match spec.creator_funding {
            CreatorFunding::CommunityRemainder => (community - creator, creator),
            CreatorFunding::SeparateCharge => (community, 0),
            CreatorFunding::Undecided => unreachable!("undecided models are rejected"),
        };
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
        assert_eq!(got.creator_lamports, creator_taken);
        assert_eq!(got.available_lamports, available);
        assert_eq!(got.holder_rewards, holder, "{spec:?} community={community}");
        assert_eq!(
            got.flagship_buyback, flagship,
            "{spec:?} community={community}"
        );
        assert_eq!(got.rounding_remainder, rounding);
        assert_eq!(
            got.creator_lamports + got.holder_rewards + got.flagship_buyback,
            community
        );
        assert_eq!(got.holder_rewards + got.flagship_buyback, available);
        match spec.split.dust {
            DustAssignment::RemainderToHolderRewards => {
                assert_eq!(got.flagship_buyback, flagship_floor);
                assert_eq!(got.holder_rewards, holder_floor + rounding);
            }
            DustAssignment::RemainderToFlagshipBuyback => {
                assert_eq!(got.holder_rewards, holder_floor);
                assert_eq!(got.flagship_buyback, flagship_floor + rounding);
            }
        }
        if spec.split.holder_rewards_bps == 0 {
            assert_eq!(got.holder_rewards, 0);
        }
        if spec.split.flagship_buyback_bps == 0 {
            assert_eq!(got.flagship_buyback, 0);
        }
    }

    #[test]
    fn disabled_default_rejects_before_any_split() {
        let disabled = AllocationModel::disabled();
        assert_eq!(disabled.mode, AllocationMode::Disabled);
        assert_eq!(disabled.creator_funding, CreatorFunding::Undecided);
        assert_eq!(disabled.split.holder_rewards_bps, 0);
        assert_eq!(
            error_code(divide(&disabled, 420, 0).unwrap_err()),
            ERROR_OFFSET
        );
        assert_eq!(
            error_code(divide(&disabled, u64::MAX, u64::MAX).unwrap_err()),
            ERROR_OFFSET
        );
        let mut configured = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        configured.mode = AllocationMode::Disabled;
        assert_eq!(
            error_code(divide(&configured, 10, 0).unwrap_err()),
            ERROR_OFFSET
        );
    }

    #[test]
    fn undecided_funding_is_rejected_before_the_split_is_judged() {
        let undecided = model(
            CreatorFunding::Undecided,
            0,
            0,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        assert_eq!(
            error_code(divide(&undecided, 10, 0).unwrap_err()),
            ERROR_OFFSET + 1
        );
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
                CreatorFunding::SeparateCharge,
                holder,
                flagship,
                DustAssignment::RemainderToFlagshipBuyback,
                full_recipients(),
            );
            assert_eq!(
                error_code(divide(&spec, 100, 0).unwrap_err()),
                ERROR_OFFSET + 2,
                "{holder}+{flagship}"
            );
        }
    }

    #[test]
    fn unset_recipients_are_rejected_for_configured_legs() {
        let missing_holder = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            recipients(false, true, false),
        );
        assert_eq!(
            error_code(divide(&missing_holder, 0, 0).unwrap_err()),
            ERROR_OFFSET + 3
        );

        let missing_flagship = model(
            CreatorFunding::CommunityRemainder,
            1,
            9_999,
            DustAssignment::RemainderToFlagshipBuyback,
            recipients(true, false, true),
        );
        assert_eq!(
            error_code(divide(&missing_flagship, 0, 0).unwrap_err()),
            ERROR_OFFSET + 3
        );

        let missing_creator = model(
            CreatorFunding::CommunityRemainder,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            recipients(true, true, false),
        );
        assert_eq!(
            error_code(divide(&missing_creator, 5, 1).unwrap_err()),
            ERROR_OFFSET + 3
        );
    }

    #[test]
    fn zero_creator_may_omit_creator_wallet_and_zero_share_may_omit_its_wallet() {
        let spec = model(
            CreatorFunding::CommunityRemainder,
            0,
            10_000,
            DustAssignment::RemainderToFlagshipBuyback,
            recipients(false, true, false),
        );
        let got = divide(&spec, 25, 0).unwrap();
        assert_eq!(got.holder_rewards, 0);
        assert_eq!(got.flagship_buyback, 25);
        assert_eq!(got.creator_lamports, 0);
        assert_eq!(got.rounding_remainder, 0);
    }

    #[test]
    fn separate_charge_rejects_a_creator_deduction() {
        let spec = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        assert_eq!(
            error_code(divide(&spec, 10, 1).unwrap_err()),
            ERROR_OFFSET + 5
        );
        assert_division(&spec, 10, 0);
    }

    #[test]
    fn creator_above_the_community_amount_is_rejected() {
        let spec = model(
            CreatorFunding::CommunityRemainder,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        assert_eq!(
            error_code(divide(&spec, 10, 11).unwrap_err()),
            ERROR_OFFSET + 4
        );
        assert_eq!(
            error_code(divide(&spec, 0, 1).unwrap_err()),
            ERROR_OFFSET + 4
        );
        let fee = crate::beam_economics::allocate(100_000_000_000).unwrap();
        assert!(fee.treasury > 0 && fee.founder > 0);
        assert_eq!(
            error_code(divide(&spec, fee.community, fee.community + 1).unwrap_err()),
            ERROR_OFFSET + 4
        );
    }

    #[test]
    fn creator_first_remainder_is_divided_after_the_exact_deduction() {
        let dust = DustAssignment::RemainderToHolderRewards;
        let spec = model(
            CreatorFunding::CommunityRemainder,
            5_000,
            5_000,
            dust,
            full_recipients(),
        );
        let got = divide(&spec, 10, 4).unwrap();
        // Available amount is 6. An even split of the gross 10, followed by
        // taking 4 from the holder leg, would leave holder 1 and flagship 5.
        assert_eq!(
            got,
            CommunityDivision {
                community_lamports: 10,
                creator_lamports: 4,
                available_lamports: 6,
                holder_rewards: 3,
                flagship_buyback: 3,
                rounding_remainder: 0,
            }
        );

        let uneven = model(
            CreatorFunding::CommunityRemainder,
            2_500,
            7_500,
            DustAssignment::RemainderToFlagshipBuyback,
            full_recipients(),
        );
        let got = divide(&uneven, 100, 10).unwrap();
        assert_eq!(got.available_lamports, 90);
        assert_eq!(got.holder_rewards, 22);
        assert_eq!(got.flagship_buyback, 68);
        assert_eq!(got.rounding_remainder, 1);
        assert_eq!(
            got.creator_lamports + got.holder_rewards + got.flagship_buyback,
            100
        );
    }

    #[test]
    fn one_lamport_follows_the_configured_dust_leg() {
        let holder_dust = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            full_recipients(),
        );
        let flagship_dust = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToFlagshipBuyback,
            full_recipients(),
        );
        assert_eq!(divide(&holder_dust, 1, 0).unwrap().holder_rewards, 1);
        assert_eq!(divide(&holder_dust, 1, 0).unwrap().flagship_buyback, 0);
        assert_eq!(divide(&flagship_dust, 1, 0).unwrap().holder_rewards, 0);
        assert_eq!(divide(&flagship_dust, 1, 0).unwrap().flagship_buyback, 1);
        assert_eq!(divide(&holder_dust, 0, 0).unwrap().rounding_remainder, 0);
    }

    #[test]
    fn same_recipient_pubkey_is_currently_accepted() {
        let mut recipients = full_recipients();
        recipients.flagship_buyback = recipients.holder_rewards;
        let spec = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            recipients,
        );
        let got = divide(&spec, 9, 0).unwrap();
        assert_eq!(got.holder_rewards + got.flagship_buyback, 9);
    }

    #[test]
    fn provisional_even_split_matches_half_the_available_amount() {
        assert_eq!(PROVISIONAL_AVAILABLE_SHARE_BPS, 5_000);
        let split = provisional_even_split(DustAssignment::RemainderToHolderRewards);
        assert_eq!(
            u32::from(split.holder_rewards_bps) + u32::from(split.flagship_buyback_bps),
            SPLIT_BPS_DENOMINATOR
        );
        let spec = AllocationModel::arithmetic_only(
            CreatorFunding::CommunityRemainder,
            split,
            full_recipients(),
        );
        assert_division(&spec, 420_000_000, 0);
        let got = divide(&spec, 420_000_000, 0).unwrap();
        assert_eq!(got.holder_rewards, 210_000_000);
        assert_eq!(got.flagship_buyback, 210_000_000);
        assert_eq!(got.rounding_remainder, 0);
    }

    #[test]
    fn proposed_100_sol_component_conserves_every_leg() {
        let quote = 100_000_000_000;
        let separate = AllocationModel::arithmetic_only(
            CreatorFunding::SeparateCharge,
            provisional_even_split(DustAssignment::RemainderToHolderRewards),
            full_recipients(),
        );
        let plain = divide_proposed_community_leg(&separate, quote, 0).unwrap();
        assert_eq!(
            plain.fee,
            crate::beam_economics::FeeAllocation {
                total: 500_000_000,
                community: 420_000_000,
                treasury: 50_000_000,
                founder: 30_000_000,
            }
        );
        assert_eq!(plain.community.holder_rewards, 210_000_000);
        assert_eq!(plain.community.flagship_buyback, 210_000_000);
        assert_eq!(
            plain.fee.total,
            plain.fee.treasury
                + plain.fee.founder
                + plain.community.holder_rewards
                + plain.community.flagship_buyback
        );

        let from_community = AllocationModel::arithmetic_only(
            CreatorFunding::CommunityRemainder,
            provisional_even_split(DustAssignment::RemainderToFlagshipBuyback),
            full_recipients(),
        );
        let creator = 20_000_000;
        let with_creator = divide_proposed_community_leg(&from_community, quote, creator).unwrap();
        assert_eq!(with_creator.fee.treasury, plain.fee.treasury);
        assert_eq!(with_creator.fee.founder, plain.fee.founder);
        assert_eq!(with_creator.community.creator_lamports, creator);
        assert_eq!(with_creator.community.available_lamports, 400_000_000);
        assert_eq!(with_creator.community.holder_rewards, 200_000_000);
        assert_eq!(with_creator.community.flagship_buyback, 200_000_000);
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
    fn economics_composition_conserves_dust_quotes_and_bounds() {
        let separate = AllocationModel::arithmetic_only(
            CreatorFunding::SeparateCharge,
            provisional_even_split(DustAssignment::RemainderToHolderRewards),
            full_recipients(),
        );
        let remainder = AllocationModel::arithmetic_only(
            CreatorFunding::CommunityRemainder,
            SplitParameters {
                holder_rewards_bps: 1,
                flagship_buyback_bps: 9_999,
                dust: DustAssignment::RemainderToFlagshipBuyback,
            },
            full_recipients(),
        );
        for quote in [
            0,
            1,
            199,
            200,
            201,
            1_801,
            3_201,
            9_999,
            10_000,
            100_000_000_000,
            u64::MAX,
        ] {
            let fee = crate::beam_economics::allocate(quote).unwrap();
            assert_eq!(fee.total, fee.community + fee.treasury + fee.founder);
            let plain = divide_proposed_community_leg(&separate, quote, 0).unwrap();
            assert_eq!(plain.fee.community, fee.community);
            assert_eq!(plain.fee.treasury, fee.treasury);
            assert_eq!(plain.fee.founder, fee.founder);
            assert_eq!(
                plain.community.holder_rewards + plain.community.flagship_buyback,
                fee.community
            );
            for creator in [0, fee.community / 2, fee.community] {
                let got = divide_proposed_community_leg(&remainder, quote, creator).unwrap();
                assert_eq!(got.fee.treasury, fee.treasury);
                assert_eq!(got.fee.founder, fee.founder);
                assert_eq!(
                    got.community.creator_lamports
                        + got.community.holder_rewards
                        + got.community.flagship_buyback,
                    fee.community
                );
                assert!(got.community.rounding_remainder <= 1);
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
            3,
            9,
            10,
            11,
            49,
            50,
            51,
            99,
            100,
            101,
            999,
            1_000,
            1_801,
            3_201,
            9_999,
            10_000,
            420_000_000,
            1_000_000_000,
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
                    let needs_holder = holder_bps > 0;
                    let needs_flagship = flagship_bps > 0;
                    let separate = model(
                        CreatorFunding::SeparateCharge,
                        holder_bps,
                        flagship_bps,
                        dust,
                        recipients(needs_holder, needs_flagship, false),
                    );
                    assert_division(&separate, community, 0);

                    for creator in [0_u64, community.min(1), community / 2, community] {
                        let wallets = if creator == 0 {
                            recipients(needs_holder, needs_flagship, false)
                        } else {
                            recipients(needs_holder, needs_flagship, true)
                        };
                        let spec = model(
                            CreatorFunding::CommunityRemainder,
                            holder_bps,
                            flagship_bps,
                            dust,
                            wallets,
                        );
                        assert_division(&spec, community, creator);
                    }
                }
            }
        }
    }

    #[test]
    fn dust_rules_differ_only_by_the_single_remainder_lamport() {
        let amounts = [
            0_u64,
            1,
            2,
            7,
            10,
            11,
            99,
            100,
            1_801,
            420_000_000,
            u64::MAX,
        ];
        for community in amounts {
            for creator in [0, community / 3, community] {
                let available = community - creator;
                let holder_dust = model(
                    CreatorFunding::CommunityRemainder,
                    3_333,
                    6_667,
                    DustAssignment::RemainderToHolderRewards,
                    full_recipients(),
                );
                let flagship_dust = model(
                    CreatorFunding::CommunityRemainder,
                    3_333,
                    6_667,
                    DustAssignment::RemainderToFlagshipBuyback,
                    full_recipients(),
                );
                let left = divide(&holder_dust, community, creator).unwrap();
                let right = divide(&flagship_dust, community, creator).unwrap();
                assert_eq!(left.rounding_remainder, right.rounding_remainder);
                assert!(left.rounding_remainder <= 1);
                assert_eq!(
                    left.holder_rewards - right.holder_rewards,
                    left.rounding_remainder
                );
                assert_eq!(
                    right.flagship_buyback - left.flagship_buyback,
                    left.rounding_remainder
                );
                assert_eq!(left.holder_rewards + left.flagship_buyback, available);
                if available % 2 == 0 && left.rounding_remainder == 0 {
                    assert_eq!(left.holder_rewards, right.holder_rewards);
                }
            }
        }
    }

    #[test]
    fn full_creator_deduction_leaves_both_uses_at_zero() {
        let spec = model(
            CreatorFunding::CommunityRemainder,
            5_000,
            5_000,
            DustAssignment::RemainderToFlagshipBuyback,
            full_recipients(),
        );
        let got = divide(&spec, 17, 17).unwrap();
        assert_eq!(got.available_lamports, 0);
        assert_eq!(got.holder_rewards, 0);
        assert_eq!(got.flagship_buyback, 0);
        assert_eq!(got.creator_lamports, 17);
    }

    #[test]
    fn odd_maximum_amount_assigns_one_rounding_lamport() {
        let spec = model(
            CreatorFunding::SeparateCharge,
            5_000,
            5_000,
            DustAssignment::RemainderToHolderRewards,
            recipients(true, true, false),
        );
        let got = divide(&spec, u64::MAX, 0).unwrap();
        assert_eq!(got.rounding_remainder, 1);
        assert_eq!(got.holder_rewards, (u64::MAX / 2) + 1);
        assert_eq!(got.flagship_buyback, u64::MAX / 2);
        assert_eq!(got.holder_rewards + got.flagship_buyback, u64::MAX);
    }
}
