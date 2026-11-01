//! Partial reconstruction from captured AMM execution and static control flow.
//! Ordinary signed offline fixtures cover AMM legs and genuine captured Pump CPIs;
//! this is not the historical source or a claim of whole-program equivalence.
use anchor_lang::{
    accounts::{
        account::Account,
        interface_account::InterfaceAccount,
        program::Program,
        signer::Signer,
        unchecked_account::UncheckedAccount,
    },
    prelude::{
        AccountInfo, Context, CpiContext, Id, Key, Pubkey, Rent, Result,
        SolanaSysvar, System, ToAccountInfo,
    },
    AccountDeserialize, Accounts, AccountsExit,
    emit, emit_cpi, err, error, require, require_eq, require_gt, require_gte,
    require_keys_eq, require_keys_neq,
};
use anchor_spl::{
    token::Token,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use crate::{
    instructions::swap::trade_v2::{self, BuyResult, TradeEventContext},
    state::{GlobalConfig, Pool, UserVolumeAccumulator},
    trade::{FEE_OWNER, PUMP, PumpAmmProgram, TradeFeeConfig},
    utils::transfer_fee,
    PumpAmmError,
};
use std::collections::BTreeSet;



/// Bonding-curve program (`6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`):
/// the `declare_program!(pump)` program type, so `Program<PumpProgram>`
/// checks the account key against the declared program id
/// (same address as `trade::PUMP`).
///
pub use crate::pump::program::Pump as PumpProgram;

#[derive(Accounts)]
pub struct AmmHop<'info> {
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut,
        has_one = base_mint @ PumpAmmError::InvalidBaseMint,
        has_one = quote_mint @ PumpAmmError::InvalidQuoteMint,
        has_one = pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount,
        has_one = pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount,
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

/// Fee endpoints carried together through an AMM leg.
#[derive(Clone, Copy)]
pub struct HopFeeGates {
    pub protocol: bool,
    pub creator: bool,
    pub lp: bool,
}

#[derive(Accounts)]
pub struct MultiHop<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_in_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_out_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(seeds=[b"global_config"],bump)]
    pub global_config: Box<Account<'info, GlobalConfig>>,
    #[account(seeds=[b"fee_config",crate::ID.as_ref()],bump=fee_config.bump,seeds::program=FEE_OWNER)]
    pub fee_config: Box<Account<'info, TradeFeeConfig>>,
    #[account(init_if_needed,payer=user,space=137,seeds=[b"user_volume_accumulator",user.key().as_ref()],bump)]
    pub user_volume_accumulator: Box<Account<'info, UserVolumeAccumulator>>,
    /// CHECK: Validated at a protocol-fee endpoint or by the captured Pump CPI.
    #[account(mut)]
    pub buyback_fee_recipient: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    /// CHECK: Self-event PDA.
    #[account(seeds=[b"__event_authority"],bump)]
    pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, PumpAmmProgram>,
    pub pump_program: Program<'info, PumpProgram>,
    /// CHECK: Forwarded unchanged to Pump, which validates its context.
    pub pump_global: UncheckedAccount<'info>,
    /// CHECK: Forwarded unchanged to Pump, which validates its context.
    pub pump_fee_config: UncheckedAccount<'info>,
    /// CHECK: Forwarded unchanged to Pump, which validates its context.
    pub pump_event_authority: UncheckedAccount<'info>,
}

pub struct HopFeeContext<'a, 'info> {
    pub pool: &'a Box<Account<'info, Pool>>,
    pub global_config: &'a Account<'info, GlobalConfig>,
    pub accounts: &'a MultiHop<'info>,
    pub fee_config_len: &'a usize,
    pub base_supply: &'a u64,
    pub base_reserve: &'a u64,
    pub quote_reserve: &'a u64,
    pub gates: &'a HopFeeGates,
}

fn token_amount(a: &AccountInfo) -> Result<u64> {
    let data = a.try_borrow_data()?;
    let account = TokenAccount::try_deserialize(&mut &data[..])?;
    Ok(account.amount)
}

