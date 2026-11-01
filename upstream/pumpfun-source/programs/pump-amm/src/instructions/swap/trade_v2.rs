use anchor_lang::{
    accounts::{
        account::Account,
        interface::Interface,
        interface_account::InterfaceAccount,
        program::Program,
        signer::Signer,
        unchecked_account::UncheckedAccount,
    },
    prelude::{AccountInfo, Clock, Context, Id, Key, Pubkey, Rent, Result, SolanaSysvar, System, ToAccountInfo},
    AccountDeserialize, Accounts,
    emit, emit_cpi, error, require, require_gte, require_gt, require_keys_eq, require_keys_neq,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{
    curve::fees::AppliedFees,
    state::{GlobalConfig, Pool, UserVolumeAccumulator},
    utils::transfer_fee::{self, TransferFeeData},
    PumpAmmError, trade::{fees, prepare, buy_transfers},
};
pub use crate::pump_fees::{accounts::FeeConfig as TradeFeeConfig, types::{Fees as TradeFees, FeeTier as TradeFeeTier}};
pub const FEE_OWNER: Pubkey = anchor_lang::solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const PUMP: Pubkey = anchor_lang::solana_program::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub struct PumpAmmProgram;
impl Id for PumpAmmProgram {
    fn id() -> Pubkey {
        crate::ID
    }
}

#[derive(Accounts)]
pub struct TradeV2<'info> {
    #[account(mut, has_one = base_mint @ PumpAmmError::InvalidBaseMint, has_one = quote_mint @ PumpAmmError::InvalidQuoteMint, has_one = pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount, has_one = pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)] pub user: Signer<'info>,
    pub global_config: Box<Account<'info, GlobalConfig>>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)] pub user_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)] pub user_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)] pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)] pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    #[account(init_if_needed, payer = user, space = 137, seeds = [b"user_volume_accumulator", user.key().as_ref()], bump)]
    pub user_volume_accumulator: Box<Account<'info, UserVolumeAccumulator>>,
    #[account(seeds = [b"fee_config", crate::ID.as_ref()], bump = fee_config.bump, seeds::program = FEE_OWNER)]
    pub fee_config: Box<Account<'info, TradeFeeConfig>>,
    /// CHECK: Token owner, mint and associated address are checked by the handler.
    #[account(mut)] pub buyback_fee_recipient: UncheckedAccount<'info>,
    /// CHECK: PDA signer for self events.
    #[account(seeds=[b"__event_authority"], bump)] pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, PumpAmmProgram>,
}













































#[inline(never)]
pub fn validate_cashback_coin(is_cashback_coin: u8) -> Result<()> {
    require!(is_cashback_coin == 0, PumpAmmError::CashbackCoinNotSupported);
    Ok(())
}










#[inline(always)]
pub fn validate_multi_hop_pool(pool: &Pool, charges_creator_fee: bool) -> Result<()> {

    require!(pool.creator == crate::utils::pool_authority(&pool.base_mint), PumpAmmError::OnlyPumpPools);
    require!(!charges_creator_fee || !pool.is_cashback_coin, PumpAmmError::CashbackCoinNotSupported);
    // Mayhem pools route their fees through the Mayhem program, which a
    // multi-hop swap does not forward, so they are rejected even when this
    // hop charges no fees.
    require!(!pool.is_mayhem_mode, PumpAmmError::MayhemPoolNotSupported);
    Ok(())
}




