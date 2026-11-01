//! Recovered sharing updates; captured Pump/AMM dependencies execute their own CPIs.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::{Instruction,AccountMeta},program::invoke};
use crate::{FeeError, pump_global}; use crate::instructions::fee_sharing::preprocess_remaining;
use crate::types::Shareholder;
use crate::donation::{PUMP, AMM, BondingCurve, ConfigStatus, SharingConfig};
use crate::sharing::{Pump, Amm};
// Static legacy source hypothesis, bound to the frozen 8ff12 source. Not compiled.
pub use crate::instructions::fee_sharing::update_fee_shares::update_fee_shares;















































pub use crate::instructions::fee_sharing::update_fee_shares_v2::update_fee_shares_v2;






































































fn authorize(state:&SharingConfig,authority:Pubkey)->Result<()>{
 require_keys_eq!(state.admin,authority,FeeError::NotAuthorized);
 require!(!state.admin_revoked,FeeError::SharingConfigAdminRevoked);
 require!(state.status==ConfigStatus::Active,FeeError::SharingConfigNotActive);Ok(())
}
fn validate_remaining(state:&SharingConfig,remaining:&[AccountInfo],stride:usize)->Result<()>{
 require!(remaining.len()>=state.shareholders.len()*stride,FeeError::NotEnoughRemainingAccounts);
 for (i,share) in state.shareholders.iter().enumerate(){require_keys_eq!(remaining[i].key(),share.address,FeeError::ShareholderAccountMismatch);}Ok(())
}
fn validate_shares(shares:&[Shareholder])->Result<()>{
 require!(!shares.is_empty(),FeeError::NoShareholders);require!(shares.len()<=10,FeeError::TooManyShareholders);
 let mut sum=0u16;let mut seen=Vec::new();
 for share in shares {require!(share.share_bps>0,FeeError::ZeroShareNotAllowed);sum=sum.checked_add(share.share_bps).ok_or(error!(FeeError::ShareCalculationOverflow))?;require!(!seen.contains(&share.address),FeeError::DuplicateShareholder);seen.push(share.address);}
 require!(sum==10000,FeeError::InvalidShareTotal);Ok(())
}
fn call<'info>(program_id:Pubkey,mut infos:Vec<AccountInfo<'info>>,flags:&[(bool,bool)],data:Vec<u8>,remaining:&[AccountInfo<'info>])->Result<()>{
 let mut accounts:Vec<AccountMeta>=infos.iter().zip(flags).map(|(a,(w,s))|if *w{AccountMeta::new(a.key(),*s)}else{AccountMeta::new_readonly(a.key(),*s)}).collect();
 accounts.extend(remaining.iter().map(|a|if a.is_writable{AccountMeta::new(a.key(),a.is_signer)}else{AccountMeta::new_readonly(a.key(),a.is_signer)}));infos.extend_from_slice(remaining);invoke(&Instruction{program_id,accounts,data},&infos)?;Ok(())
}
pub use crate::instructions::fee_sharing::update_fee_shares::UpdateFeeSharesEvent;

