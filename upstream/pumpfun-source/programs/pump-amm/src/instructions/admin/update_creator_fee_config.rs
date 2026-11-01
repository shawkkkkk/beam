use crate::*;

pub fn update_creator_fee_config(ctx: Context<UpdateCreatorFeeConfig>, creator_fee_configurable: bool,
    max_configurable_creator_fee_bps: u64) -> Result<()> {
    let accounts = &mut *ctx.accounts;
    instructions::admin::extend_global_config(
        &accounts.global_config, &accounts.system_program, &accounts.admin,
    )?;
    accounts.global_config.creator_fee_configurable = creator_fee_configurable;
    accounts.global_config.max_configurable_creator_fee_bps = max_configurable_creator_fee_bps;
    let event = UpdateCreatorFeeConfigEvent { timestamp: Clock::get()?.unix_timestamp,
        admin: accounts.admin.key(), creator_fee_configurable, max_configurable_creator_fee_bps };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
