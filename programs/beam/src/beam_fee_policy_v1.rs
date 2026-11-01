//! Beam-owned fee policy and custody boundary, deliberately disabled.
//! Existing trades still use the inherited Pump Fees program. This account
//! does not quote, collect, distribute, withdraw, or route a launch surcharge.
use anchor_lang::prelude::*;

use crate::Global;

pub const POLICY_VERSION: u8 = 1;
pub const ORDINARY_BEAM_TOTAL_BPS: u16 = 50;
pub const ORDINARY_COMMUNITY_BPS: u16 = 42;
pub const ORDINARY_TREASURY_BPS: u16 = 5;
pub const ORDINARY_FOUNDER_BPS: u16 = 3;
pub const ORDINARY_PLATFORM_BPS: u16 = ORDINARY_TREASURY_BPS + ORDINARY_FOUNDER_BPS;

// Anchor account discriminator (8) plus fixed Borsh fields. No variable data.
pub const POLICY_SPACE: usize = 8 + 1 + 1 + 1 + 1 + 4 * 2 + 32 + 1 + 1;
pub const VAULT_SPACE: usize = 8 + 1 + 32 + 1 + 8;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyStatus {
    Disabled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatorFunding {
    Undecided,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurchargeStatus {
    Disabled,
}

#[account]
pub struct BeamFeePolicyV1 {
    pub version: u8,
    pub status: PolicyStatus,
    pub creator_funding: CreatorFunding,
    pub launch_surcharge: SurchargeStatus,
    /// Nominal Beam component only. No all-in or creator-fee claim is stored.
    pub ordinary_total_bps: u16,
    pub ordinary_community_bps: u16,
    pub ordinary_treasury_bps: u16,
    pub ordinary_founder_bps: u16,
    pub vault: Pubkey,
    pub policy_bump: u8,
    pub vault_bump: u8,
}

#[account]
pub struct BeamFeeVaultV1 {
    pub version: u8,
    pub policy: Pubkey,
    pub bump: u8,
    /// Always zero in V1. This is not a withdrawable balance or fee receipt.
    pub credited_fee_lamports: u64,
}

#[derive(Accounts)]
pub struct InitializeBeamFeePolicyV1<'info> {
    #[account(seeds = [b"global"], bump, has_one = authority @ crate::PumpError::NotAuthorized)]
    pub global: Box<Account<'info, Global>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = POLICY_SPACE, seeds = [b"beam-fee-policy", b"v1".as_ref()], bump)]
    pub policy: Account<'info, BeamFeePolicyV1>,
    #[account(init, payer = authority, space = VAULT_SPACE, seeds = [b"beam-fee-vault", b"v1".as_ref()], bump)]
    pub vault: Account<'info, BeamFeeVaultV1>,
    pub system_program: Program<'info, System>,
}

#[error_code(offset = 7100)]
pub enum BeamFeePolicyError {
    #[msg("Beam fee policy V1 is disabled; creator funding is undecided")]
    QuoteDisabled,
    #[msg("Invalid Beam fee policy or vault state")]
    InvalidState,
}

impl BeamFeePolicyV1 {
    fn disabled(vault: Pubkey, policy_bump: u8, vault_bump: u8) -> Self {
        Self {
            version: POLICY_VERSION,
            status: PolicyStatus::Disabled,
            creator_funding: CreatorFunding::Undecided,
            launch_surcharge: SurchargeStatus::Disabled,
            ordinary_total_bps: ORDINARY_BEAM_TOTAL_BPS,
            ordinary_community_bps: ORDINARY_COMMUNITY_BPS,
            ordinary_treasury_bps: ORDINARY_TREASURY_BPS,
            ordinary_founder_bps: ORDINARY_FOUNDER_BPS,
            vault,
            policy_bump,
            vault_bump,
        }
    }

    /// This V1 state has no activation path. Validate every stored field and
    /// reject any attempted Beam quote; inherited Pump trade paths are separate.
    pub fn require_quote_enabled(
        &self,
        vault: &BeamFeeVaultV1,
        policy_key: Pubkey,
        vault_key: Pubkey,
    ) -> Result<()> {
        let (expected_policy, policy_bump) =
            Pubkey::find_program_address(&[b"beam-fee-policy", b"v1".as_ref()], &crate::ID);
        let (expected_vault, vault_bump) =
            Pubkey::find_program_address(&[b"beam-fee-vault", b"v1".as_ref()], &crate::ID);
        require_keys_eq!(
            policy_key,
            expected_policy,
            BeamFeePolicyError::InvalidState
        );
        require_keys_eq!(vault_key, expected_vault, BeamFeePolicyError::InvalidState);
        require_keys_eq!(self.vault, vault_key, BeamFeePolicyError::InvalidState);
        require_keys_eq!(vault.policy, policy_key, BeamFeePolicyError::InvalidState);
        require!(
            self.version == POLICY_VERSION
                && vault.version == POLICY_VERSION
                && self.policy_bump == policy_bump
                && self.vault_bump == vault_bump
                && vault.bump == vault_bump
                && self.status == PolicyStatus::Disabled
                && self.creator_funding == CreatorFunding::Undecided
                && self.launch_surcharge == SurchargeStatus::Disabled
                && self.ordinary_total_bps == ORDINARY_BEAM_TOTAL_BPS
                && self.ordinary_community_bps == ORDINARY_COMMUNITY_BPS
                && self.ordinary_treasury_bps == ORDINARY_TREASURY_BPS
                && self.ordinary_founder_bps == ORDINARY_FOUNDER_BPS
                && u32::from(self.ordinary_community_bps)
                    + u32::from(self.ordinary_treasury_bps)
                    + u32::from(self.ordinary_founder_bps)
                    == u32::from(self.ordinary_total_bps)
                && vault.credited_fee_lamports == 0,
            BeamFeePolicyError::InvalidState
        );
        err!(BeamFeePolicyError::QuoteDisabled)
    }
}

pub fn initialize(ctx: Context<InitializeBeamFeePolicyV1>) -> Result<()> {
    let policy_key = ctx.accounts.policy.key();
    let vault_key = ctx.accounts.vault.key();
    *ctx.accounts.policy = BeamFeePolicyV1::disabled(vault_key, ctx.bumps.policy, ctx.bumps.vault);
    *ctx.accounts.vault = BeamFeeVaultV1 {
        version: POLICY_VERSION,
        policy: policy_key,
        bump: ctx.bumps.vault,
        credited_fee_lamports: 0,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_code(error: anchor_lang::error::Error) -> u32 {
        match error {
            anchor_lang::error::Error::AnchorError(inner) => inner.error_code_number,
            other => panic!("expected Beam policy error, got {other:?}"),
        }
    }

    fn disabled_fixture() -> (BeamFeePolicyV1, BeamFeeVaultV1, Pubkey, Pubkey) {
        let (policy_key, policy_bump) =
            Pubkey::find_program_address(&[b"beam-fee-policy", b"v1".as_ref()], &crate::ID);
        let (vault_key, vault_bump) =
            Pubkey::find_program_address(&[b"beam-fee-vault", b"v1".as_ref()], &crate::ID);
        let policy = BeamFeePolicyV1::disabled(vault_key, policy_bump, vault_bump);
        let vault = BeamFeeVaultV1 {
            version: POLICY_VERSION,
            policy: policy_key,
            bump: vault_bump,
            credited_fee_lamports: 0,
        };
        (policy, vault, policy_key, vault_key)
    }

    #[test]
    fn state_is_versioned_disabled_undecided_and_zero_flow() {
        let (policy, vault, policy_key, vault_key) = disabled_fixture();
        assert_eq!(policy.status, PolicyStatus::Disabled);
        assert_eq!(policy.creator_funding, CreatorFunding::Undecided);
        assert_eq!(policy.launch_surcharge, SurchargeStatus::Disabled);
        assert_eq!(vault.credited_fee_lamports, 0);
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, policy_key, vault_key)
                    .unwrap_err()
            ),
            7100
        );
    }

    #[test]
    fn canonical_policy_and_vault_links_are_required() {
        let (mut policy, mut vault, policy_key, vault_key) = disabled_fixture();
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, Pubkey::new_unique(), vault_key)
                    .unwrap_err()
            ),
            7101
        );
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, policy_key, Pubkey::new_unique())
                    .unwrap_err()
            ),
            7101
        );
        policy.vault = Pubkey::new_unique();
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, policy_key, vault_key)
                    .unwrap_err()
            ),
            7101
        );
        policy.vault = vault_key;
        vault.policy = Pubkey::new_unique();
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, policy_key, vault_key)
                    .unwrap_err()
            ),
            7101
        );
    }

    #[test]
    fn policy_allocation_is_nominal_and_existing_math_conserves_fees() {
        let (policy, mut vault, policy_key, vault_key) = disabled_fixture();
        assert_eq!(
            policy.ordinary_community_bps
                + policy.ordinary_treasury_bps
                + policy.ordinary_founder_bps,
            policy.ordinary_total_bps
        );
        assert_eq!(ORDINARY_PLATFORM_BPS, 8);
        for quote in [0, 1, 199, 200, 10_000, 100_000_000_000, u64::MAX] {
            let allocation = crate::beam_economics::allocate(quote).unwrap();
            assert_eq!(
                allocation.total,
                allocation.community + allocation.treasury + allocation.founder
            );
        }
        vault.credited_fee_lamports = 1;
        assert_eq!(
            error_code(
                policy
                    .require_quote_enabled(&vault, policy_key, vault_key)
                    .unwrap_err()
            ),
            7101
        );
    }

    #[test]
    fn serialized_accounts_fit_their_fixed_allocation() {
        let (policy, vault, _, _) = disabled_fixture();
        let mut policy_bytes = Vec::new();
        let mut vault_bytes = Vec::new();
        policy.try_serialize(&mut policy_bytes).unwrap();
        vault.try_serialize(&mut vault_bytes).unwrap();
        assert_eq!(policy_bytes.len(), POLICY_SPACE);
        assert_eq!(vault_bytes.len(), VAULT_SPACE);
    }
}
