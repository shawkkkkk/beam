use anchor_lang::prelude::*;

/// Mayhem account slots and serialized flags remain for layout compatibility.
/// No successful Beam creation may select or enable that mode.
pub fn mayhem(enabled: bool) -> Result<()> {
    require!(!enabled, BeamError::MayhemDisabled);
    Ok(())
}

/// The original migration/CTO/router interfaces bind PumpSwap accounts. Keep
/// their discriminators, but fail before handler transfers or CPIs.
pub fn legacy_amm() -> Result<()> {
    err!(BeamError::PumpSwapDisabled)
}

#[error_code(offset = 7000)]
pub enum BeamError {
    #[msg("Mayhem is disabled in Beam")]
    MayhemDisabled,
    #[msg("This legacy interface targets PumpSwap; use the Beam Raydium interface")]
    PumpSwapDisabled,
    #[msg("Raydium migration is gated pending validator integration tests")]
    MigrationNotEnabled,
    #[msg("Invalid Raydium account, mint, owner, configuration, or PDA")]
    InvalidRaydiumAccount,
    #[msg("Migration reserves, rent, or setup funding are insufficient")]
    InvalidMigrationBalance,
    #[msg("Mint extensions are not supported by this migration candidate")]
    UnsupportedMint,
    #[msg("Raydium returned unexpected pool, vault, or LP state")]
    InvalidMigrationResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_creation_allowed_mayhem_and_legacy_amm_rejected() {
        assert!(mayhem(false).is_ok());
        assert!(mayhem(true).is_err());
        assert!(legacy_amm().is_err());
    }
}
