use crate::*;

pub fn close_user_volume_accumulator(ctx: Context<CloseUserVolumeAccumulator>) -> Result<()> {
    let account = &ctx.accounts.user_volume_accumulator;
    let total_unclaimed_tokens = account.total_unclaimed_tokens;
    let total_claimed_tokens = account.total_claimed_tokens;
    let current_sol_volume = account.current_sol_volume;
    let last_update_timestamp = account.last_update_timestamp;
    let event = CloseUserVolumeAccumulatorEvent { user: ctx.accounts.user.key(),
        timestamp: Clock::get()?.unix_timestamp, total_unclaimed_tokens,
        total_claimed_tokens, current_sol_volume,
        last_update_timestamp };
    emit!(event.clone());
    emit_cpi!(event);
    Ok(())
}
