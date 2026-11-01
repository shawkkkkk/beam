use crate::*;

/// Deposits base and quote tokens into a pool in exchange for LP tokens.
///
/// The caller asks for an exact amount of LP tokens (`amount`) and provides
/// upper bounds for the base and quote tokens it is willing to pay
/// (`base_limit`, `quote_limit`). The required token amounts are derived from
/// the current pool reserves and LP supply, rounding up so that the pool is
/// never under-collateralised by a deposit.
///
/// Accounts (see [`Liquidity`]):
///  - `pool`: the pool being deposited into; owns both reserve accounts and
///    is the mint authority of the LP mint.
///  - `global_config`: global program configuration, used for the
///    deposit kill switch.
///  - `user`: the depositor, signs the token transfers.
///  - `base_mint` / `quote_mint`: the pool mints. The quote mint may be a
///    Token-2022 mint with a transfer fee extension, in which case the user
///    pays the gross amount so that the pool receives exactly the computed
///    quote amount.
///  - `lp_mint`: the pool LP mint (Token-2022).
///  - `user_base_token_account` / `user_quote_token_account`: source token
///    accounts of the depositor.
///  - `user_pool_token_account`: receives the newly minted LP tokens.
///  - `pool_base_token_account` / `pool_quote_token_account`: pool reserves.
///  - `token_program` / `token_2022_program`: token programs used for the
///    transfers, selected per mint owner.
///
/// Rounding:
///  - `base_amount_in  = ceil(pool_base_reserves  * lp_out / lp_supply)`
///  - `quote_amount_in = ceil(pool_quote_reserves * lp_out / lp_supply)`
///
/// For pools that can be boosted the virtual quote reserves are scaled by the
/// same LP ratio so that the pool price is unchanged by the deposit.
///
/// Emits [`DepositEvent`] both as a program log and through a self-CPI.
///
/// The reserves reported in the event are the balances observed before any
/// transfer of this instruction, `lp_mint_supply` is the supply before the
/// LP tokens of this deposit are minted.
///
/// Errors:
///  - [`PumpAmmError::DisabledDeposit`] when deposits are disabled.
///  - [`PumpAmmError::ExceededSlippage`] when either limit is exceeded.
///  - [`PumpAmmError::Truncation`] when a computed amount does not fit in u64.
///  - [`PumpAmmError::Overflow`] / [`PumpAmmError::DivisionByZero`] for
///    arithmetic failures.
pub fn deposit(
    ctx: Context<Liquidity>,
    amount: u64,
    base_limit: u64,
    quote_limit: u64,
) -> Result<()> {
    let a = &mut *ctx.accounts;

    require!(
        a.global_config.disable_flags & 2 == 0,
        PumpAmmError::DisabledDeposit
    );

    // Snapshot balances before any transfer, they are reported in the event.
    let user_base_token_reserves = a.user_base_token_account.amount;
    let user_quote_token_reserves = a.user_quote_token_account.amount;
    let base_reserve = a.pool_base_token_account.amount;
    let pool_quote_token_reserves = a.pool_quote_token_account.amount;

    // Accrued protocol / creator fees sitting in the quote vault are not
    // part of the liquidity reserves.
    let quote_reserve = liquidity_quote_reserves(&a.pool, pool_quote_token_reserves)?;

    let old_supply = a.pool.lp_supply;
    // base_amount = ceil(base_reserve * amount / lp_supply)
    let base_amount = u64::try_from(crate::curve::fees::ceil_div(
        (base_reserve as u128) * (amount as u128),
        old_supply as u128,
    )?)
    .map_err(|_| error!(PumpAmmError::Truncation))?;

    require_gte!(base_limit, base_amount, PumpAmmError::ExceededSlippage);

    // quote_amount = ceil(quote_reserve * amount / lp_supply)
    //
    // The same rounding as for the base side is used.
    // This is the amount the pool must receive. With a transfer fee on the
    // quote mint the user has to send a larger, fee-inclusive amount, which
    // is what the quote limit is checked against.
    let quote_amount = u64::try_from(crate::curve::fees::ceil_div(
        (quote_reserve as u128) * (amount as u128),
        old_supply as u128,
    )?)
    .map_err(|_| error!(PumpAmmError::Truncation))?;


    let fee_data = crate::utils::transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let quote_transfer_amount =
        crate::utils::transfer_fee::amount_before_fee(&fee_data, quote_amount)?;
    let quote_transfer_fee = quote_transfer_amount
        .checked_sub(quote_amount)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    require_gte!(quote_limit, quote_transfer_amount, PumpAmmError::ExceededSlippage);

    // Transfer base tokens from the user to the pool.
    let base_program = if a.base_mint.to_account_info().owner == a.token_program.key {
        a.token_program.as_ref()
    } else {
        a.token_2022_program.as_ref()
    };
    crate::utils::token::transfer_checked(
        base_program,
        &a.base_mint,
        &a.user_base_token_account,
        &a.pool_base_token_account,
        a.user.as_ref(),
        base_amount,
        &[],
    )?;

    // Transfer quote tokens (fee inclusive) from the user to the pool.
    let quote_program = if a.quote_mint.to_account_info().owner == a.token_program.key {
        a.token_program.as_ref()
    } else {
        a.token_2022_program.as_ref()
    };
    crate::utils::token::transfer_checked(
        quote_program,
        &a.quote_mint,
        &a.user_quote_token_account,
        &a.pool_quote_token_account,
        a.user.as_ref(),
        quote_transfer_amount,
        &[],
    )?;

    // Mint LP tokens to the user, signed by the pool.
    crate::utils::token::mint_to_pool(&a.token_2022_program, &a.lp_mint,
        &a.user_pool_token_account, amount, &a.pool)?;

    // The pool tracks the LP supply itself.
    a.pool.lp_supply = a
        .pool
        .lp_supply
        .checked_add(amount)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;

    if a.pool.can_boost() {
        // Scale the virtual quote reserves by the same LP ratio so that the
        // pool price is unchanged by the deposit:
        // change = virtual_reserve * amount / lp_supply (rounded down)
        let effective_virtual = liquidity_effective_virtual(&a.pool)?;

        let virtual_reserve = u128::try_from(effective_virtual)
            .map_err(|_| error!(PumpAmmError::Overflow))?;
        let change = virtual_reserve.checked_mul(amount as u128)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?
            .checked_div(old_supply as u128)
            .ok_or_else(|| error!(PumpAmmError::DivisionByZero))?;

        let virtual_quote_reserves = a.pool.virtual_quote_reserves;
        let change = i128::try_from(change)
            .map_err(|_| error!(PumpAmmError::Overflow))?;
        a.pool.virtual_quote_reserves = virtual_quote_reserves.checked_add(change)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    }

    let event = DepositEvent {
        timestamp: Clock::get()?.unix_timestamp,
        lp_token_amount_out: amount,
        max_base_amount_in: base_limit,
        max_quote_amount_in: quote_limit,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves: base_reserve,
        pool_quote_token_reserves,
        base_amount_in: base_amount,
        quote_amount_in: quote_amount,
        lp_mint_supply: old_supply,
        pool: a.pool.key(),
        user: a.user.key(),
        user_base_token_account: a.user_base_token_account.key(),
        user_quote_token_account: a.user_quote_token_account.key(),
        user_pool_token_account: a.user_pool_token_account.key(),
        quote_transfer_fee,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

#[event]
#[derive(Clone)]
pub struct DepositEvent {
    pub timestamp: i64,
    pub lp_token_amount_out: u64,
    pub max_base_amount_in: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_in: u64,
    pub quote_amount_in: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    pub quote_transfer_fee: u64,
}