/// Where the tokens entering the next hop currently are.
pub enum HopSource<'info> {
    /// The user's input token account.
    User,
    /// A vault of the previous AMM hop's pool (`true`: its base vault).
    Pool(Box<AmmHop<'info>>, bool),
    /// Already delivered into the next hop's vault by a curve run.
    Curve,
}



































































#[inline(never)]
pub fn apply_hop_fees(context: &HopFeeContext, amount: u64) -> Result<crate::curve::fees::AppliedFees> {
    crate::curve::fees::select_and_apply_v2(
        context.pool, context.global_config, &context.accounts.fee_config,
        *context.fee_config_len, *context.base_supply, *context.base_reserve,
        *context.quote_reserve, amount,
    )
    .map(|fees| gate_fees(fees, *context.gates))
}

/// Zero the fee legs this hop does not charge.
fn gate_fees(fees: crate::curve::fees::AppliedFees, gates: HopFeeGates) -> crate::curve::fees::AppliedFees {
    crate::curve::fees::AppliedFees {
        lp_fee_bps: if gates.lp { fees.lp_fee_bps } else { 0 },
        protocol_fee_bps: if gates.protocol { fees.protocol_fee_bps } else { 0 },
        creator_fee_bps: if gates.creator { fees.creator_fee_bps } else { 0 },
        lp_fee: if gates.lp { fees.lp_fee } else { 0 },
        protocol_fee: if gates.protocol { fees.protocol_fee } else { 0 },
        creator_fee: if gates.creator { fees.creator_fee } else { 0 },
    }
}


/// Kind of venue a hop trades on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HopKind {
    Amm,
    Curve,
}







/// Classify a hop by its pool account owner: PumpSwap pools are AMM hops,
/// Pump bonding curves are curve hops.
fn hop_kind(accounts: &[AccountInfo]) -> Result<HopKind> {
    let owner = accounts[2].owner;
    if *owner == crate::ID {
        Ok(HopKind::Amm)
    } else if *owner == PUMP {
        Ok(HopKind::Curve)
    } else {
        err!(PumpAmmError::AccountTypeNotSupported)
    }
}
/// Output mint of a hop entered with `mint`.
fn next_mint(mint: Pubkey, base_mint: Pubkey, quote_mint: Pubkey) -> Result<Pubkey> {


    if mint == quote_mint {
        Ok(base_mint)
    } else if mint == base_mint {
        Ok(quote_mint)
    } else {
        err!(PumpAmmError::MultiHopDiscontinuousPath)
    }
}