#[derive(Accounts)]
pub struct UpdateFeeShares<'info>{
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
 pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=PUMP)]pub global:Account<'info,pump_global::Global>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Account<'info,SharingConfig>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP)]pub bonding_curve:Account<'info,BondingCurve>,
 /// CHECK: creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump,seeds::program=PUMP)]pub pump_creator_vault:AccountInfo<'info>,
 pub system_program:Program<'info,System>,
 pub pump_program:Program<'info,Pump>,
 /// CHECK: Pump event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=PUMP)]pub pump_event_authority:AccountInfo<'info>,
 pub pump_amm_program:Program<'info,Amm>,
 /// CHECK: AMM event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=AMM)]pub amm_event_authority:AccountInfo<'info>,
 #[account(address=anchor_spl::token::spl_token::native_mint::ID)]pub wsol_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(address=anchor_spl::token::ID)]pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 /// CHECK: AMM creator vault PDA.
 #[account(mut,seeds=[b"creator_vault",sharing_config.key().as_ref()],bump,seeds::program=AMM)]pub coin_creator_vault_authority:AccountInfo<'info>,
 /// CHECK: canonical token account address; dependency validates data.
 #[account(mut,address=anchor_spl::associated_token::get_associated_token_address_with_program_id(&coin_creator_vault_authority.key(),&anchor_spl::token::spl_token::native_mint::ID,&anchor_spl::token::ID))]pub coin_creator_vault_ata:AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct UpdateFeeSharesV2<'info>{
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
 #[account(mut)]pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=PUMP)]pub global:Box<Account<'info,pump_global::Global>>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Box<Account<'info,SharingConfig>>,
 #[account(seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP)]pub bonding_curve:Box<Account<'info,BondingCurve>>,
 /// CHECK: creator vault PDA.
 #[account(mut,seeds=[b"creator-vault",sharing_config.key().as_ref()],bump,seeds::program=PUMP)]pub pump_creator_vault:AccountInfo<'info>,
 /// CHECK: canonical token address, validated by dependency.
 #[account(mut,seeds=[pump_creator_vault.key().as_ref(),token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub pump_creator_vault_ata:AccountInfo<'info>,
 pub system_program:Program<'info,System>,
 pub pump_program:Program<'info,Pump>,
 /// CHECK: Pump event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=PUMP)]pub pump_event_authority:AccountInfo<'info>,
 pub pump_amm_program:Program<'info,Amm>,
 /// CHECK: AMM event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=AMM)]pub amm_event_authority:AccountInfo<'info>,
#[account(mint::token_program=token_program)]pub quote_mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
pub token_program:Interface<'info,anchor_spl::token_interface::TokenInterface>,
 pub associated_token_program:Program<'info,anchor_spl::associated_token::AssociatedToken>,
 /// CHECK: AMM creator vault PDA.
 #[account(mut,seeds=[b"creator_vault",sharing_config.key().as_ref()],bump,seeds::program=AMM)]pub coin_creator_vault_authority:AccountInfo<'info>,
 /// CHECK: canonical token account address; dependency validates data.
 #[account(mut,seeds=[coin_creator_vault_authority.key().as_ref(),token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token_program.key())]pub coin_creator_vault_ata:AccountInfo<'info>,
}

// Original [234096,235304): no compared-key payload on authorization failure.
use crate::instructions::fee_sharing::legacy_authorize;






pub type PreparedLegacyPayout<'info> = (Vec<Shareholder>, bool, Vec<AccountInfo<'info>>);





// Original [199072,204080). Per-entry availability precedes key and executable.
pub(crate) use crate::instructions::fee_sharing::update_fee_shares::prepare_legacy_payout;














































// Original [204080,205768). This is a legacy adapter, not the V2 caller.
pub fn legacy_payout_accounts<'info>(
    ctx: &mut Context<'_, '_, '_, 'info, UpdateFeeShares<'info>>,
) -> Result<Vec<AccountInfo<'info>>> { let state = &mut ctx.accounts.sharing_config; let remaining = ctx.remaining_accounts;
    let (shareholders, changed, accounts) = prepare_legacy_payout(&remaining, &state.shareholders)?;
    if !changed {
        // The original adapter makes a second copy before dropping prepared.accounts.
        Ok(remaining.to_vec())
    } else {
        state.shareholders = shareholders;
        state.exit(&crate::ID)?;
        Ok(accounts)
    }
}

// Original [205768,211408): scalar validation, then total, then duplicates.
fn legacy_validate_shares(shares: &[Shareholder]) -> Result<()> {
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
    Ok(())
}

