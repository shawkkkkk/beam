pub mod claim;
pub(crate) use claim::*;
pub mod close_user_volume_accumulator;
pub mod init_user_volume_accumulator;
pub(crate) use init_user_volume_accumulator::*;
pub mod sync_user_volume_accumulator;

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseUserVolumeAccumulator<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, close = user, seeds = [b"user_volume_accumulator", user.key().as_ref()], bump)]
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,
}
