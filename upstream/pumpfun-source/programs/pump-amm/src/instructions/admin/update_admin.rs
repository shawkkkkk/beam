use crate::*;

pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> {
    let admin = ctx.accounts.global_config.admin;
    let new_admin = ctx.accounts.new_admin.key();
    ctx.accounts.global_config.admin = new_admin;
    let event = UpdateAdminEvent { timestamp: Clock::get()?.unix_timestamp, admin, new_admin };
    emit!(event.clone());
    emit_cpi!(event);
    Ok(())
}
