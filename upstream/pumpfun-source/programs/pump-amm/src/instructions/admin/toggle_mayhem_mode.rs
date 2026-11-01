use crate::*;














pub fn toggle_mayhem_mode(ctx: Context<AdminGlobal>, enabled: bool) -> Result<()> {
    require_neq!(ctx.accounts.global_config.mayhem_mode_enabled, enabled, PumpAmmError::MayhemModeInDesiredState);
    ctx.accounts.global_config.mayhem_mode_enabled = enabled;
    Ok(())
}
