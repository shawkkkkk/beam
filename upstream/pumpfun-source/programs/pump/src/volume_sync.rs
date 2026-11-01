use crate::*;
pub use crate::token_incentives::volume_accumulator::GlobalVolumeAccumulator;
#[event_cpi]
#[derive(Accounts)]
pub struct SyncUserVolumeAccumulator<'info> {
    /// CHECK: used to derive the user accumulator and name the event.
    pub user: AccountInfo<'info>,
    #[account(seeds=[b"global_volume_accumulator"],bump)]
    pub global_volume_accumulator: Account<'info,GlobalVolumeAccumulator>,
    #[account(mut,seeds=[b"user_volume_accumulator",user.key().as_ref()],bump)]
    pub user_volume_accumulator: Account<'info,UserVolumeAccumulator>,
}
fn reset(g:&GlobalVolumeAccumulator,u:&mut UserVolumeAccumulator,now:i64){if u.last_update_timestamp<g.start_time && now<g.start_time {u.total_claimed_tokens=0;u.has_total_claimed_tokens=false;}}
pub fn execute(ctx:Context<SyncUserVolumeAccumulator>)->Result<()> {
    let a=&mut *ctx.accounts; let timestamp=Clock::get()?.unix_timestamp;
    let before=a.user_volume_accumulator.total_claimed_tokens;
    reset(&a.global_volume_accumulator,&mut a.user_volume_accumulator,timestamp);
    let after=a.user_volume_accumulator.total_claimed_tokens;
    if before==after {
        msg!("No changes in accumulator for user={}",a.user.key());
    }else{
        let event=SyncUserVolumeAccumulatorEvent{user:a.user.key(),total_claimed_tokens_before:before,total_claimed_tokens_after:after,timestamp};
        emit!(event);emit_cpi!(event);
    }
    Ok(())
}

#[event]
#[derive(Clone)]
pub struct SyncUserVolumeAccumulatorEvent {
    pub user: Pubkey,
    pub total_claimed_tokens_before: u64,
    pub total_claimed_tokens_after: u64,
    pub timestamp: i64,
}
