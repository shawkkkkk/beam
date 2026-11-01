//! Provisional Beam curve-component fee policy. This does not replace or
//! calculate retained Pump creator fees and is not wired into trade collection.
use anchor_lang::prelude::*;
use crate::PumpError;

pub const TOTAL_FEE_BPS: u64 = 50;
pub const COMMUNITY_NOMINAL_BPS: u64 = 42;
pub const TREASURY_NOMINAL_BPS: u64 = 5;
pub const FOUNDER_NOMINAL_BPS: u64 = 3;

#[derive(Debug, PartialEq, Eq)]
pub struct FeeAllocation {
    pub total: u64,
    pub community: u64,
    pub treasury: u64,
    pub founder: u64,
}

/// Round the total once using Pump's recovered fee routine. Cap the treasury
/// and founder against the trade notional itself, then assign split dust to
/// the community. Never round two separate charges up against the trader.
pub fn allocate(quote_amount: u64) -> Result<FeeAllocation> {
    let total = crate::fee_math::fee(quote_amount, TOTAL_FEE_BPS)?;
    let founder = u64::try_from(u128::from(quote_amount) * u128::from(FOUNDER_NOMINAL_BPS) / 10_000)
        .map_err(|_| error!(PumpError::Truncation))?;
    let treasury = u64::try_from(u128::from(quote_amount) * u128::from(TREASURY_NOMINAL_BPS) / 10_000)
        .map_err(|_| error!(PumpError::Truncation))?;
    let community = total.checked_sub(founder).and_then(|v| v.checked_sub(treasury))
        .ok_or_else(|| error!(PumpError::Overflow))?;
    Ok(FeeAllocation { total, community, treasury, founder })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_hundred_sol_trade_assigns_point_four_two_community_point_zero_five_treasury_point_zero_three_founder() {
        assert_eq!(allocate(100_000_000_000).unwrap(), FeeAllocation {
            total: 500_000_000, community: 420_000_000, treasury: 50_000_000, founder: 30_000_000,
        });
    }
    #[test]
    fn dust_favors_community_without_a_second_rounded_charge() {
        assert_eq!(allocate(1).unwrap(), FeeAllocation { total: 1, community: 1, treasury: 0, founder: 0 });
        // Splitting the rounded total by 10%/6% would exceed 5/3 bps of
        // these trade notionals by one lamport.
        assert_eq!(allocate(1_801).unwrap(), FeeAllocation { total: 10, community: 10, treasury: 0, founder: 0 });
        assert_eq!(allocate(3_201).unwrap(), FeeAllocation { total: 17, community: 16, treasury: 1, founder: 0 });
        assert_eq!(allocate(10_000).unwrap(), FeeAllocation { total: 50, community: 42, treasury: 5, founder: 3 });
        assert_eq!(allocate(0).unwrap(), FeeAllocation { total: 0, community: 0, treasury: 0, founder: 0 });
    }
    #[test]
    fn boundary_allocations_conserve_every_lamport() {
        for amount in [0,1,199,200,201,9999,10_000,1_000_000_000,u64::MAX] {
            let a = allocate(amount).unwrap();
            assert_eq!(a.total, a.community + a.treasury + a.founder);
            assert!(u128::from(a.founder) * 10_000 <= u128::from(amount) * FOUNDER_NOMINAL_BPS as u128);
            assert!(u128::from(a.treasury) * 10_000 <= u128::from(amount) * TREASURY_NOMINAL_BPS as u128);
            assert!(u128::from(a.community) * 10_000 >= u128::from(amount) * COMMUNITY_NOMINAL_BPS as u128);
        }
    }
}
