//! Proposed Beam fee policy. This allocation engine is not yet wired into the
//! recovered Pump fee-program CPI or a post-graduation keeper.
use anchor_lang::prelude::*;
use crate::PumpError;

pub const TOTAL_FEE_BPS: u64 = 50;
pub const COMMUNITY_SHARE_BPS: u64 = 9_000;
pub const PLATFORM_SHARE_BPS: u64 = 1_000;

#[derive(Debug, PartialEq, Eq)]
pub struct FeeAllocation {
    pub total: u64,
    pub community: u64,
    pub platform: u64,
}

/// Round the total once using Pump's recovered fee routine. Assign split dust
/// to the community, never round two separate charges up against the trader.
pub fn allocate(quote_amount: u64) -> Result<FeeAllocation> {
    let total = crate::fee_math::fee(quote_amount, TOTAL_FEE_BPS)?;
    let platform = u64::try_from(u128::from(total) * u128::from(PLATFORM_SHARE_BPS) / 10_000)
        .map_err(|_| error!(PumpError::Truncation))?;
    Ok(FeeAllocation { total, community: total - platform, platform })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_hundred_sol_trade_assigns_point_four_five_to_community_point_zero_five_to_beam() {
        assert_eq!(allocate(100_000_000_000).unwrap(), FeeAllocation {
            total: 500_000_000, community: 450_000_000, platform: 50_000_000,
        });
    }
    #[test]
    fn dust_favors_community_without_a_second_rounded_charge() {
        assert_eq!(allocate(1).unwrap(), FeeAllocation { total: 1, community: 1, platform: 0 });
        assert_eq!(allocate(10_000).unwrap(), FeeAllocation { total: 50, community: 45, platform: 5 });
        assert_eq!(allocate(0).unwrap(), FeeAllocation { total: 0, community: 0, platform: 0 });
    }
    #[test]
    fn boundary_allocations_conserve_every_lamport() {
        for amount in [0,1,199,200,201,9999,10_000,1_000_000_000,u64::MAX] {
            let a = allocate(amount).unwrap();
            assert_eq!(a.total, a.community + a.platform);
            assert!(u128::from(a.platform) * 10 <= u128::from(a.total));
            assert!(u128::from(a.community) * 10 >= u128::from(a.total) * 9);
        }
    }
}
