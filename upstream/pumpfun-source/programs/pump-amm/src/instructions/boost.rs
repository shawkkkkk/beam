use crate::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

/// Scale applied to `base_amount * quote_amount` when deriving the initial
/// virtual quote reserves of a boosted pool.
pub const VIRTUAL_QUOTE_RESERVES_DENOMINATOR: u128 = 1_000_000_000_000_000;

/// Computes the virtual quote reserves that seed a boost:
/// `base_amount * quote_amount / 10^15`.
///
/// Returns [`PumpAmmError::Truncation`] if the result does not fit in a
/// `u64`.
pub fn calculate_virtual_quote_reserves(
    base_amount: u64,
    quote_amount: u64,
) -> Result<u64> {
    let virtual_quote_reserves = (base_amount as u128)
        * (quote_amount as u128)
        / VIRTUAL_QUOTE_RESERVES_DENOMINATOR;

    u64::try_from(virtual_quote_reserves)
        .map_err(|_| error!(PumpAmmError::Truncation))
}

/// Accounts for `init_boost`.
#[event_cpi]
#[derive(Accounts)]
pub struct InitBoost<'info> {
    /// The canonical pool being boosted.
    #[account(
        mut,
        has_one = base_mint @ PumpAmmError::InvalidBaseMint,
        has_one = quote_mint @ PumpAmmError::InvalidQuoteMint,
        has_one = pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount,
        has_one = pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount,
    )]
    pub pool: Box<Account<'info, Pool>>,
    /// Global config holding the boost enable flag.
    #[account(
        seeds = [b"global_config"],
        bump,
    )]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    /// Pump pool authority that created the pool; pays for the boost vault.
    #[account(mut)]
    pub creator: Signer<'info>,
    /// Base mint of the pool.
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Quote mint of the pool.
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Pool's base token reserve account.
    pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Pool's quote token reserve account; the boost is funded from it.
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: PDA authority constrained by seeds.
    #[account(
        seeds = [b"boost_vault", pool.key().as_ref()],
        bump,
    )]
    pub boost_vault_authority: UncheckedAccount<'info>,
    /// Associated token account of the boost vault authority for the quote mint.
    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = quote_mint,
        associated_token::authority = boost_vault_authority,
        associated_token::token_program = quote_token_program,
    )]
    pub boost_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Token program of the quote mint.
    pub quote_token_program: Interface<'info, TokenInterface>,
    /// System program.
    pub system_program: Program<'info, System>,
    /// Associated token program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn execute(ctx: Context<InitBoost>) -> Result<()> {
    let a = &mut *ctx.accounts;
    let is_pool_authority = {
        let creator = a.creator.key();
        let authority = crate::utils::pool_authority(&a.pool.base_mint);
        creator == authority
    };
    let can_boost = a.pool.can_boost();
    if !is_pool_authority || can_boost || !a.global_config.boost_enabled {
        return Ok(());
    }
    let q = a.pool_quote_token_account.amount;
    let amount = calculate_virtual_quote_reserves(a.pool_base_token_account.amount, q)?;
    if amount == 0 || amount >= q {
        return Ok(());
    }
    let p = &a.pool;
    crate::utils::token::transfer_checked(
        &a.quote_token_program.to_account_info(),
        &a.quote_mint,
        &a.pool_quote_token_account,
        &a.boost_vault,
        &a.pool.to_account_info(),
        amount,
        &[&[
            b"pool",
            &p.index.to_le_bytes(),
            p.creator.as_ref(),
            p.base_mint.as_ref(),
            p.quote_mint.as_ref(),
            &[p.pool_bump],
        ]],
    )?;
    a.pool.virtual_quote_reserves = amount as i128;
    a.pool_quote_token_account.reload()?;
    let event = InitBoostEvent {
        timestamp: Clock::get()?.unix_timestamp,
        mint: a.pool.base_mint,
        bonding_curve: crate::utils::bonding_curve(&a.pool.base_mint),
        pool: a.pool.key(),
        virtual_quote_reserves: a.pool.virtual_quote_reserves,
        real_quote_reserves_after: a.pool_quote_token_account.amount,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
/// Emitted when a migrated pool is boosted.
#[event]
#[derive(Clone)]
pub struct InitBoostEvent {
    /// Unix timestamp of the boost.
    pub timestamp: i64,
    /// Base mint of the pool.
    pub mint: Pubkey,
    /// Pump bonding curve of the base mint.
    pub bonding_curve: Pubkey,
    /// The boosted pool.
    pub pool: Pubkey,
    /// Virtual quote reserves set on the pool.
    pub virtual_quote_reserves: i128,
    /// Pool quote reserves after funding the boost vault.
    pub real_quote_reserves_after: u64,
}

/// Accounts for `boost_buy_and_burn`.
#[event_cpi]
#[derive(Accounts)]
pub struct BoostBuyAndBurn<'info> {
    /// The boosted pool.
    #[account(
        has_one = base_mint @ PumpAmmError::InvalidBaseMint,
        has_one = quote_mint @ PumpAmmError::InvalidQuoteMint,
        has_one = pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount,
        has_one = pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount,
    )]
    pub pool: Box<Account<'info, Pool>>,
    /// Global boost authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Global config holding the boost authority and flags.
    #[account(
        seeds = [b"global_config"],
        bump,
    )]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    /// Base mint of the pool; its supply is reduced by the burn.
    #[account(mut)]
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Quote mint of the pool.
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Pool's base token reserve account; the bought base tokens are burned from it.
    #[account(mut)]
    pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Pool's quote token reserve account; receives the boost quote.
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Authority is bound to this pool by PDA seeds.
    #[account(
        seeds = [b"boost_vault", pool.key().as_ref()],
        bump,
    )]
    pub boost_vault_authority: UncheckedAccount<'info>,
    /// Boost vault holding the quote tokens used for the buy.
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = boost_vault_authority,
        associated_token::token_program = quote_token_program,
    )]
    pub boost_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Token program of the base mint.
    pub base_token_program: Interface<'info, TokenInterface>,
    /// Token program of the quote mint.
    pub quote_token_program: Interface<'info, TokenInterface>,
}

