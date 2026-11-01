use crate::*;

pub fn set_boost_authority(ctx: Context<SetBoostAuthority>) -> Result<()> {
    let accounts = &mut *ctx.accounts;
    instructions::admin::extend_global_config(
        &accounts.global_config, &accounts.system_program, &accounts.admin,
    )?;
    let old_boost_authority = accounts.global_config.boost_authority;
    let new_boost_authority = accounts.boost_authority.key();
    accounts.global_config.boost_authority = new_boost_authority;
    let event = SetBoostAuthorityEvent { timestamp: Clock::get()?.unix_timestamp,
        admin: accounts.admin.key(), old_boost_authority, new_boost_authority };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
