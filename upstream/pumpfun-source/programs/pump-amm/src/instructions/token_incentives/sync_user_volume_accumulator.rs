use crate::*;

pub fn sync_user_volume_accumulator(ctx: Context<SyncUserVolumeAccumulator>) -> Result<()> {
    let g = &ctx.accounts.global_volume_accumulator;
    let u = &mut ctx.accounts.user_volume_accumulator;
    let timestamp = Clock::get()?.unix_timestamp;
    let total_claimed_tokens_before = u.total_claimed_tokens;
    crate::incentives::reset(g, u, timestamp);
    let total_claimed_tokens_after = u.total_claimed_tokens;
    if total_claimed_tokens_before == total_claimed_tokens_after {
        msg!("No changes in accumulator for user={}", ctx.accounts.user.key());
        return Ok(());
    }
    let event = SyncUserVolumeAccumulatorEvent { user: ctx.accounts.user.key(), total_claimed_tokens_before, total_claimed_tokens_after, timestamp };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