#[inline(never)]
pub fn validate_buyback_token_account(
    global_config: &GlobalConfig,
    token_account: &AccountInfo,
    quote_mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<()> {
    require_keys_eq!(*token_account.owner, *token_program, PumpAmmError::AccountTypeNotSupported);
    let owner = {
        let data = token_account.try_borrow_data()?;
        let mut data_slice = data.as_ref();
        let state = TokenAccount::try_deserialize(&mut data_slice)?;



        require_keys_eq!(state.mint, *quote_mint, PumpAmmError::InvalidQuoteMint);
        state.owner
    };
    global_config.validate_buyback_recipient(&owner, &token_account.key(), quote_mint, token_program)
}


pub fn quote_reserves(pool: &Pool, raw_quote_reserves: u64) -> Result<(u64, u64)> {
    let available = pool.available_quote_reserves(raw_quote_reserves)?;
    let effective = pool.effective_quote_reserves(raw_quote_reserves)?;
    Ok((available, effective))
}

pub struct BuyResult {
    pub base_amount_out: u64,
    pub quote_amount_in: u64,
    pub fees: AppliedFees,
    pub user_quote_amount_in: u64,
    pub buyback_fee: u64,
    pub pool_quote_amount_in: u64,
    pub quote_transfer_fee: u64,
}






































































#[inline(never)]
fn total_quote(amounts: &[u64; 4]) -> Result<u64> {
    amounts.iter().try_fold(0u64, |total, amount| {
        total.checked_add(*amount).ok_or_else(|| error!(PumpAmmError::Overflow))
    })
}

#[inline(never)]
pub fn calculate_buy_result(
    buyback_basis_points: u64,
    is_mayhem_mode: u8,
    quote_transfer_fee: &TransferFeeData,
    base_amount_out: u64,
    quote_amount_in: u64,
    fees: AppliedFees,
) -> Result<BuyResult> {
    let user_quote_amount_in = {
        let amounts = [quote_amount_in, fees.lp_fee, fees.protocol_fee, fees.creator_fee];
        total_quote(&amounts)
    }?;
    let (buyback_fee, _) = GlobalConfig::split_buyback_fee(buyback_basis_points, fees.protocol_fee, is_mayhem_mode != 0);
    let net_pool_quote = user_quote_amount_in
        .checked_sub(buyback_fee)






















        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    let pool_quote_amount_in = transfer_fee::amount_before_fee(quote_transfer_fee, net_pool_quote)?;
    Ok(BuyResult {
        fees,
        base_amount_out,
        quote_amount_in,
        user_quote_amount_in,
        buyback_fee,
        pool_quote_amount_in,
        quote_transfer_fee: pool_quote_amount_in
            .checked_sub(net_pool_quote)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?,
    })
}


impl BuyResult {
    pub(crate) fn total_user_quote(&self) -> Result<u64> {
        self.pool_quote_amount_in.checked_add(self.buyback_fee)
            .ok_or_else(|| error!(PumpAmmError::Overflow))
    }
}
/// Initial exact-input quote and its fee amounts, before buyback/transfer-fee preview.
#[inline(never)]
pub fn estimate_buy_quote(pool: &Pool, selected: &AppliedFees, available: u64) -> Result<(u64, AppliedFees)> {
    let creator_bps = if pool.coin_creator == Pubkey::default() { 0 } else { selected.creator_fee_bps };
    let fee_bps = selected.protocol_fee_bps.saturating_add(selected.lp_fee_bps).saturating_add(creator_bps);
    let estimate = if fee_bps == 0 { available } else if available == 0 { 0 } else {
        // The denominator is at least 10_000, so this quotient is <= available.
        ((available as u128) * 10_000 / ((fee_bps as u128) + 10_000)) as u64
    };














    require_gt!(estimate, 0, PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees);
    let lp_fee = crate::curve::fees::fee(estimate, selected.lp_fee_bps)?;
    let protocol_fee = crate::curve::fees::fee(estimate, selected.protocol_fee_bps)?;
    let creator_fee = crate::curve::fees::fee(estimate, creator_bps)?;
    let required = {
        let amounts = [estimate, lp_fee, protocol_fee, creator_fee];
        total_quote(&amounts)
    }?;
    let mut quote = estimate;
    if required > available {
        quote = quote.checked_sub(required.saturating_sub(available))
            .ok_or(PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees)?;






        require_gt!(quote, 0, PumpAmmError::BuySlippageBelowMinBaseAmountOut);
    }
    return Ok((quote, AppliedFees { lp_fee_bps: selected.lp_fee_bps, protocol_fee_bps: selected.protocol_fee_bps,
        creator_fee_bps: selected.creator_fee_bps, lp_fee, protocol_fee, creator_fee }));
}

















/// Returns the quote after LP fees and the quote paid to the seller.
#[inline(never)]
pub fn calculate_sell_quote(
    quote_amount_out: u64,
    fees: &AppliedFees,
    real_quote_reserves: u64,
) -> Result<(u64, u64)> {
    let quote_amount_out_without_lp_fee = quote_amount_out.checked_sub(fees.lp_fee)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    require_gte!(real_quote_reserves, quote_amount_out_without_lp_fee,
        PumpAmmError::InsufficientRealQuoteReserves);




    let user_quote_amount_out = quote_amount_out_without_lp_fee.checked_sub(fees.creator_fee)
        .and_then(|amount| amount.checked_sub(fees.protocol_fee))
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    Ok((quote_amount_out_without_lp_fee, user_quote_amount_out))
}

pub struct TradeEventContext<'a, 'info> {
    pub pool: &'a Account<'info, Pool>,
    pub global_config: &'a GlobalConfig,
    pub base_supply: u64,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_volume_accumulator: &'a UserVolumeAccumulator,
}

#[inline(never)]
pub fn create_sell_event(
    context: TradeEventContext,
    base_amount: u64,
    limit: u64,
    user_base_reserves: u64,
    user_quote_reserves: u64,
    b: u64,
    raw_quote_reserves: u64,
    curve_quote: u64,
    selected: &AppliedFees,
    quote_without_lp_fee: u64,
    total_quote: u64,
    old_virtual: i128,
    buyback_received: u64,
    quote_transfer_fee: u64,
) -> Result<crate::trade::SellEvent> {
    let pool = context.pool;
    let global_config = context.global_config;
    let base_supply = context.base_supply;
    let user = context.user;
    let user_base_token_account = context.user_base_token_account;
    let user_quote_token_account = context.user_quote_token_account;
    let timestamp = Clock::get()?.unix_timestamp;
        let event = crate::trade::SellEvent {
            timestamp,
            base_amount_in: base_amount,
            min_quote_amount_out: limit,
            user_base_token_reserves: user_base_reserves,
            user_quote_token_reserves: user_quote_reserves,
            pool_base_token_reserves: b,
            pool_quote_token_reserves: raw_quote_reserves,
            quote_amount_out: curve_quote,
            lp_fee_basis_points: selected.lp_fee_bps,
            lp_fee: selected.lp_fee,
            protocol_fee_basis_points: selected.protocol_fee_bps,
            protocol_fee: selected.protocol_fee,
            quote_amount_out_without_lp_fee: quote_without_lp_fee,
            user_quote_amount_out: total_quote,
            pool: pool.key(),
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: pool.coin_creator,
            coin_creator_fee_basis_points: selected.creator_fee_bps,
            coin_creator_fee: selected.creator_fee,
            cashback_fee_basis_points: 0,
            cashback: 0,
            buyback_fee: buyback_received,
            virtual_quote_reserves: old_virtual,
            user,
            user_base_token_account,
            user_quote_token_account,
            holder_rewards_bps: if pool.is_holder_reward { selected.creator_fee_bps } else { 0 },
            holder_rewards: if pool.is_holder_reward { selected.creator_fee } else { 0 },
            buyback_fee_basis_points: if pool.is_mayhem_mode { 0 } else { global_config.buyback_basis_points },
            base_supply: if pool.is_mayhem_mode {1_000_000_000_000_000} else {base_supply},
            can_boost: pool.can_boost(),
            creator_fee_unclaimed: pool.creator_fees,
            quote_transfer_fee,
        };
    Ok(event)
}






























#[inline(never)]
pub fn create_buy_event(
    context: TradeEventContext,
    result: &BuyResult,
    max_quote_amount_in: u64,
    min_base_amount_out: u64,
    quote_amount_in: u64,
    user_quote_amount_in: u64,
    user_base_reserves: u64,
    user_quote_reserves: u64,
    b: u64,
    raw_quote_reserves: u64,
    old_virtual: i128,
    buyback_received: u64,
    ix_name: &str,
) -> Result<crate::trade::BuyEvent> {
    let pool = context.pool;
    let global_config = context.global_config;
    let base_supply = context.base_supply;
    let volume = context.user_volume_accumulator;
    let user = context.user;
    let user_base_token_account = context.user_base_token_account;
    let user_quote_token_account = context.user_quote_token_account;
    let quote_with_lp_fee = result.quote_amount_in.checked_add(result.fees.lp_fee)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    let total_unclaimed_tokens = volume.total_unclaimed_tokens;
    let total_claimed_tokens = volume.total_claimed_tokens;
    let current_sol_volume = volume.current_sol_volume;
    let last_update_timestamp = volume.last_update_timestamp;
    let timestamp = Clock::get()?.unix_timestamp;
        let event = crate::trade::BuyEvent {
            timestamp,
            base_amount_out: result.base_amount_out,
            max_quote_amount_in,
            user_base_token_reserves: user_base_reserves,
            user_quote_token_reserves: user_quote_reserves,
            pool_base_token_reserves: b,
            pool_quote_token_reserves: raw_quote_reserves,
            quote_amount_in,
            lp_fee_basis_points: result.fees.lp_fee_bps,
            lp_fee: result.fees.lp_fee,
            protocol_fee_basis_points: result.fees.protocol_fee_bps,
            protocol_fee: result.fees.protocol_fee,
            quote_amount_in_with_lp_fee: quote_with_lp_fee,
            user_quote_amount_in,
            pool: pool.key(),
            user,
            user_base_token_account,
            user_quote_token_account,
            protocol_fee_recipient: Pubkey::default(),
            protocol_fee_recipient_token_account: Pubkey::default(),
            coin_creator: pool.coin_creator,
            coin_creator_fee_basis_points: result.fees.creator_fee_bps,
            coin_creator_fee: result.fees.creator_fee,
            holder_rewards_bps: if pool.is_holder_reward { result.fees.creator_fee_bps } else { 0 },
            holder_rewards: if pool.is_holder_reward { result.fees.creator_fee } else { 0 },
            track_volume: false,
            total_unclaimed_tokens,
            total_claimed_tokens,
            current_sol_volume,
            last_update_timestamp,
            min_base_amount_out,
            ix_name: ix_name.to_string(),
            cashback_fee_basis_points: 0,
            cashback: 0,
            buyback_fee_basis_points: if pool.is_mayhem_mode { 0 } else { global_config.buyback_basis_points },
            base_supply: if pool.is_mayhem_mode { 1_000_000_000_000_000 } else { base_supply },
            buyback_fee: buyback_received,
            virtual_quote_reserves: old_virtual,
            can_boost: pool.can_boost(),
            creator_fee_unclaimed: pool.creator_fees,
            quote_transfer_fee: result.quote_transfer_fee,
        };
    Ok(event)
}





















pub struct BuyPreviewContext<'a, 'info> {
    pub pool: &'a Box<Account<'info, Pool>>,
    pub global_config: &'a Box<Account<'info, GlobalConfig>>,
    pub quote_transfer_fee: &'a TransferFeeData,
    pub fees: &'a AppliedFees,
}

#[inline(never)]
pub fn preview_total_quote(context: BuyPreviewContext, quote_amount_in: u64) -> Result<u64> {
    let global_config = context.global_config.as_ref();
    let pool = context.pool.as_ref();
    let quote_transfer_fee = context.quote_transfer_fee;
    let result = {
        let fees = context.fees.clone();
        calculate_buy_result(
            global_config.buyback_basis_points,
            pool.is_mayhem_mode as u8,
            quote_transfer_fee,
            0,
            quote_amount_in,
            fees,
        )
    }?;
    result.total_user_quote()
}

#[inline(never)]
pub fn execute_buy(ctx: Context<TradeV2>, amount: u64, limit: u64) -> Result<()> {
    let a = &mut *ctx.accounts;












































    require!(a.global_config.disable_flags & 8 == 0, PumpAmmError::DisabledBuy);
    validate_cashback_coin(a.pool.is_cashback_coin as u8)?;
    crate::instructions::swap::prepare_pool(&a.pool, a.system_program.as_ref(), a.user.as_ref())?;








    require_gt!(amount, 0, PumpAmmError::ZeroBaseAmount);
    require_gt!(limit, 0, PumpAmmError::ZeroQuoteAmount);
    validate_buyback_token_account(
        &a.global_config, a.buyback_fee_recipient.as_ref(), &a.quote_mint.key(), &a.quote_token_program.key(),
    )?;
    let user_base_reserves = a.user_base_token_account.amount;
    let user_quote_reserves = a.user_quote_token_account.amount;
    let b = a.pool_base_token_account.amount;
    let raw_quote_reserves = a.pool_quote_token_account.amount;
    let (_, q) = quote_reserves(&a.pool, raw_quote_reserves)?;

    let old_virtual = a.pool.virtual_quote_reserves;
    let quote = crate::curve::constant_product::quote_for_base(amount, b, q)?;
    let selected = fees(a, b, q, quote)?;
    let quote_fee_data = transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let result = calculate_buy_result(a.global_config.buyback_basis_points, a.pool.is_mayhem_mode as u8,
        &quote_fee_data, amount, quote, selected)?;
    let user_quote_amount_in = result.user_quote_amount_in;

















    require_gte!(limit, result.total_user_quote()?, PumpAmmError::ExceededSlippage);
    let buyback_fee = buy_transfers(a, &result)?;

    let event = create_buy_event(
        TradeEventContext {
            pool: &a.pool, global_config: &a.global_config,
            base_supply: a.base_mint.supply,
            user: a.user.key(), user_base_token_account: a.user_base_token_account.key(),
            user_quote_token_account: a.user_quote_token_account.key(),
            user_volume_accumulator: &a.user_volume_accumulator,
        }, &result, limit, amount, quote, user_quote_amount_in,
        user_base_reserves, user_quote_reserves, b, raw_quote_reserves,
        old_virtual, transfer_fee::amount_after_fee(&quote_fee_data, buyback_fee)?, "buy_v2",
    )?;
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

#[inline(never)]
pub fn execute_buy_exact_quote(ctx: Context<TradeV2>, amount: u64, limit: u64) -> Result<()> {
    let a = &mut *ctx.accounts;












    require!(a.global_config.disable_flags & 8 == 0, PumpAmmError::DisabledBuy);
    validate_cashback_coin(a.pool.is_cashback_coin as u8)?;
    crate::instructions::swap::prepare_pool(&a.pool, a.system_program.as_ref(), a.user.as_ref())?;









    require_gt!(limit, 0, PumpAmmError::ZeroBaseAmount);
    require_gt!(amount, 0, PumpAmmError::ZeroQuoteAmount);
    validate_buyback_token_account(
        &a.global_config, a.buyback_fee_recipient.as_ref(), &a.quote_mint.key(), &a.quote_token_program.key(),
    )?;
    let user_base_reserves = a.user_base_token_account.amount;
    let user_quote_reserves = a.user_quote_token_account.amount;
    let b = a.pool_base_token_account.amount;
    let raw_quote_reserves = a.pool_quote_token_account.amount;
    let (_, q) = quote_reserves(&a.pool, raw_quote_reserves)?;

    let old_virtual = a.pool.virtual_quote_reserves;
    let quote_fee_data = transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let available = transfer_fee::amount_after_fee(&quote_fee_data, amount)?;




    require_gt!(available, 0, PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees);
    let selected = fees(a, b, q, available)?;
    let (estimate, exact_fees) = estimate_buy_quote(&a.pool, &selected, available)?;
    let context = BuyPreviewContext { pool: &a.pool, global_config: &a.global_config,
        quote_transfer_fee: &quote_fee_data, fees: &exact_fees };
    let preview = preview_total_quote(context, estimate)?;
    let mut quote = estimate;
    if preview > amount {
        quote = estimate.checked_sub(preview.saturating_sub(amount))
            .ok_or(PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees)?;
























        require_gt!(quote, 0, PumpAmmError::BuySlippageBelowMinBaseAmountOut);
    }
    let out = crate::curve::constant_product::base_for_quote(quote, b, q)?;









    require_gte!(out, limit, PumpAmmError::BuySlippageBelowMinBaseAmountOut);




    require_gte!(b, out, PumpAmmError::TooLittlePoolTokenLiquidity);
    let result = buy_result(BuyPreviewContext { pool: &a.pool, global_config: &a.global_config,
        quote_transfer_fee: &quote_fee_data, fees: &exact_fees }, out, quote)?;
    let buyback_fee = buy_transfers(a, &result)?;

    let event = create_buy_event(
        TradeEventContext {
            pool: &a.pool, global_config: &a.global_config,
            base_supply: a.base_mint.supply,
            user: a.user.key(), user_base_token_account: a.user_base_token_account.key(),
            user_quote_token_account: a.user_quote_token_account.key(),
            user_volume_accumulator: &a.user_volume_accumulator,
        }, &result, amount, limit, amount, quote,
        user_base_reserves, user_quote_reserves, b, raw_quote_reserves,
        old_virtual, transfer_fee::amount_after_fee(&quote_fee_data, buyback_fee)?, "buy_exact_quote_in_v2",
    )?;
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

/// V2 sell execution follows the original transfer and state-update order.
#[inline(never)]
pub fn execute_sell(ctx: Context<TradeV2>, base_amount: u64, limit: u64) -> Result<()> {
    let a = &mut *ctx.accounts;



















    require!(a.global_config.disable_flags & 16 == 0, PumpAmmError::DisabledSell);
    validate_cashback_coin(a.pool.is_cashback_coin as u8)?;
    crate::instructions::swap::prepare_pool(&a.pool, a.system_program.as_ref(), a.user.as_ref())?;








    require_gt!(base_amount, 0, PumpAmmError::ZeroBaseAmount);
    validate_buyback_token_account(
        &a.global_config, a.buyback_fee_recipient.as_ref(), &a.quote_mint.key(), &a.quote_token_program.key(),
    )?;
    let user_base_reserves = a.user_base_token_account.amount;
    let user_quote_reserves = a.user_quote_token_account.amount;
    let b = a.pool_base_token_account.amount;
    let raw_quote_reserves = a.pool_quote_token_account.amount;
    let (real_quote_reserves, q) = quote_reserves(&a.pool, raw_quote_reserves)?;

    let old_virtual = a.pool.virtual_quote_reserves;
    let curve_quote = crate::curve::constant_product::quote_for_sell(base_amount, b, q)?;
    let selected = fees(a, b, q, curve_quote)?;
    let (quote_without_lp_fee, total_quote) = calculate_sell_quote(curve_quote, &selected, real_quote_reserves)?;
    let quote_fee_data = transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let quote_received = transfer_fee::amount_after_fee(&quote_fee_data, total_quote)?;














    require_gte!(quote_received, limit, PumpAmmError::ExceededSlippage);
    let quote_transfer_fee = total_quote.checked_sub(quote_received)





        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    crate::utils::token::transfer_checked(
        a.base_token_program.as_ref(), &a.base_mint, &a.user_base_token_account,
        &a.pool_base_token_account, a.user.as_ref(), base_amount, &[],
    )?;
    crate::utils::token::transfer_from_pool(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.pool_quote_token_account,
        &a.user_quote_token_account, total_quote, &a.pool,
    )?;
    let buyback_fee = a.pool.accrue_fees(
        &a.global_config, selected.protocol_fee, selected.creator_fee,
    )?;
    crate::utils::token::transfer_from_pool_to_raw(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.pool_quote_token_account,
        &a.pool, a.buyback_fee_recipient.as_ref(), buyback_fee,
    )?;
    set_accumulator_user(&mut a.user_volume_accumulator, a.user.as_ref());



    let event = create_sell_event(
        TradeEventContext {
            pool: &a.pool,
            global_config: &a.global_config,
            base_supply: a.base_mint.supply,
            user: a.user.key(),
            user_volume_accumulator: &a.user_volume_accumulator,
            user_base_token_account: a.user_base_token_account.key(),
            user_quote_token_account: a.user_quote_token_account.key(),
        },
        base_amount, limit, user_base_reserves, user_quote_reserves, b,
        raw_quote_reserves, curve_quote, &selected, quote_without_lp_fee, total_quote,
        old_virtual, transfer_fee::amount_after_fee(&quote_fee_data, buyback_fee)?, quote_transfer_fee,
    )?;
    emit!(event);
    emit_cpi!(event);
    Ok(())
}






















































































































































#[inline(always)]
pub fn set_accumulator_user(accumulator: &mut UserVolumeAccumulator, user: &AccountInfo) {
    if accumulator.user == Pubkey::default() {
        accumulator.user.as_mut().copy_from_slice(user.key.as_ref());
    }
}

#[inline(always)]
fn buy_result(context: BuyPreviewContext, base_amount_out: u64, quote_amount_in: u64) -> Result<BuyResult> {
    let global_config = context.global_config.as_ref();
    let pool = context.pool.as_ref();
    let quote_transfer_fee = context.quote_transfer_fee;
    let fees = context.fees.clone();
    calculate_buy_result(
        global_config.buyback_basis_points,
        pool.is_mayhem_mode as u8,
        quote_transfer_fee,
        base_amount_out,
        quote_amount_in,
        fees,
    )
}