/// Route `amount_in` of the user's input token through every hop in
/// `remaining_accounts` and require at least `min_amount_out` of the output token.
pub fn execute<'info>(
    ctx: Context<'_, '_, 'info, 'info, MultiHop<'info>>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    require_gt!(amount_in, 0, PumpAmmError::ZeroQuoteAmount);
    require_gt!(min_amount_out, 0, PumpAmmError::ZeroBaseAmount);
    let remaining = ctx.remaining_accounts;
    let hops = remaining.chunks_exact(5);

    require!(!remaining.is_empty() && hops.remainder().is_empty(), PumpAmmError::NotEnoughRemainingAccounts);
    let hop_count = hops.len();
    let mut output_mints = Vec::with_capacity(hop_count);
    let mut mint = ctx.accounts.user_in_token_account.mint;
    let mut direction = None;
    for hop in hops {
        let quote_mint = hop[1].key();
        let next = next_mint(mint, hop[0].key(), hop[1].key())?;
        let buy = mint == quote_mint;
        match direction {
            None => direction = Some(buy),






            Some(direction) => require!(direction == buy, PumpAmmError::MultiHopMixedDirection),
        }
        mint = next;
        output_mints.push(mint);
    }
    require_keys_eq!(
        mint,
        ctx.accounts.user_out_token_account.mint,
        PumpAmmError::MultiHopDiscontinuousPath
    );


    let buy = direction.ok_or_else(|| error!(PumpAmmError::NotEnoughRemainingAccounts))?;
    let mut mint = ctx.accounts.user_in_token_account.mint;
    let mut amount = amount_in;
    let mut source = HopSource::User;
    let mut i = 0;
    while i < hop_count {
        let start = i.saturating_mul(5);
        //
       let hop = &remaining[start..start.saturating_add(5)];
        match hop_kind(hop)? {
            HopKind::Amm => {

            let mut accounts = hop;
            let mut hop = AmmHop::try_accounts(
                &crate::ID,
                &mut accounts,
                &[],
                &mut AmmHopBumps {},
                &mut BTreeSet::new(),
            )?;
            let last = i + 1 == hop_count;
            let gates = HopFeeGates {
                protocol: if buy { i == 0 } else { last },
                creator: if buy { last } else { i == 0 },
                lp: if buy { last } else { i == 0 },
            };
            let (output, output_is_base) =
                amm_hop(&ctx, &mut hop, &mint.key(), amount, &source, if last { Some(min_amount_out) } else { None }, gates)?;
            amount = output;
            hop.exit(&crate::ID)?;
            source = HopSource::Pool(Box::new(hop), output_is_base);
            //
            //
            //
            //
            //
            //
            mint = output_mints[i];
            i += 1;
            }
            HopKind::Curve => {
            let mut end = i + 1;
            //
            //
            //
            while end < hop_count
                 && hop_kind(&remaining[end.saturating_mul(5)..end.saturating_mul(5).saturating_add(5)])? == HopKind::Curve
            {
                end = end.saturating_add(1);
            }
            //
            //
            let hops = &remaining[start..end.saturating_mul(5)];
            let flags: Vec<u8> = (i..end)
                .map(|j| match (buy, j == 0, j + 1 == hop_count) { (true, first, last) => (first as u8) | ((last as u8) << 1), (false, first, last) => (last as u8) | ((first as u8) << 1) })
                .collect();
            let last_mint = output_mints[end.saturating_sub(1)];
            let destination = if end == hop_count {
                ctx.accounts.user_out_token_account.to_account_info()
            } else {
                //
                vault_for_mint(
                     &remaining[end.saturating_mul(5)..end.saturating_mul(5).saturating_add(5)],
                    &last_mint.key(),
                )?
            };
            amount = curve_run(&ctx, hops, flags, amount, &source, destination, buy)?;
            mint = last_mint;
            source = HopSource::Curve;
            i = end;
            }
        }
    }









    require_gte!(amount, min_amount_out, PumpAmmError::ExceededSlippage);
    trade_v2::set_accumulator_user(&mut ctx.accounts.user_volume_accumulator, ctx.accounts.user.as_ref());


    Ok(())
}








