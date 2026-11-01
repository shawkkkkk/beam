//! IDL-derived account order/flags only; business constraints are open.
use anchor_lang::prelude::*;
use crate::types::{FeeConfig, FeeProgramGlobal};
use crate::FeeError;

#[event_cpi]
#[derive(Accounts)]
pub struct ExtendFeeConfig<'info> {
    /// CHECK: owner, discriminator, stored bump and PDA are validated by handler.
    #[account(mut)]
    pub fee_config: AccountInfo<'info>,
    pub user: Signer<'info>,
    /// CHECK: used only as the configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetFees<'info> {
    #[account(seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump)]
    pub fee_config: Account<'info, FeeConfig>,
    /// CHECK: Exact deployed constraint not recovered.
    pub config_program_id: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct GetFeesWithQuoteMint<'info> {
    #[account(seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump)]
    pub fee_config: Account<'info, FeeConfig>,
    /// CHECK: Exact deployed constraint not recovered.
    pub config_program_id: UncheckedAccount<'info>,
}

pub use crate::instructions::admin::initialize_fee_config::*;












pub use crate::instructions::admin::initialize_fee_program_global::*;












#[derive(Accounts)]
pub struct RevokeFeeSharingAuthority {}

#[event_cpi]
#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"fee-program-global"], bump = fee_program_global.bump,
              has_one = authority @ FeeError::NotAuthorized)]
    pub fee_program_global: Account<'info, FeeProgramGlobal>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SetClaimRateLimit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"fee-program-global"], bump = fee_program_global.bump,
              has_one = authority @ FeeError::NotAuthorized)]
    pub fee_program_global: Account<'info, FeeProgramGlobal>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SetDisableFlags<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"fee-program-global"], bump = fee_program_global.bump,
              has_one = authority @ FeeError::NotAuthorized)]
    pub fee_program_global: Account<'info, FeeProgramGlobal>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SetExoticFlatFees<'info> {
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: only used as the configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct SetSocialClaimAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"fee-program-global"], bump = fee_program_global.bump,
              has_one = authority @ FeeError::NotAuthorized)]
    pub fee_program_global: Account<'info, FeeProgramGlobal>,
}

#[derive(Accounts)]
pub struct TransferFeeSharingAuthority {}

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    /// CHECK: accepted as the next authority key; zero and nonsigner keys are allowed.
    pub new_admin: AccountInfo<'info>,
    /// CHECK: configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateFeeConfig<'info> {
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    pub admin: Signer<'info>,
    /// CHECK: only used as the configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpdateStableFeeConfig<'info> {
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    pub admin: Signer<'info>,
    /// CHECK: only used as the configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpsertFeeTiers<'info> {
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    pub admin: Signer<'info>,
    /// CHECK: configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct UpsertStableFeeTiers<'info> {
    #[account(mut, seeds = [b"fee_config", config_program_id.key().as_ref()], bump = fee_config.bump,
              has_one = admin @ FeeError::InvalidAdmin)]
    pub fee_config: Account<'info, FeeConfig>,
    pub admin: Signer<'info>,
    /// CHECK: configuration PDA namespace.
    pub config_program_id: UncheckedAccount<'info>,
}
