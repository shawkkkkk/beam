//! Disabled AMM v4 settlement assertions for synthetic integration work.
//!
//! A future instruction must capture these observations from verified
//! AccountInfos immediately around its CPI. Caller-constructed snapshots do
//! not authenticate Raydium state, token ownership, or LP custody. This module
//! performs no transfer, CPI, receipt write, or migration activation.

use anchor_lang::prelude::Pubkey;

use crate::{
    beam_amm_v4::{
        derive_pool_addresses, expected_initial_user_lp, user_lp_ata, AmmV4AbiError,
        SPL_TOKEN_PROGRAM_ID,
    },
    beam_amm_v4_funding::{self, FundingPlan, FundingSettlement},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementError {
    Abi(AmmV4AbiError),
    Alias,
    WrongAccount,
    WrongBeforeBalance,
    WrongFundingDelta,
    WrongDelta,
    InvalidNativeBacking,
    Replay,
    Funding(beam_amm_v4_funding::FundingError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettlementKeys {
    pub curve: Pubkey,
    pub setup_authority: Pubkey,
    pub migration_fee_recipient: Pubkey,
    pub base_source: Pubkey,
    pub wsol_source: Pubkey,
    pub base_vault: Pubkey,
    pub wsol_vault: Pubkey,
    pub lp_destination: Pubkey,
    pub create_fee_destination: Pubkey,
}

impl SettlementKeys {
    fn all(self) -> [Pubkey; 9] {
        [
            self.curve,
            self.setup_authority,
            self.migration_fee_recipient,
            self.base_source,
            self.wsol_source,
            self.base_vault,
            self.wsol_vault,
            self.lp_destination,
            self.create_fee_destination,
        ]
    }

    fn no_alias(self) -> Result<(), SettlementError> {
        let all = self.all();
        for (i, key) in all.iter().enumerate() {
            if *key == Pubkey::default() || all[i + 1..].contains(key) {
                return Err(SettlementError::Alias);
            }
        }
        Ok(())
    }
}

pub fn expected_keys(
    amm_program: &Pubkey,
    base_mint: &Pubkey,
    market_seed: &Pubkey,
    curve: Pubkey,
    migration_fee_recipient: Pubkey,
) -> Result<SettlementKeys, SettlementError> {
    let p = derive_pool_addresses(amm_program, market_seed).map_err(SettlementError::Abi)?;
    let signer =
        Pubkey::find_program_address(&[b"pool-authority", base_mint.as_ref()], &crate::ID).0;
    let keys = SettlementKeys {
        curve,
        setup_authority: signer,
        migration_fee_recipient,
        base_source: user_lp_ata(&signer, base_mint),
        wsol_source: user_lp_ata(&signer, &anchor_spl::token::spl_token::native_mint::id()),
        base_vault: p.coin_vault,
        wsol_vault: p.pc_vault,
        lp_destination: user_lp_ata(&signer, &p.lp_mint),
        create_fee_destination: p.create_pool_fee_destination,
    };
    keys.no_alias()?;
    Ok(keys)
}

/// Amounts must be read from the *same* keyed accounts before and after CPI.
/// WSOL reserve fields must be read from classic SPL Token `is_native` data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Balances {
    pub curve_lamports: u64,
    pub setup_lamports: u64,
    pub migration_fee_recipient_lamports: u64,
    pub base_source_tokens: u64,
    pub wsol_source_tokens: u64,
    pub wsol_source_lamports: u64,
    pub wsol_source_rent_reserve: u64,
    pub base_vault_tokens: u64,
    pub wsol_vault_tokens: u64,
    pub wsol_vault_lamports: u64,
    pub wsol_vault_rent_reserve: u64,
    pub lp_destination_tokens: u64,
    pub create_fee_destination_tokens: u64,
    pub create_fee_destination_lamports: u64,
    pub create_fee_destination_rent_reserve: u64,
    pub receipt_written: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub keys: SettlementKeys,
    pub balances: Balances,
}

fn debited(before: u64, after: u64, expected: u64) -> Result<(), SettlementError> {
    if before.checked_sub(after) == Some(expected) {
        Ok(())
    } else {
        Err(SettlementError::WrongDelta)
    }
}

fn credited(before: u64, after: u64, expected: u64) -> Result<(), SettlementError> {
    if after.checked_sub(before) == Some(expected) {
        Ok(())
    } else {
        Err(SettlementError::WrongDelta)
    }
}

fn backed(balance: u64, reserve: u64, tokens: u64) -> Result<(), SettlementError> {
    if reserve.checked_add(tokens) == Some(balance) {
        Ok(())
    } else {
        Err(SettlementError::InvalidNativeBacking)
    }
}

/// Assert recipient-bound exact changes across funding and one CPI.
/// `before_funding` precedes the curve-to-WSOL source transfer and caller
/// setup contribution. `before_cpi` follows both credits but precedes the
/// Raydium CPI. `after_cpi` follows pool initialization, migration-fee payment
/// to the stored recipient, and a future authenticated receipt write.
/// The caller must verify actual account owners, mint/authority data, pool
/// identity, LP custody, and fee policy; keys and amounts here are observations,
/// not proofs by themselves. No instruction calls this function today.
pub fn verify(
    expected: SettlementKeys,
    plan: FundingPlan,
    base_amount: u64,
    base_decimals: u8,
    before_funding: Observation,
    before_cpi: Observation,
    after_cpi: Observation,
) -> Result<FundingSettlement, SettlementError> {
    expected.no_alias()?;
    if before_funding.keys != expected || before_cpi.keys != expected || after_cpi.keys != expected
    {
        return Err(SettlementError::WrongAccount);
    }
    let i = before_funding.balances;
    let b = before_cpi.balances;
    let a = after_cpi.balances;
    if i.receipt_written || b.receipt_written || !a.receipt_written {
        return Err(SettlementError::Replay);
    }
    let expected_curve_before = plan
        .required_curve_lamports
        .checked_add(plan.curve_donor_surplus)
        .ok_or(SettlementError::Funding(
            beam_amm_v4_funding::FundingError::Overflow,
        ))?;
    if i.curve_lamports != expected_curve_before
        || i.setup_lamports != plan.setup_authority_before
        || i.base_source_tokens != base_amount
        || i.wsol_source_tokens != 0
        || i.base_vault_tokens != 0
        || i.wsol_vault_tokens != 0
        || i.lp_destination_tokens != 0
    {
        return Err(SettlementError::WrongBeforeBalance);
    }
    backed(
        i.wsol_source_lamports,
        i.wsol_source_rent_reserve,
        i.wsol_source_tokens,
    )?;
    backed(
        i.create_fee_destination_lamports,
        i.create_fee_destination_rent_reserve,
        i.create_fee_destination_tokens,
    )?;
    let mut expected_before_cpi = i;
    expected_before_cpi.curve_lamports = i
        .curve_lamports
        .checked_sub(plan.pool_quote)
        .ok_or(SettlementError::WrongFundingDelta)?;
    expected_before_cpi.setup_lamports = i
        .setup_lamports
        .checked_add(plan.caller_setup_budget)
        .ok_or(SettlementError::WrongFundingDelta)?;
    expected_before_cpi.wsol_source_tokens = i
        .wsol_source_tokens
        .checked_add(plan.pool_quote)
        .ok_or(SettlementError::WrongFundingDelta)?;
    expected_before_cpi.wsol_source_lamports = i
        .wsol_source_lamports
        .checked_add(plan.pool_quote)
        .ok_or(SettlementError::WrongFundingDelta)?;
    if b != expected_before_cpi || b.setup_lamports != plan.setup_authority_after_funding {
        return Err(SettlementError::WrongFundingDelta);
    }
    backed(
        b.wsol_source_lamports,
        b.wsol_source_rent_reserve,
        b.wsol_source_tokens,
    )?;
    backed(
        a.wsol_source_lamports,
        a.wsol_source_rent_reserve,
        a.wsol_source_tokens,
    )?;
    backed(
        a.wsol_vault_lamports,
        a.wsol_vault_rent_reserve,
        a.wsol_vault_tokens,
    )?;
    backed(
        b.create_fee_destination_lamports,
        b.create_fee_destination_rent_reserve,
        b.create_fee_destination_tokens,
    )?;
    backed(
        a.create_fee_destination_lamports,
        a.create_fee_destination_rent_reserve,
        a.create_fee_destination_tokens,
    )?;
    debited(
        b.curve_lamports,
        a.curve_lamports,
        plan.migration_fee_unrouted,
    )?;
    credited(
        b.migration_fee_recipient_lamports,
        a.migration_fee_recipient_lamports,
        plan.migration_fee_unrouted,
    )?;
    debited(b.base_source_tokens, a.base_source_tokens, base_amount)?;
    debited(b.wsol_source_tokens, a.wsol_source_tokens, plan.pool_quote)?;
    debited(
        b.wsol_source_lamports,
        a.wsol_source_lamports,
        plan.pool_quote,
    )?;
    credited(b.base_vault_tokens, a.base_vault_tokens, base_amount)?;
    credited(b.wsol_vault_tokens, a.wsol_vault_tokens, plan.pool_quote)?;
    credited(
        b.create_fee_destination_tokens,
        a.create_fee_destination_tokens,
        plan.current_create_pool_fee,
    )?;
    credited(
        b.create_fee_destination_lamports,
        a.create_fee_destination_lamports,
        plan.current_create_pool_fee,
    )?;
    let expected_lp = expected_initial_user_lp(base_amount, plan.pool_quote, base_decimals)
        .map_err(SettlementError::Abi)?;
    credited(
        b.lp_destination_tokens,
        a.lp_destination_tokens,
        expected_lp,
    )?;
    beam_amm_v4_funding::settle(plan, a.curve_lamports, a.setup_lamports)
        .map_err(SettlementError::Funding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beam_amm_v4::DEVNET_PROGRAM_ID;
    use crate::beam_amm_v4_funding::{BaseMintProgram, FundingInputs, QuoteAsset};

    fn fixture() -> (
        SettlementKeys,
        FundingPlan,
        Observation,
        Observation,
        Observation,
    ) {
        let plan = beam_amm_v4_funding::plan(FundingInputs {
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
        })
        .unwrap();
        let keys = expected_keys(
            &DEVNET_PROGRAM_ID,
            &Pubkey::new_from_array([40; 32]),
            &Pubkey::new_from_array([41; 32]),
            Pubkey::new_from_array([42; 32]),
            Pubkey::new_from_array([43; 32]),
        )
        .unwrap();
        let before = Observation {
            keys,
            balances: Balances {
                curve_lamports: 77,
                setup_lamports: 37,
                migration_fee_recipient_lamports: 10,
                base_source_tokens: 1_000,
                wsol_source_tokens: 115,
                wsol_source_lamports: 215,
                wsol_source_rent_reserve: 100,
                base_vault_tokens: 0,
                wsol_vault_tokens: 0,
                wsol_vault_lamports: 0,
                wsol_vault_rent_reserve: 0,
                lp_destination_tokens: 0,
                create_fee_destination_tokens: 20,
                create_fee_destination_lamports: 70,
                create_fee_destination_rent_reserve: 50,
                receipt_written: false,
            },
        };
        let after = Observation {
            keys,
            balances: Balances {
                curve_lamports: 67,
                setup_lamports: 27,
                migration_fee_recipient_lamports: 20,
                base_source_tokens: 0,
                wsol_source_tokens: 0,
                wsol_source_lamports: 100,
                wsol_source_rent_reserve: 100,
                base_vault_tokens: 1_000,
                wsol_vault_tokens: 115,
                wsol_vault_lamports: 315,
                wsol_vault_rent_reserve: 200,
                lp_destination_tokens: 338,
                create_fee_destination_tokens: 24,
                create_fee_destination_lamports: 74,
                create_fee_destination_rent_reserve: 50,
                receipt_written: true,
            },
        };
        let mut initial = before;
        initial.balances.curve_lamports = 192;
        initial.balances.setup_lamports = 7;
        initial.balances.wsol_source_tokens = 0;
        initial.balances.wsol_source_lamports = 100;
        (keys, plan, initial, before, after)
    }

    #[test]
    fn exact_recipient_bound_deltas_preserve_fee_liabilities_and_donors() {
        let (keys, plan, initial, before, after) = fixture();
        let result = verify(keys, plan, 1_000, 0, initial, before, after).unwrap();
        assert_eq!(result.caller_refund, 20);
        assert_eq!(result.caller_setup_spent, 10);
        assert_eq!(result.curve_lamports_after, 67);
        assert_eq!(plan.protected_curve_floor, 25);
        assert_eq!(plan.curve_donor_surplus, 42);
    }

    #[test]
    fn aliases_wrong_recipients_and_replay_fail_before_balance_acceptance() {
        let (keys, plan, initial, before, mut after) = fixture();
        after.keys.migration_fee_recipient = Pubkey::new_from_array([44; 32]);
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongAccount)
        );
        let (keys, plan, initial, before, after) = fixture();
        let mut alias = keys;
        alias.migration_fee_recipient = alias.curve;
        assert_eq!(
            verify(alias, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::Alias)
        );
        let (keys, plan, initial, mut before, after) = fixture();
        before.balances.receipt_written = true;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::Replay)
        );
    }

    #[test]
    fn donation_fee_lp_and_curve_delta_substitutions_fail() {
        let (keys, plan, initial, mut before, after) = fixture();
        before.balances.base_source_tokens += 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongFundingDelta)
        );
        let (keys, plan, initial, before, mut after) = fixture();
        after.balances.curve_lamports -= 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongDelta)
        );
        let (keys, plan, initial, before, mut after) = fixture();
        after.balances.create_fee_destination_tokens += 1;
        after.balances.create_fee_destination_lamports += 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongDelta)
        );
        let (keys, plan, initial, before, mut after) = fixture();
        after.balances.lp_destination_tokens = 0;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongDelta)
        );
        let (keys, plan, initial, before, mut after) = fixture();
        after.balances.wsol_vault_lamports += 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::InvalidNativeBacking)
        );
        let (keys, plan, initial, mut before, after) = fixture();
        before.balances.wsol_source_tokens += 1;
        before.balances.wsol_source_lamports += 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongFundingDelta)
        );
        let (keys, plan, initial, before, mut after) = fixture();
        after.balances.migration_fee_recipient_lamports += 1;
        assert_eq!(
            verify(keys, plan, 1_000, 0, initial, before, after),
            Err(SettlementError::WrongDelta)
        );
    }
}
