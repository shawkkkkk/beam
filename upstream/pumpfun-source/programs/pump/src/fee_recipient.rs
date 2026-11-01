use anchor_lang::prelude::*;
use crate::{Global, PumpError};
use crate::trade_types::BondingCurve;

/// Validate the recipient against the curve's configured fee-recipient family.
pub fn validate_fee_recipient<'info>(
    fee_recipient: &UncheckedAccount<'info>,
    global: &Account<'info, Global>,
    bonding_curve: &Account<'info, BondingCurve>,
) -> Result<()> {
    let key = fee_recipient.key();
    if bonding_curve.is_mayhem_mode {
        if key == global.reserved_fee_recipient || global.reserved_fee_recipients.contains(&key) { return Ok(()); }
// layout-pad(5): aligns the next Source site with the original line




        return err!(PumpError::NotAuthorized);
    }
    if key == global.fee_recipient || global.fee_recipients.binary_search(&key).is_ok() { return Ok(()); }
// layout-pad(13): aligns the next Source site with the original line












    err!(PumpError::NotAuthorized)
}

/// Validate the eight fee recipients (remaining accounts) and store them in global.
#[inline(never)]
pub fn set_fee_recipients(ctx:&mut Context<crate::set_params::SetParams>)->Result<[Pubkey;8]>{
    let global=&mut ctx.accounts.global;





    require_gte!(ctx.remaining_accounts.len(),8usize,PumpError::NotEnoughRemainingAccounts);
    let fee_recipients:Vec<Pubkey>=ctx.remaining_accounts[..8].iter().map(|account|account.key()).collect();
    let fee_recipients:[Pubkey;8]=fee_recipients.try_into().map_err(|_|PumpError::NotEnoughRemainingAccounts)?;










    require!(fee_recipients.iter().all(|recipient|*recipient!=Pubkey::default()),PumpError::AllFeeRecipientsShouldBeNonZero);
    let rent=Rent::get()?;
    for (index,account) in ctx.remaining_accounts[..8].iter().enumerate(){
        if !rent.is_exempt(account.lamports(),account.try_data_len()?){
            return Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::ConstraintRentExempt).with_account_name(format!("fee_recipients[{}]",index)));
        }
    }











    require!(fee_recipients.windows(2).all(|pair|pair[0]<pair[1]),PumpError::UnsortedNotUniqueFeeRecipients);
    global.fee_recipient=fee_recipients[0];global.fee_recipients.copy_from_slice(&fee_recipients[1..]);
    Ok(fee_recipients)
}