/// Token program that owns `mint`.
fn program_for<'info>(
    a: &MultiHop<'info>,
    mint: &InterfaceAccount<'info, Mint>,
) -> Result<AccountInfo<'info>> {
    let owner = mint.to_account_info().owner;
    if *owner == anchor_spl::token_2022::ID {
        Ok(a.token_2022_program.to_account_info())
    } else if *owner == anchor_spl::token::ID {
        Ok(a.token_program.to_account_info())
    } else {
        err!(PumpAmmError::AccountTypeNotSupported)
    }
}
#[inline(never)]
fn amm_hop<'info>(
    ctx: &Context<'_, '_, 'info, 'info, MultiHop<'info>>,
    hop: &mut AmmHop<'info>,
    input_mint: &Pubkey,
    amount: u64,
    source: &HopSource<'info>,

    minimum: Option<u64>,
    gates: HopFeeGates,
) -> Result<(u64, bool)> {
    let a = &ctx.accounts;
    let buy = *input_mint == hop.quote_mint.key();
    if buy {






































        require!(
            a.global_config.disable_flags & 8 == 0,
            PumpAmmError::DisabledBuy
        );
    } else {
        require!(
            a.global_config.disable_flags & 16 == 0,
            PumpAmmError::DisabledSell
        );
    }
    trade_v2::validate_multi_hop_pool(&hop.pool, gates.creator)?;
    crate::instructions::swap::prepare_pool(&hop.pool, a.system_program.as_ref(), a.user.as_ref())?;
    let base_program = program_for(a, &hop.base_mint)?;
    let quote_program = program_for(a, &hop.quote_mint)?;
    if gates.protocol {
        trade_v2::validate_buyback_token_account(
            &a.global_config,
            a.buyback_fee_recipient.as_ref(),
            &hop.quote_mint.key(),
            &quote_program.key(),
        )?;
    }
    let (input_program, input_mint, input_vault) = if buy {
        (&quote_program, &hop.quote_mint, &hop.pool_quote_token_account)
    } else {
        (&base_program, &hop.base_mint, &hop.pool_base_token_account)
    };
    let (user_input_account, user_input_balance) = match source {
        HopSource::User => (
            a.user_in_token_account.key(),
            a.user_in_token_account.amount,
        ),
        _ => (Pubkey::default(), 0),
    };

    let (user_output_account, user_output_balance) = match minimum {
        Some(_) => (
            a.user_out_token_account.key(),
            token_amount(&a.user_out_token_account.to_account_info())?,
        ),
        None => (Pubkey::default(), 0),
    };
    match source {
        HopSource::User => crate::utils::token::transfer_checked(
            input_program,
            input_mint,
            &a.user_in_token_account,
            input_vault,
            &a.user.to_account_info(),
            amount,
            &[],
        )?,
        HopSource::Pool(previous, base) => {
            let vault = if *base {
                &previous.pool_base_token_account
            } else {
                &previous.pool_quote_token_account
            };
            if vault.key() != input_vault.key() {
                crate::utils::token::transfer_from_pool(
                    input_program,
                    input_mint,
                    vault,
                    input_vault,
                    amount,
                    &previous.pool,
                )?;
            }
        }
        HopSource::Curve => {}
    }
    hop.pool_base_token_account.reload()?;
    hop.pool_quote_token_account.reload()?;
    let quote_fee = transfer_fee::from_mint(&hop.quote_mint.to_account_info())?;
    let received = if buy { transfer_fee::amount_after_fee(&quote_fee, amount)? } else { amount };
    let (base_reserve, raw_quote_reserve) = if buy {
        let quote = hop.pool_quote_token_account.amount;

































        let quote = quote.checked_sub(received).ok_or_else(|| error!(PumpAmmError::Overflow))?;
        (hop.pool_base_token_account.amount, quote)
    } else {
        let base = hop.pool_base_token_account.amount;



        let base = base.checked_sub(amount).ok_or_else(|| error!(PumpAmmError::Overflow))?;
        (base, hop.pool_quote_token_account.amount)
    };
    let (real_quote_reserves, quote_reserve) =
        trade_v2::quote_reserves(&hop.pool, raw_quote_reserve)?;
    let old_virtual = hop.pool.virtual_quote_reserves;
    let base_supply = hop.base_mint.supply;
    let fee_config_len = a.fee_config.to_account_info().data_len();
    let context = HopFeeContext {
        pool: &hop.pool,
        global_config: &a.global_config,
        accounts: a,
        fee_config_len: &fee_config_len,
        base_supply: &base_supply,
        base_reserve: &base_reserve,
        quote_reserve: &quote_reserve,
        gates: &gates,
    };
    if buy {
        let fees = apply_hop_fees(&context, received)?;
        let (_, fees) = trade_v2::estimate_buy_quote(&hop.pool, &fees, received)?;


























        let curve_quote = received
            .checked_sub(fees.lp_fee)
            .and_then(|v| v.checked_sub(fees.protocol_fee))
            .and_then(|v| v.checked_sub(fees.creator_fee)).ok_or_else(|| error!(PumpAmmError::Overflow))?;
        let base_amount =
            crate::curve::constant_product::base_for_quote(curve_quote, base_reserve, quote_reserve)?;



        require_gt!(base_amount, 0, PumpAmmError::ZeroBaseAmount);
        require_gte!(base_reserve, base_amount, PumpAmmError::TooLittlePoolTokenLiquidity);
        if minimum.is_some() {
            crate::utils::token::transfer_from_pool(
                &base_program,
                &hop.base_mint,
                &hop.pool_base_token_account,
                &a.user_out_token_account,
                base_amount,
                &hop.pool,
            )?;
        }
        let buyback_fee = accrue_hop_fees(
            &a.global_config,
            a.buyback_fee_recipient.as_ref(),
            hop,
            &quote_program,
            fees.protocol_fee,
            fees.creator_fee,
            gates,
        )?;


        let result = BuyResult {
            fees: fees.clone(),
            base_amount_out: base_amount,
            quote_amount_in: curve_quote,
            user_quote_amount_in: received,
            buyback_fee, pool_quote_amount_in: amount,
            quote_transfer_fee: amount.checked_sub(received).ok_or_else(|| error!(PumpAmmError::Overflow))?,
        };








        let event = trade_v2::create_buy_event(
            TradeEventContext {
                pool: &hop.pool,
                global_config: &a.global_config,
                base_supply,
                user: a.user.key(),
                user_base_token_account: user_output_account,
                user_quote_token_account: user_input_account,
                user_volume_accumulator: &a.user_volume_accumulator,
            },
            &result,
            amount,
            minimum.unwrap_or(0),
            amount,
            curve_quote,
            user_output_balance,
            user_input_balance,
            base_reserve,
            raw_quote_reserve,
            old_virtual,
            transfer_fee::amount_after_fee(&quote_fee, buyback_fee)?,
            "multi_hop_swap",
        )?;
        emit!(event);
        emit_cpi!(event);
        Ok((base_amount, true))
    } else {
        let quote = crate::curve::constant_product::quote_for_sell(amount, base_reserve, quote_reserve)?;
        let fees = apply_hop_fees(&context, quote)?;
        let (quote_without_lp_fee, total_quote) =
            trade_v2::calculate_sell_quote(quote, &fees, real_quote_reserves)?;
        require_gt!(total_quote, 0, PumpAmmError::ZeroQuoteAmount);
        let quote_received = transfer_fee::amount_after_fee(&quote_fee, total_quote)?;




        let quote_transfer_fee = total_quote.checked_sub(quote_received).ok_or_else(|| error!(PumpAmmError::Overflow))?;
        if minimum.is_some() { crate::utils::token::transfer_from_pool(&quote_program, &hop.quote_mint, &hop.pool_quote_token_account, &a.user_out_token_account, total_quote, &hop.pool)?; }
        let buyback_fee = accrue_hop_fees(&a.global_config, a.buyback_fee_recipient.as_ref(), hop, &quote_program, fees.protocol_fee, fees.creator_fee, gates)?;

        let event = trade_v2::create_sell_event(
            TradeEventContext { pool: &hop.pool, global_config: &a.global_config, base_supply, user: a.user.key(),
                user_volume_accumulator: &a.user_volume_accumulator, user_base_token_account: user_input_account, user_quote_token_account: user_output_account },
            amount, minimum.unwrap_or(0), user_input_balance, user_output_balance, base_reserve, raw_quote_reserve,
            quote, &fees, quote_without_lp_fee, total_quote, old_virtual, transfer_fee::amount_after_fee(&quote_fee, buyback_fee)?, quote_transfer_fee,
        )?;
        emit!(event);
        emit_cpi!(event);
        Ok((quote_received, false))
    }
}

































