use crate::utils::quote::is_native_or_usdc;
use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct AdminCtoPool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub global_config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [&crate::utils::POOL_AUTHORITY_SEED, pool.base_mint.as_ref()],
        bump,
        seeds::program = trade::PUMP,
    )]
    pub pool_authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[event]
#[derive(Clone)]
pub struct AdminCtoPoolEvent {
    pub timestamp: i64,
    pub base_mint: Pubkey,
    pub pool: Pubkey,
    pub old_coin_creator: Pubkey,
    pub new_coin_creator: Pubkey,
    pub is_holder_reward: bool,
    pub is_cashback_coin: bool,
    pub old_creator_fee_bps: u64,
    pub new_creator_fee_bps: u64,
}

/// Admin-driven community takeover of a canonical Pump pool.
///
/// The pool's coin creator is replaced by `coin_creator`. When
/// `is_holder_reward` is set, the new coin creator must be the Pump
/// holder-rewards PDA of the base mint and the pool stops being a
/// cashback coin. A holder-reward pool can only be reassigned to the
/// holder-rewards PDA again.
///
/// When `creator_fee_bps` is provided, the pool creator fee is updated as
/// well, subject to the global creator fee configuration.
///
/// All accrued coin creator fees must be swept before the takeover.
pub fn handler(
    ctx: Context<AdminCtoPool>,
    coin_creator: Pubkey,
    is_holder_reward: bool,
    creator_fee_bps: Option<u64>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let global_config = &ctx.accounts.global_config;

    // Only canonical pools created by Pump migrations (pool creator is the
    // Pump pool-authority PDA of the base mint) have a coin creator.
    // (see `crate::utils::pool_authority`).

    require!(
        pool.creator == crate::utils::pool_authority(&pool.base_mint),
        PumpAmmError::OnlyCanonicalPumpPoolsCanHaveCoinCreator
    );
    require!(
        !pool.is_mayhem_mode,
        PumpAmmError::CtoNotAllowedForMayhemPool
    );

    let (holder_rewards, _) =
        Pubkey::find_program_address(&[&crate::utils::HOLDER_REWARDS_SEED, pool.base_mint.as_ref()], &trade::PUMP);

    if pool.is_holder_reward {
        require!(
            is_holder_reward && coin_creator == holder_rewards,
            PumpAmmError::HolderRewardCreatorImmutable
        );
    }
    if is_holder_reward {
        require_keys_eq!(
            coin_creator,
            holder_rewards,
            PumpAmmError::InvalidHolderRewardCoinCreator
        );
    }

    let is_cashback_coin = if is_holder_reward {
        false
    } else {
        pool.is_cashback_coin
    };

    if let Some(creator_fee_bps) = creator_fee_bps {
        // Creator fees are not configurable for SOL / USDC quoted pools, nor
        // for cashback coins.
        require!(
            !is_native_or_usdc(&pool.quote_mint),
            PumpAmmError::CreatorFeeNotConfigurableForQuote
        );
        require!(
            !is_cashback_coin,
            PumpAmmError::CreatorFeeNotAllowedForCashbackCoin
        );
        require!(
            global_config.creator_fee_configurable,
            PumpAmmError::CreatorFeeNotConfigurable
        );
        require!(
            creator_fee_bps > 0
                && creator_fee_bps <= global_config.max_configurable_creator_fee_bps,
            PumpAmmError::CreatorFeeBpsOutOfRange
        );
    }

    // The coin creator fees accrued so far belong to the old coin creator
    // and must be collected before it is replaced (see
    // `collect_coin_creator_fee`).
    require!(
        pool.creator_fees == 0,
        PumpAmmError::CreatorFeesNotSwept
    );

    prepare_pool(
        pool,
        &ctx.accounts.system_program,
        &ctx.accounts.payer,
    )?;

    let old_coin_creator = pool.coin_creator;
    pool.coin_creator = coin_creator;
    pool.is_holder_reward = is_holder_reward;
    pool.is_cashback_coin = is_cashback_coin;

    let old_creator_fee_bps = pool.creator_fee_bps;
    if let Some(creator_fee_bps) = creator_fee_bps {
        pool.creator_fee_bps = creator_fee_bps;
    }
    pool.can_edit_creator_fee = false;

    let event = AdminCtoPoolEvent {
        timestamp: Clock::get()?.unix_timestamp,
        base_mint: pool.base_mint,
        pool: pool.key(),
        old_coin_creator,
        new_coin_creator: coin_creator,
        is_holder_reward,
        is_cashback_coin,
        old_creator_fee_bps,
        new_creator_fee_bps: pool.creator_fee_bps,
    };
    emit!(event);
    emit_cpi!(event);

    Ok(())
}

fn prepare_pool<'info>(
    pool: &Account<'info, Pool>,
    system_program: &Program<'info, System>,
    payer: &Signer<'info>,
) -> Result<()> {
    crate::instructions::swap::prepare_pool(pool, system_program.as_ref(), payer.as_ref())
}
