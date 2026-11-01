use crate::*;


















pub fn toggle_cashback_enabled(ctx: Context<AdminGlobal>, enabled: bool) -> Result<()> {
    require_neq!(ctx.accounts.global_config.is_cashback_enabled, enabled, PumpAmmError::CashbackNotInDesiredState);
    ctx.accounts.global_config.is_cashback_enabled = enabled;
    Ok(())
}