// Original adapters own fixed accounts; generated CPI owns their final cleanup.
pub fn legacy_transfer_creator_fees_to_pump<'info>(
    curve: &Account<'info, BondingCurve>, program: AccountInfo<'info>, wsol_mint: AccountInfo<'info>, token_program: AccountInfo<'info>, system_program: AccountInfo<'info>, associated_token_program: AccountInfo<'info>,
    coin_creator: AccountInfo<'info>, coin_creator_vault_authority: AccountInfo<'info>, coin_creator_vault_ata: AccountInfo<'info>, pump_creator_vault: AccountInfo<'info>, event_authority: AccountInfo<'info>,
) -> Result<()> {
    if curve.real_sol_reserves == 0
        && curve.virtual_sol_reserves == 0
        && curve.real_token_reserves == 0
        && curve.virtual_token_reserves == 0
        && coin_creator_vault_ata.owner != &System::id()
    {
        let accounts = crate::pump_amm::cpi::accounts::TransferCreatorFeesToPump { program: program.clone(), wsol_mint, token_program, system_program, associated_token_program, coin_creator, coin_creator_vault_authority, coin_creator_vault_ata, pump_creator_vault, event_authority };
        return crate::pump_amm::cpi::transfer_creator_fees_to_pump(
            CpiContext::new(program, accounts),
        );
    }
    Ok(())
}

pub(crate) fn legacy_distribute_creator_fees<'info>(program: AccountInfo<'info>, mint: AccountInfo<'info>, bonding_curve: AccountInfo<'info>,
    sharing_config: AccountInfo<'info>, creator_vault: AccountInfo<'info>, system_program: AccountInfo<'info>, event_authority: AccountInfo<'info>, remaining: Vec<AccountInfo<'info>>) -> Result<()> {
    let accounts = crate::pump::cpi::accounts::DistributeCreatorFees {
        program: program.clone(), mint, bonding_curve, sharing_config, creator_vault, system_program, event_authority };
    let mut remaining = remaining;
    let ctx = CpiContext::new(program, accounts).with_remaining_accounts(std::mem::take(&mut remaining));
    crate::pump::cpi::distribute_creator_fees(ctx)?;
    Ok(())
}






































pub(crate) struct PreparedV2Payout<'info> {
    pub(crate) shareholders: Vec<Shareholder>,
    pub(crate) changed: bool,
    pub(crate) accounts: Vec<AccountInfo<'info>>,
}

// [219376,225920): two account blocks, not interleaved pairs.
pub(crate) use crate::instructions::fee_sharing::update_fee_shares_v2::prepare_v2_quote_payout;













































// Same owned zero-quote contract [199072,204080), without changing legacy code.
fn prepare_v2_zero_payout<'info>(
    remaining: &[AccountInfo<'info>], shareholders: &[Shareholder],
) -> Result<PreparedV2Payout<'info>> {
    let mut accounts = Vec::with_capacity(remaining.len());
    let mut retained = Vec::with_capacity(shareholders.len());
    let mut sum = 0u16;
    let mut max_weight = 0u16;
    let mut max_index = 0usize;
    for (i, share) in shareholders.iter().enumerate() {
        require!(i < remaining.len(), FeeError::NotEnoughRemainingAccounts);
        let account = &remaining[i];
        if account.key() != share.address {
            return Err(error!(FeeError::ShareholderAccountMismatch)
                .with_values((account.key(), share.address)));
        }
        if account.executable { continue; }
        retained.push(share.clone());
        accounts.push(account.clone());
        if share.share_bps > max_weight {
            max_weight = share.share_bps;
            max_index = retained.len() - 1;
        }
        sum = sum.saturating_add(share.share_bps);
    }
    if retained.len() == shareholders.len() {
        return Ok(PreparedV2Payout {
            shareholders: retained, changed: false, accounts: remaining.to_vec(),
        });
    }
    if retained.is_empty() {
        return Ok(PreparedV2Payout {
            shareholders: Vec::new(), changed: true, accounts: Vec::new(),
        });
    }
    v2_rescale_retained(&mut retained, sum, max_index);
    Ok(PreparedV2Payout { shareholders: retained, changed: true, accounts })
}

