use anchor_lang::prelude::*;
use crate::{FeeError, types::Shareholder, sharing_update::{UpdateFeeSharesV2, UpdateFeeSharesEvent}}; use crate::sharing_update::PreparedV2Payout; use crate::sharing_update::{v2_expected_quote, v2_distribute_creator_fees, v2_transfer_creator_fees_to_pump, prepare_legacy_payout}; use crate::instructions::fee_sharing::{legacy_authorize, preprocess_remaining};





































































































































pub fn update_fee_shares_v2<'info>(mut ctx:Context<'_, '_, '_, 'info,UpdateFeeSharesV2<'info>>,shareholders:Vec<Shareholder>)->Result<()> {
  require!(ctx.accounts.sharing_config.version!=1,FeeError::FeeSharesAlreadyUpdated);
  // Same complete authorization target [234096,235304) as legacy.
  legacy_authorize(&ctx.accounts.authority,&ctx.accounts.sharing_config)?;
  let expected = { let info = ctx.accounts.bonding_curve.to_account_info(); match crate::common::curve_quote_mint(&info)? { stored if stored == Pubkey::default() => anchor_spl::token::spl_token::native_mint::ID, stored => stored } };

  if ctx.accounts.quote_mint.key() != expected {









      return Err(error!(FeeError::InvalidQuoteMint).with_pubkeys((ctx.accounts.quote_mint.key(), expected)));
  }
  ctx.remaining_accounts = preprocess_remaining(
      ctx.remaining_accounts, &ctx.accounts.bonding_curve,
      &ctx.accounts.mint.key(), &expected,
  )?;
  v2_transfer_creator_fees_to_pump(
      &ctx.accounts.bonding_curve,

          ctx.accounts.pump_amm_program.to_account_info(),
          ctx.accounts.quote_mint.to_account_info(),
          ctx.accounts.token_program.to_account_info(),
          ctx.accounts.system_program.to_account_info(),
          ctx.accounts.associated_token_program.to_account_info(),
          ctx.accounts.sharing_config.to_account_info(),
          ctx.accounts.coin_creator_vault_authority.to_account_info(),
          ctx.accounts.coin_creator_vault_ata.to_account_info(),
          ctx.accounts.pump_creator_vault.to_account_info(),
          ctx.accounts.pump_creator_vault_ata.to_account_info(),
          ctx.accounts.amm_event_authority.to_account_info(),
          ctx.accounts.authority.as_ref(),

  )?;
  // The cloned AccountInfo/borrow die before payout preparation.
  let zero_quote = crate::common::legacy_curve_quote_is_zero(&ctx.accounts.bonding_curve.to_account_info())?;
  let (prepared_shareholders, changed, payout_accounts) = if zero_quote {
      prepare_legacy_payout(&ctx.remaining_accounts, &ctx.accounts.sharing_config.shareholders)?
  } else {
      prepare_v2_quote_payout(&ctx.remaining_accounts, &ctx.accounts.sharing_config.shareholders)?
  };
  if changed {
      ctx.accounts.sharing_config.shareholders = prepared_shareholders;
      ctx.accounts.sharing_config.exit(&crate::ID)?;
  }
  if !payout_accounts.is_empty() {
      v2_distribute_creator_fees(
          ctx.accounts.pump_program.to_account_info(),
          ctx.accounts.mint.to_account_info(),
          ctx.accounts.bonding_curve.to_account_info(),
          ctx.accounts.sharing_config.to_account_info(),
          ctx.accounts.pump_creator_vault.to_account_info(),
          ctx.accounts.pump_creator_vault_ata.to_account_info(),
          ctx.accounts.quote_mint.to_account_info(),
          ctx.accounts.token_program.to_account_info(),
          ctx.accounts.associated_token_program.to_account_info(),
          ctx.accounts.system_program.to_account_info(),
          ctx.accounts.pump_event_authority.to_account_info(),
          ctx.accounts.authority.to_account_info(),


          payout_accounts,
      )?;
  }
  crate::instructions::fee_sharing::update_fee_shares_v2::finish_v2_update(ctx, shareholders)


 }














pub(crate) fn prepare_v2_quote_payout<'info>(
    remaining: &&[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<crate::sharing_update::PreparedLegacyPayout<'info>> {
    let n = shareholders.len();
    let mut accounts = Vec::with_capacity(n);
    let mut partners = Vec::with_capacity(n);
    let mut retained = Vec::with_capacity(n);
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        let account = remaining.get(i).ok_or_else(|| error!(FeeError::NotEnoughRemainingAccounts))?;

         let j = n + i;
        let partner = remaining.get(j).ok_or_else(|| error!(FeeError::NotEnoughRemainingAccounts))?;

        if account.key() != share.address { return Err(error!(FeeError::ShareholderAccountMismatch).with_values((account.key(), share.address))); }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        partners.push(partner.clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len().saturating_sub(1);
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == n {
        let all = remaining.to_vec();


        return Ok((retained, false, all));
    }
    if retained.is_empty() {
        return Ok((Vec::new(), true, Vec::new()));
    }
    let mut normalized_sum = 0u16;
    for share in retained.iter_mut() {
        share.share_bps = if sum == 0 { 0 } else {
            ((u64::from(share.share_bps) * 10_000 / u64::from(sum)).min(10_000)) as u16
        };
        normalized_sum = normalized_sum.saturating_add(share.share_bps);
    }












              retained[max_index].share_bps = retained[max_index].share_bps
        .saturating_add(10_000u16.saturating_sub(normalized_sum));
    accounts.extend(partners);
    Ok((retained, true, accounts))
}







pub(crate) fn finish_v2_update<'info>(mut ctx: Context<'_, '_, '_, 'info, UpdateFeeSharesV2<'info>>, shareholders: Vec<Shareholder>) -> Result<()> {
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
