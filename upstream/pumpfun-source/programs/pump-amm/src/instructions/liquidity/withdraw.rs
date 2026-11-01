use crate::*;

/// Withdraws base and quote tokens from a pool by burning LP tokens.
///
/// The caller burns an exact amount of LP tokens (`amount`) and provides
/// lower bounds for the base and quote tokens it expects to receive
/// (`base_limit`, `quote_limit`). The token amounts are derived from the
/// current pool reserves and LP supply, rounding down so that a withdrawal
/// can never take more than its share of the reserves.
///
/// Accounts (see [`Liquidity`]):
///  - `pool`: the pool being withdrawn from; owns both reserve accounts and
///    signs the reserve transfers.
///  - `global_config`: global program configuration, used for the
///    withdraw kill switch.
///  - `user`: the LP token holder, signs the LP burn.
///  - `base_mint` / `quote_mint`: the pool mints. The quote mint may be a
///    Token-2022 mint with a transfer fee extension, in which case the user
///    receives the computed quote amount minus the transfer fee.
///  - `lp_mint`: the pool LP mint (Token-2022).
///  - `user_base_token_account` / `user_quote_token_account`: destination
///    token accounts of the user.
///  - `user_pool_token_account`: LP tokens are burned from this account.
///  - `pool_base_token_account` / `pool_quote_token_account`: pool reserves.
///  - `token_program` / `token_2022_program`: token programs used for the
///    transfers, selected per mint owner.
///
/// Rounding:
///  - `base_amount_out  = floor(pool_base_reserves  * lp_in / lp_supply)`
///  - `quote_amount_out = floor(pool_quote_reserves * lp_in / lp_supply)`
///
/// For pools that can be boosted the virtual quote reserves are scaled by the
/// same LP ratio (rounded up) so that the pool price is unchanged.
///
/// Emits [`WithdrawEvent`] both as a program log and through a self-CPI.
///
/// The reserves reported in the event are the balances observed before any
/// transfer of this instruction, `lp_mint_supply` is the supply before the
/// LP tokens of this withdrawal are burned.
///
/// Errors:
///  - [`PumpAmmError::DisabledWithdraw`] when withdrawals are disabled.
///  - [`PumpAmmError::SeedLockViolation`] when the pool LP supply is below
///    the LP mint supply.
///  - [`PumpAmmError::ExceededSlippage`] when either limit is not met.
///  - [`PumpAmmError::Truncation`] when a computed amount does not fit in u64.
///  - [`PumpAmmError::Overflow`] for arithmetic failures, including a pool
///    whose LP supply would underflow.
pub fn withdraw(
    ctx: Context<Liquidity>,
    amount: u64,
    base_limit: u64,
    quote_limit: u64,
) -> Result<()> {
    let a = &mut *ctx.accounts;

    require!(
        a.global_config.disable_flags & 4 == 0,
        PumpAmmError::DisabledWithdraw
    );

    require_gte!(
        a.pool.lp_supply,
        a.lp_mint.supply,
        PumpAmmError::SeedLockViolation
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

    // base_amount = floor(base_reserve * amount / lp_supply)
    let base_amount = u64::try_from(crate::curve::constant_product::liquidity_floor_ratio(
        base_reserve, amount, old_supply,
    )?)
    .map_err(|_| error!(PumpAmmError::Truncation))?;

    require_gte!(base_amount, base_limit, PumpAmmError::ExceededSlippage);

    // quote_amount = floor(quote_reserve * amount / lp_supply)
    //
    // The same rounding as for the base side is used.
    //
    // This is the amount leaving the pool. With a transfer fee on the quote
    // mint the user receives less, which is what the quote limit is checked
    // against.
    let quote_amount = u64::try_from(crate::curve::constant_product::liquidity_floor_ratio(
        quote_reserve, amount, old_supply,
    )?)
    .map_err(|_| error!(PumpAmmError::Truncation))?;


    let fee_data = crate::utils::transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let quote_received =
        crate::utils::transfer_fee::amount_after_fee(&fee_data, quote_amount)?;
    let quote_transfer_fee = quote_amount
        .checked_sub(quote_received)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    require_gte!(quote_received, quote_limit, PumpAmmError::ExceededSlippage);

    // Transfer base tokens from the pool to the user, signed by the pool.
    // The token program is selected by the owner of the mint.
    let base_program = if a.base_mint.to_account_info().owner == a.token_program.key {
        a.token_program.as_ref()
    } else {
        a.token_2022_program.as_ref()
    };
    crate::utils::token::transfer_from_pool(
        base_program,
        &a.base_mint,
        &a.pool_base_token_account,
        &a.user_base_token_account,
        base_amount,
        &a.pool,
    )?;

    // Transfer quote tokens from the pool to the user.
    let quote_program = if a.quote_mint.to_account_info().owner == a.token_program.key {
        a.token_program.as_ref()
    } else {
        a.token_2022_program.as_ref()
    };
    crate::utils::token::transfer_from_pool(
        quote_program,
        &a.quote_mint,
        &a.pool_quote_token_account,
        &a.user_quote_token_account,
        quote_amount,
        &a.pool,
    )?;

    // Burn the LP tokens of the user. The LP mint is always a Token-2022
    // mint whose authority is the pool.
    crate::utils::token::burn_from_user(&a.token_2022_program, &a.lp_mint,
        &a.user_pool_token_account, a.user.as_ref(), amount)?;

    // The pool tracks the LP supply itself.
    a.pool.lp_supply = a
        .pool
        .lp_supply
        .checked_sub(amount)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;

    if a.pool.can_boost() {
        // Scale the virtual quote reserves by the same LP ratio so that the
        // pool price is unchanged by the withdrawal:
        // change = virtual_reserve * amount / lp_supply (rounded up)
        //
        // Rounding up keeps the remaining virtual reserves on the safe side.
        let change = crate::curve::fees::ceil_div(
            u128::try_from(liquidity_effective_virtual(&a.pool)?)
                .map_err(|_| error!(PumpAmmError::Overflow))?
                .checked_mul(amount as u128)
                .ok_or_else(|| error!(PumpAmmError::Overflow))?,
            old_supply as u128,
        )?;

        let change = u64::try_from(change).map_err(|_| error!(PumpAmmError::Truncation))?;
        let change = i128::from(change);

        a.pool.virtual_quote_reserves = a.pool.virtual_quote_reserves.checked_sub(change)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    }

    let event = WithdrawEvent {
        timestamp: Clock::get()?.unix_timestamp,
        lp_token_amount_in: amount,
        min_base_amount_out: base_limit,
        min_quote_amount_out: quote_limit,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves: base_reserve,
        pool_quote_token_reserves,
        base_amount_out: base_amount,
        quote_amount_out: quote_amount,
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
pub struct WithdrawEvent {
    pub timestamp: i64,
    pub lp_token_amount_in: u64,
    pub min_base_amount_out: u64,
    pub min_quote_amount_out: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub base_amount_out: u64,
    pub quote_amount_out: u64,
    pub lp_mint_supply: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub user_pool_token_account: Pubkey,
    pub quote_transfer_fee: u64,
}
