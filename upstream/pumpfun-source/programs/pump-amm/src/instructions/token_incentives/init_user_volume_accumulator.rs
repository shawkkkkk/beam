use {
    anchor_lang::prelude::*,
    crate::state::UserVolumeAccumulator,
};
const ACCOUNT_SPACE: usize = 137;

#[event_cpi]
#[derive(Accounts)]
pub struct InitUserVolumeAccumulator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: A user key is sufficient; the user need not sign initialization.
    pub user: AccountInfo<'info>,
    #[account(init_if_needed, payer = payer, space = ACCOUNT_SPACE, seeds = [b"user_volume_accumulator", user.key().as_ref()], bump)]
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,
    pub system_program: Program<'info, System>,
}

pub fn init_user_volume_accumulator(ctx: Context<InitUserVolumeAccumulator>) -> Result<()> {
    let event = crate::state::InitUserVolumeAccumulatorEvent { payer: ctx.accounts.payer.key(),
        user: ctx.accounts.user.key(), timestamp: Clock::get()?.unix_timestamp };
    ctx.accounts.user_volume_accumulator.user = ctx.accounts.user.key();
    emit!(event.clone());
    emit_cpi!(event);
    Ok(())
}
