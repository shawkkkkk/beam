use crate::*;

const NORMAL_QUOTE: Pubkey = Pubkey::new_from_array([131,13,252,159,222,95,230,184,170,124,4,164,118,233,30,138,198,187,38,74,173,144,250,25,201,223,73,216,92,62,91,94]);
const STABLE_QUOTE: Pubkey = anchor_lang::solana_program::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

#[derive(Default, Clone, Copy)]
pub struct AppliedFees {
    pub lp_fee_bps: u64,
    pub protocol_fee_bps: u64,
    pub creator_fee_bps: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub creator_fee: u64,
}

pub(crate) fn ceil_div(numerator: u128, denominator: u128) -> anchor_lang::Result<u128> {
    numerator.checked_add(denominator).ok_or_else(|| error!(PumpAmmError::Overflow))?
        .checked_sub(1)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?
        .checked_div(denominator)
        .ok_or_else(|| error!(PumpAmmError::DivisionByZero))
}






pub(crate) fn fee(amount: u64, basis_points: u64) -> Result<u64> {
    let numerator = (amount as u128) * (basis_points as u128);
    let value = crate::curve::fees::ceil_div(numerator, 10000)?;
    u64::try_from(value).map_err(|_| error!(PumpAmmError::Truncation))
}


fn apply(pool: &Pool, amount: u64, lp: u64, protocol: u64, creator: u64) -> Result<AppliedFees> {
    let lp_fee = fee(amount, lp)?;
    let protocol_fee = fee(amount, protocol)?;
    let creator_fee = if pool.coin_creator != Pubkey::default() { fee(amount, creator)? } else { 0 };
    Ok(AppliedFees { lp_fee_bps: lp, protocol_fee_bps: protocol, creator_fee_bps: creator,
        lp_fee, protocol_fee, creator_fee })
}

pub fn select_and_apply<'info>(
    pool: &Account<'info, Pool>,
    global_config: &Account<'info, GlobalConfig>,
    fee_config: &Account<'info, trade::TradeFeeConfig>,
    fee_program: &Program<'info, legacy_sell::FeesProgram>,
    config_program: &AccountInfo<'info>,
    base_mint: &InterfaceAccount<'info, Mint>,
    base_reserve: u64,
    quote_reserve: u64,
    quote_amount: u64,
) -> Result<AppliedFees> {
    let context = CpiContext::new(fee_program.to_account_info(), crate::pump_fees::cpi::accounts::GetFeesWithQuoteMint {
        fee_config: fee_config.to_account_info(), config_program_id: config_program.clone(),
    });
    let mut selected = crate::pump_fees::cpi::get_fees_with_quote_mint(
        context,
        pool.creator.as_ref() == crate::utils::pool_authority(&pool.base_mint).as_ref(),
        pool.market_cap(base_mint.supply, base_reserve, quote_reserve)?,
        pool.quote_mint,
    )?.get();
    if global_config.creator_fee_configurable && pool.creator_fee_bps != 0 {
        selected.creator_fee_bps = pool.creator_fee_bps;
    }
    apply(pool, quote_amount, selected.lp_fee_bps, selected.protocol_fee_bps, selected.creator_fee_bps)
}


























































































fn tier(tiers: &[trade::TradeFeeTier], cap: u128) -> Result<&trade::TradeFees> {
    require!(!tiers.is_empty(), PumpAmmError::FeeTiersEmpty);
    if cap < tiers[0].market_cap_lamports_threshold {
        Ok(&tiers[0].fees)
    } else {
        Ok(&tiers.iter().rev().find(|t| cap >= t.market_cap_lamports_threshold).unwrap_or(&tiers[0]).fees)
    }
}

fn select_v2_fees(
    pool: &Pool,
    config: &trade::TradeFeeConfig,
    fee_config_len: usize,
    canonical: bool,
    cap: u128,
) -> Result<trade::TradeFees> {









    require!(fee_config_len >= 2512, PumpAmmError::FeeConfigTooShort);
    let selected = if !canonical {
        &config.flat_fees
    } else {
        let quote = &pool.quote_mint;
        match quote {
            q if q == &anchor_spl::token::spl_token::native_mint::ID || q == &NORMAL_QUOTE => tier(&config.fee_tiers, cap)?,
            q if q == &STABLE_QUOTE => {
                let tiers: &[trade::TradeFeeTier] = if fee_config_len >= 4073 { &config.stable_fee_tiers } else { &[] };
                tier(tiers, cap)?
            }
            _ if fee_config_len >= 4097 && (config.exotic_flat_fees.lp_fee_bps != 0 || config.exotic_flat_fees.protocol_fee_bps != 0 || config.exotic_flat_fees.creator_fee_bps != 0) => &config.exotic_flat_fees,
            _ => &config.flat_fees,
        }
    };
    Ok(selected.clone())
}

pub fn select_and_apply_v2(
    pool: &Pool,
    global_config: &GlobalConfig,
    config: &trade::TradeFeeConfig,
    fee_config_len: usize,
    base_supply: u64,
    base_reserve: u64,
    quote_reserve: u64,
    quote_amount: u64,
) -> Result<AppliedFees> {
    let cap = pool.market_cap(base_supply, base_reserve, quote_reserve)?;
    let canonical = {
        let authority = Pubkey::find_program_address(&[&crate::utils::POOL_AUTHORITY_SEED, pool.base_mint.as_ref()], &trade::PUMP).0;
        pool.creator == authority
    };
    let selected = select_v2_fees(pool, config, fee_config_len, canonical, cap)?;
    apply_selected_v2(pool, global_config, quote_amount, selected)
}

fn apply_selected_v2(pool: &Pool, global_config: &GlobalConfig, quote_amount: u64, mut selected: trade::TradeFees) -> Result<AppliedFees> {
    if global_config.creator_fee_configurable && pool.creator_fee_bps != 0 {
        selected.creator_fee_bps = pool.creator_fee_bps;
    }
    apply(pool, quote_amount, selected.lp_fee_bps, selected.protocol_fee_bps, selected.creator_fee_bps)
}