// Arithmetic contract shared by the two V2-owned preparation fragments.
// This factoring is a source hypothesis, not a recovered historical function.
fn v2_rescale_retained(retained: &mut [Shareholder], sum: u16, max_index: usize) {
    let mut normalized_sum = 0u16;
    for share in retained.iter_mut() {
        share.share_bps = if sum == 0 { 0 } else {
            ((u64::from(share.share_bps) * 10_000 / u64::from(sum)).min(10_000)) as u16
        };
        normalized_sum = normalized_sum.saturating_add(share.share_bps);
    }
    retained[max_index].share_bps = retained[max_index].share_bps
        .saturating_add(10_000u16.saturating_sub(normalized_sum));
}

// [212344,213328): data_len and infallible borrow, not try_borrow_data.
pub(crate) fn v2_expected_quote(info: &AccountInfo<'_>) -> Pubkey {
    let stored = if info.data_len() < 115 { Pubkey::default() } else {
        let data = info.data.borrow();
        Pubkey::new_from_array(data[83..115].try_into().unwrap())
    };
    if stored == Pubkey::default() {
        anchor_spl::token::spl_token::native_mint::ID
    } else { stored }
}

pub fn v2_transfer_creator_fees_to_pump<'info>(
    curve: &Account<'info, BondingCurve>, program: AccountInfo<'info>, quote_mint: AccountInfo<'info>, token_program: AccountInfo<'info>, system_program: AccountInfo<'info>, associated_token_program: AccountInfo<'info>,
    coin_creator: AccountInfo<'info>, coin_creator_vault_authority: AccountInfo<'info>, coin_creator_vault_ata: AccountInfo<'info>, pump_creator_vault: AccountInfo<'info>, pump_creator_vault_ata: AccountInfo<'info>, event_authority: AccountInfo<'info>, payer: &AccountInfo<'info>,
) -> Result<()> {
    if curve.real_sol_reserves == 0
        && curve.virtual_sol_reserves == 0
        && curve.real_token_reserves == 0
        && curve.virtual_token_reserves == 0
        && coin_creator_vault_ata.owner != &System::id()
    {
        let accounts = crate::pump_amm::cpi::accounts::TransferCreatorFeesToPumpV2 { payer: payer.clone(), quote_mint, token_program, system_program, associated_token_program, coin_creator, coin_creator_vault_authority, coin_creator_vault_ata, pump_creator_vault, pump_creator_vault_ata, event_authority, program: program.clone() };
        return crate::pump_amm::cpi::transfer_creator_fees_to_pump_v2(
            CpiContext::new(program, accounts),
        );
    }
    Ok(())
}

pub(crate) fn v2_distribute_creator_fees<'info>( program: AccountInfo<'info>, mint: AccountInfo<'info>, bonding_curve: AccountInfo<'info>, sharing_config: AccountInfo<'info>, creator_vault: AccountInfo<'info>, creator_vault_quote_token_account: AccountInfo<'info>, quote_mint: AccountInfo<'info>, quote_token_program: AccountInfo<'info>, associated_token_program: AccountInfo<'info>, system_program: AccountInfo<'info>, event_authority: AccountInfo<'info>, payer: AccountInfo<'info>, remaining: Vec<AccountInfo<'info>>,
) -> Result<()> {
    let accounts = crate::pump::cpi::accounts::DistributeCreatorFeesV2 {
        program: program.clone(), payer, mint, bonding_curve, sharing_config, creator_vault, system_program, event_authority, creator_vault_quote_token_account, quote_mint, quote_token_program, associated_token_program,
    };
    let mut remaining = remaining;
    let ctx = CpiContext::new(program, accounts).with_remaining_accounts(std::mem::take(&mut remaining));
    crate::pump::cpi::distribute_creator_fees_v2(ctx, true)?;
    Ok(())
}

// [225920,231576): scalar errors, then total, then duplicate count.
fn v2_validate_shares(shares: &[Shareholder]) -> Result<()> {
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
    Ok(())
}

