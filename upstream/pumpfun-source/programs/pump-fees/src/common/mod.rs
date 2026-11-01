use anchor_lang::prelude::*;















#[inline(never)]
pub fn legacy_curve_quote_is_zero(bonding_curve: &AccountInfo<'_>) -> Result<bool> {
    if bonding_curve.data_len() < 115 { return Ok(true); }

 Ok(Pubkey::try_from_slice(&bonding_curve.data.borrow()[83..115])? == Pubkey::default())
}






pub(crate) fn curve_quote_mint(bonding_curve: &AccountInfo<'_>) -> Result<Pubkey> {
    if bonding_curve.data_len() < 115 { return Ok(Pubkey::default()); }


 Ok(Pubkey::try_from_slice(&bonding_curve.data.borrow()[83..115])?)
}
