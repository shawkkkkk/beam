use crate::*;
use crate::creator_reassignment::*;









































/// Replaces the pool coin creator with the sharing config PDA.
///
//
//
//
//
pub fn migrate(ctx: Context<MigratePoolCoinCreator>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    require!(!pool.is_holder_reward, PumpAmmError::HolderRewardCreatorImmutable);

    // Fees accrued for the previous coin creator have to be swept first,
    // otherwise they would become claimable by the sharing config.
    //
    //
    //
    require!(pool.creator_fees == 0, PumpAmmError::CreatorFeesNotSwept);

    let old_coin_creator = pool.coin_creator;
    pool.coin_creator = ctx.accounts.sharing_config.key();
    let event = MigratePoolCoinCreatorEvent {
        timestamp: Clock::get()?.unix_timestamp,
        base_mint: pool.base_mint,
        pool: pool.key(),
        sharing_config: ctx.accounts.sharing_config.key(),
        old_coin_creator,
        new_coin_creator: ctx.accounts.sharing_config.key(),
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct MigratePoolCoinCreator<'info>{
 #[account(mut,seeds=[b"pool",pool.index.to_le_bytes().as_ref(),pool.creator.as_ref(),pool.base_mint.as_ref(),pool.quote_mint.as_ref()],bump,constraint=pool.creator==crate::utils::pool_authority(&pool.base_mint) @ PumpAmmError::OnlyCanonicalPumpPoolsCanHaveCoinCreator)]pub pool:Account<'info,Pool>,
 #[account(seeds=[b"sharing-config",pool.base_mint.as_ref()],bump,seeds::program=FEES_PROGRAM,constraint=sharing_config.mint==pool.base_mint @ PumpAmmError::InvalidSharingConfigBaseMint,constraint=sharing_config.admin==pool.coin_creator @ PumpAmmError::InvalidSharingConfigCoinCreator)]pub sharing_config:Account<'info,SharingConfig>,
}
