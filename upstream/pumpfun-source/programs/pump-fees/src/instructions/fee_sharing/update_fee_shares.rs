use anchor_lang::prelude::*;
use crate::types::*;
use crate::donation::ConfigStatus;
use crate::{FeeError, sharing_update::{UpdateFeeShares, legacy_transfer_creator_fees_to_pump, legacy_payout_accounts, legacy_distribute_creator_fees}};
use super::{legacy_authorize, preprocess_remaining};












































































































#[event]
pub struct UpdateFeeSharesEvent{pub timestamp:i64,pub mint:Pubkey,pub sharing_config:Pubkey,pub admin:Pubkey,pub new_shareholders:Vec<Shareholder>,pub version:u8}










pub fn update_fee_shares<'info>(mut ctx:Context<'_, '_, '_, 'info,UpdateFeeShares<'info>>,shareholders:Vec<Shareholder>)->Result<()> {
  // The temporary AccountInfo and its data borrow end before version/auth checks.
  let quote_is_zero = crate::common::legacy_curve_quote_is_zero(&ctx.accounts.bonding_curve.to_account_info())?;
  require!(quote_is_zero,FeeError::UnsupportedQuoteMint);
  require!(ctx.accounts.sharing_config.version!=1,FeeError::FeeSharesAlreadyUpdated);
  legacy_authorize(&ctx.accounts.authority,&ctx.accounts.sharing_config)?;
  ctx.remaining_accounts = preprocess_remaining(
      ctx.remaining_accounts, &ctx.accounts.bonding_curve,
      &ctx.accounts.mint.key(), &anchor_spl::token::spl_token::native_mint::ID,
  )?;
  // Own the fixed accounts before the adapter's reserve/owner guard.
  legacy_transfer_creator_fees_to_pump(
      &ctx.accounts.bonding_curve,

          ctx.accounts.pump_amm_program.to_account_info(),
          ctx.accounts.wsol_mint.to_account_info(),
          ctx.accounts.token_program.to_account_info(),
          ctx.accounts.system_program.to_account_info(),
          ctx.accounts.associated_token_program.to_account_info(),
          ctx.accounts.sharing_config.to_account_info(),
          ctx.accounts.coin_creator_vault_authority.to_account_info(),
          ctx.accounts.coin_creator_vault_ata.to_account_info(),
          ctx.accounts.pump_creator_vault.to_account_info(),
          ctx.accounts.amm_event_authority.to_account_info(),

  )?;
  // Existing shareholder validation belongs here, after the AMM call.
  let payout_accounts = legacy_payout_accounts(
      &mut ctx,
  )?;
  if !payout_accounts.is_empty() {
  legacy_distribute_creator_fees(
      ctx.accounts.pump_program.to_account_info(),
      ctx.accounts.mint.to_account_info(),
      ctx.accounts.bonding_curve.to_account_info(),
      ctx.accounts.sharing_config.to_account_info(),
      ctx.accounts.pump_creator_vault.to_account_info(),
      ctx.accounts.system_program.to_account_info(),
      ctx.accounts.pump_event_authority.to_account_info(),


      payout_accounts,
  )?;
  }
  finish_legacy_update(ctx, shareholders)


 }



pub(crate) fn prepare_legacy_payout<'info>(
    remaining: &&[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<crate::sharing_update::PreparedLegacyPayout<'info>> {
    let mut accounts = Vec::with_capacity(remaining.len());
    let mut retained = Vec::with_capacity(shareholders.len());
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        let account = remaining.get(i).ok_or_else(|| error!(FeeError::NotEnoughRemainingAccounts))?;

        if account.key() != share.address { return Err(error!(FeeError::ShareholderAccountMismatch).with_values((account.key(), share.address))); }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len().saturating_sub(1);
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == shareholders.len() {
        // The unchanged result includes trailing accounts, with their real flags.
        return Ok((
            retained, false, remaining.to_vec(),
        ));
    }
    if retained.is_empty() {
        return Ok((
            Vec::new(), true, Vec::new(),
        ));
    }
    let mut normalized_sum = 0u16;
    for share in &mut retained {
        share.share_bps = if sum == 0 { 0 } else {
            ((u64::from(share.share_bps) * 10_000 / u64::from(sum)).min(10_000)) as u16
        };
        normalized_sum = normalized_sum.saturating_add(share.share_bps);
    }












              retained[max_index].share_bps = retained[max_index].share_bps
        .saturating_add(10_000u16.saturating_sub(normalized_sum));
    Ok((retained, true, accounts))
}
























fn finish_legacy_update<'info>(mut ctx: Context<'_, '_, '_, 'info, UpdateFeeShares<'info>>, shareholders: Vec<Shareholder>) -> Result<()> {
    let shares = &shareholders;
    require!(!shares.is_empty(), FeeError::NoShareholders);
    require!(shares.len() <= 10, FeeError::TooManyShareholders);
    let mut sum = 0u16;
    let mut keys = Vec::with_capacity(shares.len());






    for share in shares {
        require!(share.share_bps > 0, FeeError::ZeroShareNotAllowed);
        sum = sum.checked_add(share.share_bps).ok_or(FeeError::ShareCalculationOverflow)?;
        keys.push(share.address);
    }
    keys.sort();
    keys.dedup();
















    require!(sum == 10_000, FeeError::InvalidShareTotal);
    require!(keys.len() == shares.len(), FeeError::DuplicateShareholder);

  let state=&mut ctx.accounts.sharing_config;state.shareholders=shareholders.clone();state.admin_revoked=true;
  let event=UpdateFeeSharesEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),sharing_config:state.key(),admin:state.admin,new_shareholders:shareholders,version:state.version};emit!(event);emit_cpi!(event);Ok(())
}
