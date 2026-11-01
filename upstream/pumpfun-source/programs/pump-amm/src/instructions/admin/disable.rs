use crate::*;

pub fn disable(ctx: Context<AdminGlobal>, disable_create_pool: bool,
    disable_deposit: bool, disable_withdraw: bool,
    disable_buy: bool, disable_sell: bool) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    global_config.disable_flags = (global_config.disable_flags & !1) | u8::from(disable_create_pool);
    global_config.disable_flags = (global_config.disable_flags & !2) | (u8::from(disable_deposit) << 1);
    global_config.disable_flags = (global_config.disable_flags & !4) | (u8::from(disable_withdraw) << 2);
    global_config.disable_flags = (global_config.disable_flags & !8) | (u8::from(disable_buy) << 3);
    global_config.disable_flags = (global_config.disable_flags & !16) | (u8::from(disable_sell) << 4);
    let event = DisableEvent { timestamp: Clock::get()?.unix_timestamp,
        admin: ctx.accounts.admin.key(), disable_create_pool,
        disable_deposit, disable_withdraw, disable_buy, disable_sell };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
