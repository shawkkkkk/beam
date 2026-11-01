use crate::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

#[event_cpi]
#[derive(Accounts)]
pub struct SyncUserVolumeAccumulator<'info>{
    /// CHECK: PDA seed only, no signature required.
    pub user:AccountInfo<'info>,
    #[account(seeds=[b"global_volume_accumulator"],bump)]
    pub global_volume_accumulator:Account<'info,GlobalVolumeAccumulator>,
    #[account(mut,seeds=[b"user_volume_accumulator",user.key().as_ref()],bump)]
    pub user_volume_accumulator:Account<'info,UserVolumeAccumulator>,
}
pub(crate) fn reset(g:&GlobalVolumeAccumulator,u:&mut UserVolumeAccumulator,now:i64){
    if u.last_update_timestamp<g.start_time && now<g.start_time {
        u.total_claimed_tokens=0;
        u.has_total_claimed_tokens=false;
    }
}
#[event]
#[derive(Clone)]
pub struct SyncUserVolumeAccumulatorEvent{pub user:Pubkey,pub total_claimed_tokens_before:u64,pub total_claimed_tokens_after:u64,pub timestamp:i64}
#[event]
#[derive(Clone)]
pub struct AdminUpdateTokenIncentivesEvent{pub start_time:i64,pub end_time:i64,pub day_number:u64,pub token_supply_per_day:u64,pub mint:Pubkey,pub seconds_in_a_day:i64,pub timestamp:i64}
