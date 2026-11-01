use crate::*;











pub fn toggle_boost(ctx: Context<AdminGlobalNoEvent>, enabled: bool) -> Result<()> {
    require_keys_neq!(ctx.accounts.global_config.boost_authority, Pubkey::default(), PumpAmmError::InvalidAdmin);
    ctx.accounts.global_config.boost_enabled = enabled;
    Ok(())
}