/// Pool vault holding `mint` in the next AMM hop's accounts.
#[inline(never)]
fn vault_for_mint<'info>(
    accounts: &[AccountInfo<'info>],
    mint: &Pubkey,
) -> Result<AccountInfo<'info>> {
    let vault = if accounts[1].key() == *mint {
        &accounts[4]
    } else if accounts[0].key() == *mint {
        &accounts[3]
    } else {
        return err!(PumpAmmError::MultiHopDiscontinuousPath);
    };
    Ok(vault.clone())
}

/// Apply this AMM hop's fee accounting before its optional endpoint payout.
pub fn accrue_hop_fees<'info>(
    global_config: &Account<'info, GlobalConfig>,
    recipient: &AccountInfo<'info>,
    hop: &mut AmmHop<'info>,
    token_program: &AccountInfo<'info>,
    protocol_fee: u64,
    creator_fee: u64,
    gates: HopFeeGates,
) -> Result<u64> {
    let buyback_fee = hop.pool.accrue_fees(global_config, protocol_fee, creator_fee)?;
    if gates.protocol {
        crate::utils::token::transfer_from_pool_to_raw(
            token_program, &hop.quote_mint, &hop.pool_quote_token_account,
            &hop.pool, recipient, buyback_fee,
        )?;
    }
    Ok(buyback_fee)
}
#[inline(never)]
fn curve_run<'info>(
    ctx: &Context<'_, '_, 'info, 'info, MultiHop<'info>>,
    hops: &[AccountInfo<'info>],
    flags: Vec<u8>,
    amount: u64,
    source: &HopSource<'info>,
    destination: AccountInfo<'info>,
    buy: bool,
) -> Result<u64> {
    let a = &ctx.accounts;
    let bump = [ctx.bumps.global_config];
    let config_seeds: &[&[u8]] = &[b"global_config", &bump];
    let is_native = |chunk: &[AccountInfo]| chunk[1].key() == anchor_spl::token::spl_token::native_mint::ID;
    let first_input_mint_is_wsol = buy && is_native(&hops[..5]);
    let output_native = !buy && is_native(&hops[hops.len().saturating_sub(5)..]);



    let (source, authority, pool) = match source {
        HopSource::User => (
            a.user_in_token_account.to_account_info(),
            a.user.to_account_info(),
            None,
        ),
        HopSource::Pool(hop, base) => (
            if *base {
                &hop.pool_base_token_account
            } else {
                &hop.pool_quote_token_account
            }
            .to_account_info(),
            hop.pool.to_account_info(),
            Some(&hop.pool),
        ),


        HopSource::Curve => return err!(PumpAmmError::AccountTypeNotSupported),
    };
    let same_account = source.key() == destination.key();
    let source_before = token_amount(&source)?;
    let destination_before = if same_account {
        source_before
    } else {
        token_amount(&destination)?
    };
    let cpi_accounts = crate::pump::cpi::accounts::MultiHopCurveSwap {
        global: a.pump_global.to_account_info(),
        fee_config: a.pump_fee_config.to_account_info(),
        pump_amm_global_config: a.global_config.to_account_info(),
        user: a.user.to_account_info(),
        source_authority: authority,
        source: source.clone(),
        destination: destination.clone(),
        buyback_fee_recipient: a.buyback_fee_recipient.to_account_info(),
        token_program: a.token_program.to_account_info(),
        token_2022_program: a.token_2022_program.to_account_info(),
        system_program: a.system_program.to_account_info(),
        event_authority: a.pump_event_authority.to_account_info(),
        program: a.pump_program.to_account_info(),
    };
    let program = a.pump_program.to_account_info();
    let remaining_accounts = hops.to_vec();
    let result = match pool {
        None => crate::pump::cpi::multi_hop_curve_swap(
            CpiContext::new_with_signer(program, cpi_accounts, &[config_seeds])
                .with_remaining_accounts(remaining_accounts),
            amount,
            flags,
        )?
        .get(),
        Some(pool) => {
            let index = pool.index.to_le_bytes();
            let bump = [pool.pool_bump];
            let pool_seeds: &[&[u8]] = &[
                b"pool",
                &index,
                pool.creator.as_ref(),
                pool.base_mint.as_ref(),
                pool.quote_mint.as_ref(),
                &bump,
            ];
            crate::pump::cpi::multi_hop_curve_swap(
                CpiContext::new_with_signer(program, cpi_accounts, &[config_seeds, pool_seeds])
                    .with_remaining_accounts(remaining_accounts),
                amount,
                flags,
            )?
            .get()
        }
    };
    let amount_out = result.amount_out;
    let buyback = result.buyback_fee;
    let buyback_fee_recipient = a.buyback_fee_recipient.key();
    let source_buyback = if buyback_fee_recipient == source.key() { buyback } else { 0 };
    let destination_buyback = if buyback_fee_recipient == destination.key() { buyback } else { 0 };
    if !first_input_mint_is_wsol {
        let expected = if same_account {
            source_before.checked_sub(amount).and_then(|v| v.checked_add(amount_out))
        } else {
            source_before.checked_sub(amount)
        }
        .and_then(|v| v.checked_add(source_buyback))
























        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
        require_eq!(token_amount(&source)?, expected, PumpAmmError::MultiHopCurveRunMismatch);
    }




    if !same_account && !output_native {
        require_eq!(
            token_amount(&destination)?,
            destination_before
                .checked_add(amount_out)
                .and_then(|v| v.checked_add(destination_buyback))
                .ok_or_else(|| error!(PumpAmmError::Overflow))?,
            PumpAmmError::MultiHopCurveRunMismatch
        );
    }
    Ok(amount_out)
}
