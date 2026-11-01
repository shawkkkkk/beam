use crate::*;















pub(crate) fn quote_for_base(base_amount_out: u64, base_reserve: u64, quote_reserve: u64) -> Result<u64> {
    let numerator = (quote_reserve as u128) * (base_amount_out as u128);
    let quote = crate::curve::fees::ceil_div(numerator,
        (base_reserve as u128).checked_sub(base_amount_out as u128)
            .ok_or_else(|| error!(PumpAmmError::BuyMoreBaseAmountThanPoolReserves))?
    )?;
    u64::try_from(quote)
        .map_err(|_| error!(PumpAmmError::Truncation))
}













/// Constant-product quote paid for a gross base input, before AMM fees.
#[inline(never)]
pub(crate) fn quote_for_sell(base_amount_in: u64, base_reserve: u64, quote_reserve: u64) -> Result<u64> {
    let denominator = (base_reserve as u128) + (base_amount_in as u128);
    let numerator = (quote_reserve as u128) * (base_amount_in as u128);
    let quote = numerator.checked_div(denominator)
        .ok_or_else(|| error!(PumpAmmError::DivisionByZero))?;
    u64::try_from(quote)
        .map_err(|_| error!(PumpAmmError::Truncation))
}

#[inline(never)]
pub(crate) fn base_for_quote(quote_amount_in: u64, base_reserve: u64, quote_reserve: u64) -> Result<u64> {
    let adjusted_quote = quote_amount_in.checked_sub(1).ok_or(PumpAmmError::Overflow)?;
    base_for_buy(adjusted_quote, quote_reserve, base_reserve)
}

























#[inline(never)]
pub(crate) fn base_for_buy(quote_amount_in: u64, quote_reserve: u64, base_reserve: u64) -> Result<u64> {
    let denominator = (quote_reserve as u128) + (quote_amount_in as u128);
    let numerator = (base_reserve as u128) * (quote_amount_in as u128);
    let base = numerator.checked_div(denominator)
        .ok_or_else(|| error!(PumpAmmError::DivisionByZero))?;
    u64::try_from(base)
        .map_err(|_| error!(PumpAmmError::Truncation))
}





















pub(crate) fn liquidity_floor_ratio(reserve: u64, amount: u64, supply: u64) -> Result<u128> {
    ((reserve as u128) * (amount as u128)).checked_div(supply as u128)
        .ok_or_else(|| error!(PumpAmmError::DivisionByZero))
}
