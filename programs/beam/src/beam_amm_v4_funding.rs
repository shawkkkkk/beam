//! Disabled, source-only funding helper for native-SOL graduation of a
//! classic-SPL mint to Raydium AMM v4. It is not called by any instruction.
//! No CPI, account mutation, or transfer occurs here.
//!
//! Inputs such as mint owner, current Raydium create-pool fee, and account
//! balances must eventually be verified from accounts immediately before a
//! CPI. This planner does not authorize a migration-fee recipient, decide LP
//! custody or disposal, route creator fees, or create a buyback entitlement.
//! `setup_rent_cap` must cover all caller-funded CPI/temporary-account rent;
//! outer transaction fees are not measured here. Balance arithmetic alone
//! cannot attribute funds if unrelated transfers occur between snapshots;
//! an executable adapter must constrain CPI accounts and verify reserve deltas.
//!
//! The current fee should come from `beam_amm_v4::validate_create_pool_fee_account`
//! after its owner, PDA, and mutable field are checked again before CPI.
//! Host check: `cargo test --locked --manifest-path programs/beam/Cargo.toml
//! --test amm_v4_funding`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuoteAsset {
    NativeSol,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseMintProgram {
    ClassicSpl,
    Token2022OrOther,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FundingError {
    UnsupportedAsset,
    ZeroBaseLiquidity,
    InvalidPoolQuote,
    InsufficientCurveBacking,
    CreatePoolFeeAboveCap,
    InsufficientCallerBudget,
    CurveBackingSpent,
    SetupDonorSpent,
    UnexpectedSetupCredit,
    SetupCostOutsideCap,
    Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FundingInputs {
    /// Verified account facts, not caller-selected program IDs.
    pub quote_asset: QuoteAsset,
    pub base_mint_program: BaseMintProgram,
    pub base_tokens_to_pool: u64,
    pub curve_lamports: u64,
    pub curve_rent_floor: u64,
    pub real_quote_reserves: u64,
    pub post_complete_quote_in: u64,
    /// Separate liabilities; neither is deducted from quote reserves again.
    pub creator_fee_liability: u64,
    pub protocol_fee_liability: u64,
    /// The existing migration charge, with recipient policy unresolved.
    pub migration_fee: u64,
    /// Amount contributed by this caller to a distinct setup authority.
    pub caller_setup_budget: u64,
    /// Preexisting setup-authority balance, including any donor funds.
    pub setup_authority_before: u64,
    /// Read from the current Raydium config and bounded by an admin cap.
    pub current_create_pool_fee: u64,
    pub max_create_pool_fee: u64,
    pub setup_rent_cap: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FundingPlan {
    pub owned_quote: u64,
    pub pool_quote: u64,
    pub migration_fee_unrouted: u64,
    pub protected_curve_floor: u64,
    pub required_curve_lamports: u64,
    pub curve_donor_surplus: u64,
    /// Minimum curve balance after pool quote and migration fee leave.
    pub curve_after_minimum: u64,
    pub caller_setup_budget: u64,
    pub setup_authority_before: u64,
    pub setup_authority_after_funding: u64,
    pub current_create_pool_fee: u64,
    pub setup_cost_cap: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FundingSettlement {
    pub caller_refund: u64,
    pub caller_setup_spent: u64,
    pub setup_authority_after_refund: u64,
    pub curve_lamports_after: u64,
}

fn add(left: u64, right: u64) -> Result<u64, FundingError> {
    left.checked_add(right).ok_or(FundingError::Overflow)
}

/// Plan separate curve-liquidity and caller-setup budgets. This describes a
/// possible transaction, not an executable AMM v4 adapter or quote to users.
pub fn plan(input: FundingInputs) -> Result<FundingPlan, FundingError> {
    if input.quote_asset != QuoteAsset::NativeSol
        || input.base_mint_program != BaseMintProgram::ClassicSpl
    {
        return Err(FundingError::UnsupportedAsset);
    }
    if input.base_tokens_to_pool == 0 {
        return Err(FundingError::ZeroBaseLiquidity);
    }
    let owned_quote = add(input.real_quote_reserves, input.post_complete_quote_in)?;
    let pool_quote = owned_quote
        .checked_sub(input.migration_fee)
        .filter(|amount| *amount > 0)
        .ok_or(FundingError::InvalidPoolQuote)?;
    let protected_curve_floor = add(
        add(input.curve_rent_floor, input.creator_fee_liability)?,
        input.protocol_fee_liability,
    )?;
    let required_curve_lamports = add(protected_curve_floor, owned_quote)?;
    let curve_donor_surplus = input
        .curve_lamports
        .checked_sub(required_curve_lamports)
        .ok_or(FundingError::InsufficientCurveBacking)?;
    let curve_after_minimum = add(protected_curve_floor, curve_donor_surplus)?;

    if input.current_create_pool_fee > input.max_create_pool_fee {
        return Err(FundingError::CreatePoolFeeAboveCap);
    }
    let setup_cost_cap = add(input.current_create_pool_fee, input.setup_rent_cap)?;
    if input.caller_setup_budget < setup_cost_cap {
        return Err(FundingError::InsufficientCallerBudget);
    }
    let setup_authority_after_funding =
        add(input.setup_authority_before, input.caller_setup_budget)?;

    Ok(FundingPlan {
        owned_quote,
        pool_quote,
        migration_fee_unrouted: input.migration_fee,
        protected_curve_floor,
        required_curve_lamports,
        curve_donor_surplus,
        curve_after_minimum,
        caller_setup_budget: input.caller_setup_budget,
        setup_authority_before: input.setup_authority_before,
        setup_authority_after_funding,
        current_create_pool_fee: input.current_create_pool_fee,
        setup_cost_cap,
    })
}

/// Check post-CPI balances before refunding. It rejects use of preexisting
/// setup-authority donations and any new, unaccounted setup credit; only this
/// caller's unspent contribution may be returned under the no-unrelated-flow
/// assumption. The real adapter must also verify pool identity, vault reserve
/// deltas, token balances, actual setup costs, and fee payment.
pub fn settle(
    plan: FundingPlan,
    observed_curve_after: u64,
    observed_setup_authority_after_cpi: u64,
) -> Result<FundingSettlement, FundingError> {
    if observed_curve_after < plan.curve_after_minimum {
        return Err(FundingError::CurveBackingSpent);
    }
    let caller_refund = observed_setup_authority_after_cpi
        .checked_sub(plan.setup_authority_before)
        .ok_or(FundingError::SetupDonorSpent)?;
    if observed_setup_authority_after_cpi > plan.setup_authority_after_funding {
        return Err(FundingError::UnexpectedSetupCredit);
    }
    let caller_setup_spent = plan
        .caller_setup_budget
        .checked_sub(caller_refund)
        .ok_or(FundingError::UnexpectedSetupCredit)?;
    if caller_setup_spent < plan.current_create_pool_fee || caller_setup_spent > plan.setup_cost_cap
    {
        return Err(FundingError::SetupCostOutsideCap);
    }
    Ok(FundingSettlement {
        caller_refund,
        caller_setup_spent,
        setup_authority_after_refund: plan.setup_authority_before,
        curve_lamports_after: observed_curve_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> FundingInputs {
        FundingInputs {
            quote_asset: QuoteAsset::NativeSol,
            base_mint_program: BaseMintProgram::ClassicSpl,
            base_tokens_to_pool: 1_000,
            curve_lamports: 192,
            curve_rent_floor: 10,
            real_quote_reserves: 100,
            post_complete_quote_in: 25,
            creator_fee_liability: 5,
            protocol_fee_liability: 10,
            migration_fee: 10,
            caller_setup_budget: 30,
            setup_authority_before: 7,
            current_create_pool_fee: 4,
            max_create_pool_fee: 5,
            setup_rent_cap: 16,
        }
    }

    #[test]
    fn quote_fee_liabilities_and_donor_surplus_are_separate() {
        let p = plan(valid()).unwrap();
        assert_eq!(
            (p.owned_quote, p.pool_quote, p.migration_fee_unrouted),
            (125, 115, 10)
        );
        assert_eq!(
            (p.protected_curve_floor, p.required_curve_lamports),
            (25, 150)
        );
        assert_eq!((p.curve_donor_surplus, p.curve_after_minimum), (42, 67));
        assert_eq!(
            valid().curve_lamports - p.pool_quote - p.migration_fee_unrouted,
            67
        );
        assert_eq!(
            p.curve_after_minimum,
            p.protected_curve_floor + p.curve_donor_surplus
        );
    }

    #[test]
    fn post_completion_quote_and_migration_fee_boundaries() {
        let mut v = valid();
        v.post_complete_quote_in = 0;
        v.curve_lamports = 167;
        assert_eq!(plan(v).unwrap().pool_quote, 90);
        v.real_quote_reserves = 0;
        v.post_complete_quote_in = 25;
        v.curve_lamports = 67;
        assert_eq!(plan(v).unwrap().pool_quote, 15);
        v.migration_fee = 25;
        assert_eq!(plan(v), Err(FundingError::InvalidPoolQuote));
        v.migration_fee = 26;
        assert_eq!(plan(v), Err(FundingError::InvalidPoolQuote));
        v.migration_fee = 0;
        v.post_complete_quote_in = 0;
        assert_eq!(plan(v), Err(FundingError::InvalidPoolQuote));
    }

    #[test]
    fn curve_backing_and_fee_liability_boundaries() {
        let mut v = valid();
        v.curve_lamports = 150;
        assert_eq!(plan(v).unwrap().curve_donor_surplus, 0);
        v.curve_lamports = 149;
        assert_eq!(plan(v), Err(FundingError::InsufficientCurveBacking));
        v = valid();
        v.creator_fee_liability = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
        v = valid();
        v.protocol_fee_liability = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
        v = valid();
        v.real_quote_reserves = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
        v = valid();
        v.curve_rent_floor = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
        v = valid();
        v.real_quote_reserves = u64::MAX;
        v.post_complete_quote_in = 0;
        v.migration_fee = 0;
        v.curve_rent_floor = 0;
        v.creator_fee_liability = 0;
        v.protocol_fee_liability = 0;
        v.curve_lamports = u64::MAX;
        let p = plan(v).unwrap();
        assert_eq!(
            (p.owned_quote, p.pool_quote, p.curve_after_minimum),
            (u64::MAX, u64::MAX, 0)
        );
    }

    #[test]
    fn caller_budget_covers_current_fee_and_setup_rent_cap() {
        let mut v = valid();
        assert_eq!(plan(v).unwrap().setup_cost_cap, 20);
        v.caller_setup_budget = 20;
        assert_eq!(plan(v).unwrap().setup_authority_after_funding, 27);
        v.caller_setup_budget = 19;
        assert_eq!(plan(v), Err(FundingError::InsufficientCallerBudget));
        v = valid();
        v.current_create_pool_fee = 6;
        assert_eq!(plan(v), Err(FundingError::CreatePoolFeeAboveCap));
        v = valid();
        v.current_create_pool_fee = u64::MAX;
        v.max_create_pool_fee = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
        v = valid();
        v.setup_authority_before = u64::MAX;
        assert_eq!(plan(v), Err(FundingError::Overflow));
    }

    #[test]
    fn refund_preserves_prior_setup_donations_and_only_returns_caller_money() {
        let p = plan(valid()).unwrap();
        let s = settle(p, 67, 18).unwrap();
        assert_eq!((s.caller_setup_spent, s.caller_refund), (19, 11));
        assert_eq!(s.setup_authority_after_refund, 7);
        assert_eq!(s.curve_lamports_after, 67);
        assert_eq!(settle(p, 66, 18), Err(FundingError::CurveBackingSpent));
        assert_eq!(settle(p, 67, 6), Err(FundingError::SetupDonorSpent));
        assert_eq!(settle(p, 67, 38), Err(FundingError::UnexpectedSetupCredit));
        assert_eq!(settle(p, 67, 34), Err(FundingError::SetupCostOutsideCap));
        assert_eq!(settle(p, 67, 16), Err(FundingError::SetupCostOutsideCap));
        let later_curve_donation = settle(p, 70, 18).unwrap();
        assert_eq!(later_curve_donation.curve_lamports_after, 70);
        let mut no_mandatory_cost = valid();
        no_mandatory_cost.current_create_pool_fee = 0;
        no_mandatory_cost.setup_rent_cap = 20;
        let no_cost_plan = plan(no_mandatory_cost).unwrap();
        let full_refund = settle(no_cost_plan, 67, 37).unwrap();
        assert_eq!(
            (full_refund.caller_refund, full_refund.caller_setup_spent),
            (30, 0)
        );
        assert_eq!(full_refund.setup_authority_after_refund, 7);
    }

    #[test]
    fn zero_fee_zero_rent_and_budget_are_valid_only_with_liquidity() {
        let mut v = valid();
        v.current_create_pool_fee = 0;
        v.setup_rent_cap = 0;
        v.caller_setup_budget = 0;
        let p = plan(v).unwrap();
        assert_eq!(
            settle(p, p.curve_after_minimum, 7).unwrap().caller_refund,
            0
        );
        v.base_tokens_to_pool = 0;
        assert_eq!(plan(v), Err(FundingError::ZeroBaseLiquidity));
    }

    #[test]
    fn unsupported_quote_or_mint_program_fails_closed() {
        let mut v = valid();
        v.quote_asset = QuoteAsset::Other;
        assert_eq!(plan(v), Err(FundingError::UnsupportedAsset));
        v = valid();
        v.base_mint_program = BaseMintProgram::Token2022OrOther;
        assert_eq!(plan(v), Err(FundingError::UnsupportedAsset));
    }
}
