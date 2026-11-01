use crate::*;




pub use native_transfer_fee::{decode_transfer_fees, DecodedTransferFee};
















pub type TransferFeeData = Option<DecodedTransferFee>;

/// Reads the Token-2022 transfer-fee configuration that applies in the current epoch.
pub fn from_mint(mint: &AccountInfo) -> Result<TransferFeeData> {
    if mint.owner != &anchor_spl::token_2022::ID {
        return const { Ok(None) };
    }
    let data = mint.try_borrow_data()?;
    let fees = decode_transfer_fees(&data)
        .map_err(|_| error!(PumpAmmError::UnsupportedQuoteMint))?;
    let epoch = Clock::get()?.epoch;
    Ok(fees.map(|(older, newer)| if epoch >= newer.epoch { newer } else { older }))



}

/// Fee withheld by Token-2022 when `amount` is transferred.
///
/// Mints without a transfer-fee extension charge nothing.
pub fn calculate_fee(data: &TransferFeeData, amount: u64) -> Result<u64> {
    match *data {
        None => Ok(0),
        Some(fee) => fee

            .calculate_fee(amount)
            .ok_or_else(|| error!(PumpAmmError::Overflow)),
    }
}

/// Amount received after the transfer fee is withheld.
pub fn amount_after_fee(data: &TransferFeeData, amount: u64) -> Result<u64> {
    amount
        .checked_sub(calculate_fee(data, amount)?)
        .ok_or_else(|| error!(PumpAmmError::Overflow))
}

/// Gross amount to send so that `amount` arrives after the transfer fee.
pub fn amount_before_fee(data: &TransferFeeData, amount: u64) -> Result<u64> {
    match *data {
        None => Ok(amount),
        Some(fee) => fee

            .calculate_pre_fee_amount(amount)
            .ok_or_else(|| error!(PumpAmmError::Overflow)),
    }
}

/// Harvests withheld Token-2022 transfer fees from `source` into `mint`.
pub fn harvest_to_mint<'info>(
    token_program: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    source: &AccountInfo<'info>,
) -> Result<()> {
    anchor_spl::token_2022_extensions::transfer_fee::harvest_withheld_tokens_to_mint(
        CpiContext::new(
            token_program.clone(),
            anchor_spl::token_2022_extensions::transfer_fee::HarvestWithheldTokensToMint {
                token_program_id: token_program.clone(),
                mint: mint.clone(),
            },
        ),
        vec![source.clone()],
    )
}
