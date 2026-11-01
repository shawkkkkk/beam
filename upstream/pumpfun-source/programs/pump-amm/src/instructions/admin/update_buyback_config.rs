use crate::*;
























pub fn handler(ctx: Context<AdminGlobal>, buyback_basis_points: Option<u64>) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    if let Some(bps) = buyback_basis_points {
        global_config.set_buyback_basis_points(bps)?;
    }
    match ctx.remaining_accounts.len() {
        0 => {}
        8 => {
            let mut recipients = [Pubkey::default(); 8];
            for (i, account) in ctx.remaining_accounts.iter().enumerate() {
                recipients[i] = account.key();
            }
            global_config.set_buyback_fee_recipients(recipients)?;
        }
        _ => return err!(PumpAmmError::WrongBuybackFeeRecipientsCount),
    }
    Ok(())
}