/// Emitted by `boost_buy_and_burn`.
#[event]
#[derive(Clone)]
pub struct BoostBuyAndBurnEvent {
    /// Unix timestamp of the buy and burn.
    pub timestamp: i64,
    /// Base mint of the pool.
    pub mint: Pubkey,
    /// Pump bonding curve of the base mint.
    pub bonding_curve: Pubkey,
    /// The boosted pool.
    pub pool: Pubkey,
    /// Boost authority that signed.
    pub authority: Pubkey,
    /// Quote amount requested by the authority.
    pub quote_amount_in_requested: u64,
    /// Quote amount taken from the boost vault.
    pub quote_amount_in_used: u64,
    /// Base amount bought and burned.
    pub base_amount_burned: u64,
    /// Virtual quote reserves of the pool.
    pub virtual_quote_reserves: i128,
    /// Pool quote reserves after the buy.
    pub real_quote_reserves_after: u64,
    /// Pool base reserves after the burn.
    pub base_reserves_after: u64,
    /// Quote left in the boost vault.
    pub boost_vault_remaining: u64,
}

/// Spends quote from the boost vault to buy base tokens from the pool and
/// burns them, permanently shrinking the base reserve.
pub fn buy_and_burn(
    ctx: Context<BoostBuyAndBurn>,
    requested: u64,
    minimum: u64,
) -> Result<()> {
    let a = &mut *ctx.accounts;
    let global_config = &a.global_config;

    global_config.validate_boost_authority(&a.authority.key())?;

    require!(global_config.boost_enabled, PumpAmmError::BoostDisabled);

    require!(
        global_config.disable_flags & 8 == 0,
        PumpAmmError::DisabledBuy
    );

    require!(a.pool.can_boost(), PumpAmmError::PoolCannotBoost);

    let old_vault = a.boost_vault.amount;
    let used = requested.min(old_vault);

    require_gt!(used, 0, PumpAmmError::ZeroQuoteAmount);

    let transfer_fee_data =
        crate::utils::transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let net = crate::utils::transfer_fee::amount_after_fee(&transfer_fee_data, used)?;
    require_gt!(net, 0, PumpAmmError::ZeroQuoteAmount);
    let transfer_fee = used
        .checked_sub(net)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;

    // Price the burn as a buy of `net` quote tokens against the pool's
    // effective reserves (real quote reserves plus virtual quote reserves).
    let old_quote = a.pool_quote_token_account.amount;
    a.pool.available_quote_reserves(old_quote)?;
    let old_base = a.pool_base_token_account.amount;
    let quote_reserve = a.pool.effective_quote_reserves(old_quote)?;

    let burned = crate::curve::constant_product::base_for_quote(
        net,
        old_base,
        quote_reserve,
    )?;

    require_gt!(burned, 0, PumpAmmError::ZeroBaseAmount);
    require_gte!(burned, minimum, PumpAmmError::ExceededSlippage);

    // The pool must keep some base liquidity after the burn: burning the
    // whole base reserve would leave the constant-product curve without a
    // price, and every later trade against this pool would fail.

    require_gt!(old_base, burned, PumpAmmError::TooLittlePoolTokenLiquidity);

    // Move the used quote from the boost vault into the pool.
    let pool_key = a.pool.key();
    crate::utils::token::transfer_checked(
        &a.quote_token_program,
        &a.quote_mint,
        &a.boost_vault,
        &a.pool_quote_token_account,
        &a.boost_vault_authority,
        used,
        &[&[
            b"boost_vault",
            pool_key.as_ref(),
            &[ctx.bumps.boost_vault_authority],
        ]],
    )?;

    // Burn the bought base tokens straight out of the pool, signed by the pool PDA.
    token_interface::burn(
        CpiContext::new_with_signer(
            a.base_token_program.to_account_info(),
            token_interface::Burn {
                mint: a.base_mint.to_account_info(),
                from: a.pool_base_token_account.to_account_info(),
                authority: a.pool.to_account_info(),
            },
            &[&[
                b"pool",
                &a.pool.index.to_le_bytes(),
                a.pool.creator.as_ref(),
                a.pool.base_mint.as_ref(),
                a.pool.quote_mint.as_ref(),
                &[a.pool.pool_bump],
            ]],
        ),
        burned,
    )?;

    // Refresh the balances changed by the CPIs above.
    a.pool_base_token_account.reload()?;
    a.pool_quote_token_account.reload()?;
    a.boost_vault.reload()?;
    a.base_mint.reload()?;

    let timestamp = Clock::get()?.unix_timestamp;


    // Emit a regular BuyEvent so indexers account for the boost like any
    // other buy against the pool.
    let event = crate::trade::BuyEvent {
        timestamp,
        base_amount_out: burned,
        max_quote_amount_in: requested,
        user_base_token_reserves: 0,
        user_quote_token_reserves: old_vault,
        pool_base_token_reserves: old_base,
        pool_quote_token_reserves: old_quote,
        quote_amount_in: net,
        lp_fee_basis_points: 0,
        lp_fee: 0,
        protocol_fee_basis_points: 0,
        protocol_fee: 0,
        quote_amount_in_with_lp_fee: net,
        user_quote_amount_in: net,
        pool: a.pool.key(),
        user: a.boost_vault_authority.key(),
        user_base_token_account: Pubkey::default(),
        user_quote_token_account: a.boost_vault.key(),
        protocol_fee_recipient: Pubkey::default(),
        protocol_fee_recipient_token_account: Pubkey::default(),
        coin_creator: a.pool.coin_creator,
        coin_creator_fee_basis_points: 0,
        coin_creator_fee: 0,
        track_volume: false,
        total_unclaimed_tokens: 0,
        total_claimed_tokens: 0,
        current_sol_volume: 0,
        last_update_timestamp: 0,
        min_base_amount_out: minimum,
        ix_name: "buy_exact_quote_in".to_string(),
        cashback_fee_basis_points: 0,
        cashback: 0,
        buyback_fee_basis_points: 0,
        buyback_fee: 0,
        virtual_quote_reserves: a.pool.virtual_quote_reserves,
        can_boost: a.pool.can_boost(),
        base_supply: if a.pool.is_mayhem_mode {
            1_000_000_000_000_000
        } else {
            a.base_mint.supply
        },
        holder_rewards_bps: 0,
        holder_rewards: 0,
        creator_fee_unclaimed: a.pool.creator_fees,
        quote_transfer_fee: transfer_fee,
    };
    emit!(event);
    emit_cpi!(event);

    let a = &mut *ctx.accounts;
    let event = BoostBuyAndBurnEvent {
        timestamp,
        mint: a.base_mint.key(),
        bonding_curve: crate::utils::bonding_curve(&a.base_mint.key()),
        pool: a.pool.key(),
        authority: a.authority.key(),
        quote_amount_in_requested: requested,
        quote_amount_in_used: used,
        base_amount_burned: burned,
        virtual_quote_reserves: a.pool.virtual_quote_reserves,
        real_quote_reserves_after: a.pool_quote_token_account.amount,
        base_reserves_after: a.pool_base_token_account.amount,
        boost_vault_remaining: a.boost_vault.amount,
    };
    emit!(event);
    emit_cpi!(event);

    // Close the boost vault once it is drained. Token-2022 withheld transfer
    // fees must be harvested to the mint before the account can be closed.
    let a = &mut *ctx.accounts;
    if a.boost_vault.amount == 0 {
        if transfer_fee_data.is_some() {
            require!(
                a.quote_mint.to_account_info().is_writable,
                PumpAmmError::QuoteMintNotWritable
            );
            crate::utils::transfer_fee::harvest_to_mint(
                &a.quote_token_program.to_account_info(),
                &a.quote_mint.to_account_info(),
                &a.boost_vault.to_account_info(),
            )?;
        }
        token_interface::close_account(CpiContext::new_with_signer(
            a.quote_token_program.to_account_info(),
            token_interface::CloseAccount {
                account: a.boost_vault.to_account_info(),
                destination: a.authority.to_account_info(),
                authority: a.boost_vault_authority.to_account_info(),
            },
            &[&[
                b"boost_vault",
                pool_key.as_ref(),
                &[ctx.bumps.boost_vault_authority],
            ]],
        ))?;
    }
    Ok(())
}
