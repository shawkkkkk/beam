//! Pure accounting for Beam's bonding curve. No Solana accounts or Raydium CPI
//! are implemented here; this module is intended to be reused by the program.
//!
//! Buy and sell rounding follows the formulas in pumpfun-source's Pump Main
//! program (MIT, copyright 2026 pump.fun contributors), specifically
//! `buy_quote`, `BondingCurve::sell_quote`, `apply_buy`, and `apply_sell`.

pub const BPS_DENOMINATOR: u128 = 10_000;

/// 0.50% of pre-graduation quote volume: 0.40% rewards, 0.07% treasury,
/// 0.03% founder. This does not claim control over Raydium swap fees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeePolicy {
    pub rewards_bps: u16,
    pub treasury_bps: u16,
    pub founder_bps: u16,
}

impl Default for FeePolicy {
    fn default() -> Self {
        Self {
            rewards_bps: 40,
            treasury_bps: 7,
            founder_bps: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeAmounts {
    pub total: u64,
    pub rewards: u64,
    pub treasury: u64,
    pub founder: u64,
}

impl FeePolicy {
    pub fn split(self, quote_amount: u64) -> Result<FeeAmounts, Error> {
        let total_bps = u128::from(self.rewards_bps)
            + u128::from(self.treasury_bps)
            + u128::from(self.founder_bps);
        if total_bps == 0 || total_bps >= BPS_DENOMINATOR {
            return Err(Error::InvalidConfig);
        }
        let total = ceil_div(u128::from(quote_amount) * total_bps, BPS_DENOMINATOR)?;
        let total = to_u64(total)?;
        // Allocate rounding dust to rewards. The three transfers always add
        // up exactly to the fee taken from the trader.
        let treasury = to_u64(u128::from(total) * u128::from(self.treasury_bps) / total_bps)?;
        let founder = to_u64(u128::from(total) * u128::from(self.founder_bps) / total_bps)?;
        let rewards = total
            .checked_sub(treasury + founder)
            .ok_or(Error::Overflow)?;
        Ok(FeeAmounts {
            total,
            rewards,
            treasury,
            founder,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidConfig,
    InvalidAmount,
    Overflow,
    InsufficientTokens,
    InsufficientQuote,
    Slippage,
    CurveComplete,
    CurveIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchConfig {
    pub total_supply: u64,
    pub initial_real_token_reserves: u64,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_quote_reserves: u64,
    pub migration_fee: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BondingCurve {
    pub initial_real_token_reserves: u64,
    pub virtual_token_reserves: u64,
    pub virtual_quote_reserves: u64,
    pub real_token_reserves: u64,
    pub real_quote_reserves: u64,
    pub reserved_pool_tokens: u64,
    pub complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trade {
    /// Token quantity exchanged.
    pub tokens: u64,
    /// Gross quote exchanged with the curve, before fees.
    pub quote: u64,
    pub fees: FeeAmounts,
    /// Trader pays this on buy, receives this on sell.
    pub trader_quote: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationAssets {
    pub pool_tokens: u64,
    pub pool_quote: u64,
    pub migration_fee: u64,
}

impl BondingCurve {
    pub fn new(config: LaunchConfig) -> Result<Self, Error> {
        if config.total_supply <= config.initial_real_token_reserves
            || config.initial_real_token_reserves == 0
            || config.initial_virtual_token_reserves <= config.initial_real_token_reserves
            || config.initial_virtual_quote_reserves == 0
        {
            return Err(Error::InvalidConfig);
        }
        Ok(Self {
            initial_real_token_reserves: config.initial_real_token_reserves,
            virtual_token_reserves: config.initial_virtual_token_reserves,
            virtual_quote_reserves: config.initial_virtual_quote_reserves,
            real_token_reserves: config.initial_real_token_reserves,
            real_quote_reserves: 0,
            reserved_pool_tokens: config.total_supply - config.initial_real_token_reserves,
            complete: false,
        })
    }

    /// Matches Pump Main's integer buy quote: floor(amount * vq / (vt-amount)) + 1.
    pub fn buy_quote(&self, tokens: u64) -> Result<u64, Error> {
        if tokens == 0 {
            return Err(Error::InvalidAmount);
        }
        if self.complete {
            return Err(Error::CurveComplete);
        }
        if tokens > self.real_token_reserves {
            return Err(Error::InsufficientTokens);
        }
        let denominator = self
            .virtual_token_reserves
            .checked_sub(tokens)
            .ok_or(Error::Overflow)?;
        if denominator == 0 {
            return Err(Error::InvalidConfig);
        }
        to_u64(
            u128::from(tokens) * u128::from(self.virtual_quote_reserves) / u128::from(denominator)
                + 1,
        )
    }

    pub fn buy(&mut self, tokens: u64, max_quote: u64, policy: FeePolicy) -> Result<Trade, Error> {
        let quote = self.buy_quote(tokens)?;
        let fees = policy.split(quote)?;
        let trader_quote = quote.checked_add(fees.total).ok_or(Error::Overflow)?;
        if trader_quote > max_quote {
            return Err(Error::Slippage);
        }
        let virtual_quote_reserves = self
            .virtual_quote_reserves
            .checked_add(quote)
            .ok_or(Error::Overflow)?;
        let real_quote_reserves = self
            .real_quote_reserves
            .checked_add(quote)
            .ok_or(Error::Overflow)?;
        self.virtual_token_reserves -= tokens;
        self.real_token_reserves -= tokens;
        self.virtual_quote_reserves = virtual_quote_reserves;
        self.real_quote_reserves = real_quote_reserves;
        self.complete = self.real_token_reserves == 0;
        Ok(Trade {
            tokens,
            quote,
            fees,
            trader_quote,
        })
    }

    /// Matches Pump Main's integer sell quote: floor(vq * amount / (vt+amount)).
    pub fn sell_quote(&self, tokens: u64) -> Result<u64, Error> {
        if tokens == 0 {
            return Err(Error::InvalidAmount);
        }
        if self.complete {
            return Err(Error::CurveComplete);
        }
        if self
            .real_token_reserves
            .checked_add(tokens)
            .ok_or(Error::Overflow)?
            > self.initial_real_token_reserves
        {
            return Err(Error::InsufficientTokens);
        }
        let denominator = self
            .virtual_token_reserves
            .checked_add(tokens)
            .ok_or(Error::Overflow)?;
        let quote = to_u64(
            u128::from(self.virtual_quote_reserves) * u128::from(tokens) / u128::from(denominator),
        )?;
        if quote == 0 {
            return Err(Error::InvalidAmount);
        }
        if quote > self.real_quote_reserves {
            return Err(Error::InsufficientQuote);
        }
        Ok(quote)
    }

    pub fn sell(&mut self, tokens: u64, min_quote: u64, policy: FeePolicy) -> Result<Trade, Error> {
        let quote = self.sell_quote(tokens)?;
        let fees = policy.split(quote)?;
        let trader_quote = quote.checked_sub(fees.total).ok_or(Error::Overflow)?;
        if trader_quote < min_quote {
            return Err(Error::Slippage);
        }
        let virtual_token_reserves = self
            .virtual_token_reserves
            .checked_add(tokens)
            .ok_or(Error::Overflow)?;
        let real_token_reserves = self
            .real_token_reserves
            .checked_add(tokens)
            .ok_or(Error::Overflow)?;
        self.virtual_token_reserves = virtual_token_reserves;
        self.real_token_reserves = real_token_reserves;
        self.virtual_quote_reserves -= quote;
        self.real_quote_reserves -= quote;
        Ok(Trade {
            tokens,
            quote,
            fees,
            trader_quote,
        })
    }

    /// Amounts available for a future Raydium pool CPI. Caller must verify
    /// the actual token balance and native reserve before moving assets.
    pub fn migration_assets(&self, migration_fee: u64) -> Result<MigrationAssets, Error> {
        if !self.complete {
            return Err(Error::CurveIncomplete);
        }
        let pool_quote = self
            .real_quote_reserves
            .checked_sub(migration_fee)
            .ok_or(Error::InsufficientQuote)?;
        if pool_quote == 0 {
            return Err(Error::InsufficientQuote);
        }
        Ok(MigrationAssets {
            pool_tokens: self.reserved_pool_tokens,
            pool_quote,
            migration_fee,
        })
    }
}

fn ceil_div(numerator: u128, denominator: u128) -> Result<u128, Error> {
    numerator
        .checked_add(denominator - 1)
        .ok_or(Error::Overflow)
        .map(|value| value / denominator)
}

fn to_u64(value: u128) -> Result<u64, Error> {
    u64::try_from(value).map_err(|_| Error::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> LaunchConfig {
        LaunchConfig {
            total_supply: 1_000_000,
            initial_real_token_reserves: 800_000,
            initial_virtual_token_reserves: 1_073_000,
            initial_virtual_quote_reserves: 30_000,
            migration_fee: 1_000,
        }
    }

    #[test]
    fn buy_and_sell_match_reference_rounding_and_reserves() {
        let mut curve = BondingCurve::new(config()).unwrap();
        let initial = curve;
        let buy = curve.buy(100_000, u64::MAX, FeePolicy::default()).unwrap();
        assert_eq!(buy.quote, 100_000 * 30_000 / 973_000 + 1);
        assert_eq!(buy.trader_quote, buy.quote + buy.fees.total);
        assert_eq!(curve.virtual_token_reserves, 973_000);
        assert_eq!(curve.real_token_reserves, 700_000);
        assert_eq!(curve.real_quote_reserves, buy.quote);
        let sell = curve.sell(50_000, 0, FeePolicy::default()).unwrap();
        assert_eq!(
            sell.quote,
            (u128::from(initial.virtual_quote_reserves + buy.quote) * 50_000 / 1_023_000) as u64
        );
        assert_eq!(sell.trader_quote, sell.quote - sell.fees.total);
        assert_eq!(curve.real_token_reserves, 750_000);
        assert_eq!(curve.real_quote_reserves, buy.quote - sell.quote);
    }

    #[test]
    fn fees_reconcile_including_rounding_dust() {
        let policy = FeePolicy::default();
        for quote in [0, 1, 199, 200, 10_000, u64::MAX] {
            let fee = policy.split(quote).unwrap();
            assert_eq!(fee.rewards + fee.treasury + fee.founder, fee.total);
            assert_eq!(fee.total, (u128::from(quote) * 50).div_ceil(10_000) as u64);
        }
        assert_eq!(
            policy.split(10_000).unwrap(),
            FeeAmounts {
                total: 50,
                rewards: 40,
                treasury: 7,
                founder: 3,
            }
        );
    }

    #[test]
    fn slippage_and_invalid_trades_leave_state_unchanged() {
        let mut curve = BondingCurve::new(config()).unwrap();
        let before = curve;
        assert_eq!(
            curve.buy(10_000, 1, FeePolicy::default()),
            Err(Error::Slippage)
        );
        assert_eq!(
            curve.buy(800_001, u64::MAX, FeePolicy::default()),
            Err(Error::InsufficientTokens)
        );
        assert_eq!(
            curve.sell(1, 0, FeePolicy::default()),
            Err(Error::InsufficientTokens)
        );
        assert_eq!(curve, before);
        curve.buy(100_000, u64::MAX, FeePolicy::default()).unwrap();
        let before = curve;
        assert_eq!(
            curve.sell(50_000, u64::MAX, FeePolicy::default()),
            Err(Error::Slippage)
        );
        assert_eq!(curve, before);
    }

    #[test]
    fn completion_gates_pool_assets_and_never_spends_more_than_reserves() {
        let mut curve = BondingCurve::new(config()).unwrap();
        assert_eq!(curve.migration_assets(0), Err(Error::CurveIncomplete));
        curve.buy(800_000, u64::MAX, FeePolicy::default()).unwrap();
        assert!(curve.complete);
        assert_eq!(curve.buy_quote(1), Err(Error::CurveComplete));
        let assets = curve.migration_assets(config().migration_fee).unwrap();
        assert_eq!(assets.pool_tokens, 200_000);
        assert_eq!(
            assets.pool_quote,
            curve.real_quote_reserves - config().migration_fee
        );
        assert_eq!(
            curve.migration_assets(curve.real_quote_reserves),
            Err(Error::InsufficientQuote)
        );
        assert_eq!(curve.sell_quote(1), Err(Error::CurveComplete));
    }

    #[test]
    fn config_and_overflow_are_checked() {
        let mut invalid = config();
        invalid.initial_virtual_token_reserves = invalid.initial_real_token_reserves;
        assert_eq!(BondingCurve::new(invalid), Err(Error::InvalidConfig));
        assert_eq!(
            FeePolicy {
                rewards_bps: 0,
                treasury_bps: 0,
                founder_bps: 0
            }
            .split(1),
            Err(Error::InvalidConfig)
        );
        let mut curve = BondingCurve::new(config()).unwrap();
        curve.virtual_quote_reserves = u64::MAX;
        let before = curve;
        assert_eq!(
            curve.buy(1, u64::MAX, FeePolicy::default()),
            Err(Error::Overflow)
        );
        assert_eq!(curve, before);
    }
}
