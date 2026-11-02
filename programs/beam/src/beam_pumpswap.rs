//! Host checks for the PumpSwap removal slice.
//!
//! This module is not a Beam instruction. It does not migrate a curve, price a
//! pool, collect creator fees, or call PumpSwap or Raydium.

use anchor_lang::prelude::*;

/// Program id still named by retained PumpSwap account constraints.
/// Quote acceptance does not treat an account owned by this id as a pool.
pub fn historical_program_id() -> Pubkey {
    crate::pump_quote::AMM
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::InstructionData;
    use pump_reference as reference;

    fn assert_disabled<T: std::fmt::Debug>(result: anchor_lang::Result<T>) {
        let Err(anchor_lang::error::Error::AnchorError(actual)) = result else {
            panic!("expected PumpSwapDisabled, got {result:?}");
        };
        let Err(anchor_lang::error::Error::AnchorError(policy)) = crate::beam_policy::legacy_amm() else {
            panic!("legacy_amm must be PumpSwapDisabled");
        };
        assert_eq!(actual.error_name, "PumpSwapDisabled");
        assert_eq!(actual.error_code_number, policy.error_code_number);
        assert_eq!(actual.error_msg, policy.error_msg);
    }

    #[test]
    fn historical_pumpswap_id_is_rejected_as_a_quote_owner() {
        let id = historical_program_id();
        assert_eq!(id.to_string(), "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
        assert_eq!(id, crate::AMM);
        assert!(include_str!("multi_hop_curve_swap.rs").contains(&id.to_string()));
        assert!(include_str!("creator/admin_cto.rs").contains(&id.to_string()));
    }

    #[test]
    fn graduated_pool_bytes_stay_decodable_and_unaccepted() {
        let base = Pubkey::new_from_array([4u8; 32]);
        let quote = Pubkey::new_from_array([5u8; 32]);
        let pool = Pubkey::new_from_array([3u8; 32]);
        let mut data = vec![0u8; 211];
        data[..8].copy_from_slice(&[241, 154, 109, 4, 17, 177, 109, 188]);
        data[139..171].copy_from_slice(base.as_ref());
        data[171..203].copy_from_slice(quote.as_ref());
        let state = crate::pump_quote::parse_pool(&data).expect("historical layout decoder");
        assert_eq!(state.base_vault, base);
        assert_eq!(state.quote_vault, quote);

        let mut vault = vec![0u8; 165];
        vault[64..72].copy_from_slice(&1_000u64.to_le_bytes());
        vault[108] = 1;
        let token = anchor_spl::token::ID;
        let owner = historical_program_id();
        let pool_view = crate::pump_quote::View { key: &pool, owner: &owner, data: &data };
        let base_view = crate::pump_quote::View { key: &base, owner: &token, data: &vault };
        let quote_view = crate::pump_quote::View { key: &quote, owner: &token, data: &vault };
        assert_disabled(crate::pump_quote::validate_pool(&pool_view, &pool, &base_view, &quote_view));

        let other = Pubkey::new_from_array([9u8; 32]);
        let other_view = crate::pump_quote::View { key: &pool, owner: &other, data: &data };
        assert_disabled(crate::pump_quote::validate_pool(&other_view, &pool, &base_view, &quote_view));
        assert_disabled(crate::pump_quote::graduated_quote_unavailable::<()>());
    }

    #[test]
    fn legacy_handlers_return_before_pumpswap_bodies() {
        let lib = include_str!("lib.rs");
        assert!(lib.contains("beam_policy::legacy_amm()?;migrate::execute(ctx)"));
        assert!(lib.contains("beam_policy::legacy_amm()?;migrate_v2::execute(ctx)"));
        assert!(lib.contains("beam_policy::legacy_amm()?;multi_hop_curve::execute(ctx,amount_in,hop_fees)"));
        assert!(lib.contains("beam_policy::legacy_amm()?;admin_cto::execute(ctx,is_holder_reward,creator_fee_bps,new_creator)"));
        for src in [
            include_str!("migrate.rs"),
            include_str!("migrate_v2.rs"),
            include_str!("multi_hop_curve_swap.rs"),
            include_str!("creator/admin_cto.rs"),
        ] {
            assert!(src.contains("crate::beam_policy::legacy_amm()?;"), "handler body is not fail-closed");
        }
    }

    #[test]
    fn legacy_instruction_bytes_still_match_the_reference() {
        assert_eq!(crate::instruction::Migrate {}.data(), reference::instruction::Migrate {}.data());
        assert_eq!(crate::instruction::MigrateV2 {}.data(), reference::instruction::MigrateV2 {}.data());
        let hops = vec![1u8, 2];
        assert_eq!(
            crate::instruction::MultiHopCurveSwap { amount_in: 5, hop_fees: hops.clone() }.data(),
            reference::instruction::MultiHopCurveSwap { amount_in: 5, hop_fees: hops }.data()
        );
        let creator = Pubkey::new_from_array([7u8; 32]);
        assert_eq!(
            crate::instruction::AdminCto {
                is_holder_reward: Some(false),
                creator_fee_bps: Some(25),
                new_creator: Some(creator),
            }
            .data(),
            reference::instruction::AdminCto {
                is_holder_reward: Some(false),
                creator_fee_bps: Some(25),
                new_creator: Some(creator),
            }
            .data()
        );
    }

    #[test]
    fn curve_creator_collection_does_not_name_the_pumpswap_program() {
        let source = include_str!("creator/collect_creator_fee.rs");
        assert!(!source.contains("pump_amm"));
        assert!(!source.contains("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"));
        let sharing = include_str!("creator/distribute_creator_fees.rs");
        assert!(!sharing.contains("pump_amm::cpi"));
    }
}
