use anchor_lang::prelude::*;
use crate::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

























fn distribute_cashback<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    payer: &Signer<'info>,
    user_volume: &mut Account<'info, UserVolumeAccumulator>,
    user_quote: &InterfaceAccount<'info, TokenAccount>,
    quote_decimals: u8,
    quote_mint: &InterfaceAccount<'info, Mint>,
    quote_token_program: &Interface<'info, TokenInterface>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, AssociatedToken>,
    amount: u64,
    base_mint: &Pubkey,
) -> Result<()> {
    let ata = match remaining_accounts.first() {
        Some(ata) => ata,
        None => return err!(PumpAmmError::MissingCashbackAccounts),
    };
    let expected = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &user_volume.key(),
        &quote_mint.key(),
        &quote_token_program.key(),
    );



    require!(ata.key() == expected, PumpAmmError::InvalidCashbackAccumulatorAta);


    if ata.is_writable == false { return err!(PumpAmmError::InvalidCashbackAccumulatorAta); }

    if ata.data_is_empty() {
        anchor_spl::associated_token::create(CpiContext::new(
            associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: payer.to_account_info(),
                associated_token: ata.clone(),
                authority: user_volume.to_account_info(),
                mint: quote_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: quote_token_program.to_account_info(),
            },
        ))
    } else { Ok(()) }?;

    crate::utils::token::validate_self_transfer(
        &user_quote.key(),
        &user_quote.owner,
        ata.key,
        payer.key,
    )?;

    token_interface::transfer_checked(
        CpiContext::new(
            quote_token_program.to_account_info(),
            token_interface::TransferChecked {
                from: user_quote.to_account_info(),
                mint: quote_mint.to_account_info(),
                to: ata.clone(),
                authority: payer.to_account_info(),
            },
        ),
        amount,
        quote_decimals,
    )?;


    user_volume.cashback_earned = user_volume
        .cashback_earned
        .checked_add(amount)
        .ok_or(error!(PumpAmmError::Overflow))?;
    user_volume.exit(&crate::ID)?;
    crate::instructions::swap::sell::validate_pool_v2(remaining_accounts, &base_mint.key(), 1)?; Ok(())
}






pub(crate) fn get_buyback_accounts<'a, 'info>(
    remaining_accounts: &'a [AccountInfo<'info>],
    global_config: &GlobalConfig,
    index: usize,
    quote_mint: &Pubkey,
    quote_token_program: &Pubkey,
) -> Result<(&'a AccountInfo<'info>, &'a AccountInfo<'info>)> {
    let owner = remaining_accounts.get(index)
        .ok_or_else(|| error!(PumpAmmError::BuybackFeeRecipientMissing))?;



    let dest = remaining_accounts.get(index + 1)
        .ok_or_else(|| error!(PumpAmmError::BuybackFeeRecipientMissing))?;
    global_config.validate_buyback_recipient(&owner.key(), &dest.key(), quote_mint, quote_token_program)?;
    Ok((owner, dest))
}








#[derive(Accounts)]
pub struct LegacyBuy<'info> {
    #[account(mut,has_one=base_mint @ PumpAmmError::InvalidBaseMint,has_one=quote_mint @ PumpAmmError::InvalidQuoteMint,has_one=pool_base_token_account @ PumpAmmError::InvalidPoolBaseTokenAccount,has_one=pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub global_config: Box<Account<'info, GlobalConfig>>,
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub user_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub pool_base_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Authorization checked in handler.
    pub protocol_fee_recipient: AccountInfo<'info>,
    #[account(init_if_needed,payer=user,associated_token::mint=quote_mint,associated_token::authority=protocol_fee_recipient,associated_token::token_program=quote_token_program)]
    pub protocol_fee_recipient_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Self event signer.
    #[account(seeds=[b"__event_authority"],bump)]
    pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, trade::PumpAmmProgram>,
    #[account(init_if_needed,payer=user,associated_token::mint=quote_mint,associated_token::authority=coin_creator_vault_authority,associated_token::token_program=quote_token_program)]
    pub coin_creator_vault_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Original PDA constraint.
    #[account(seeds=[b"creator_vault",pool.coin_creator.as_ref()],bump)]
    pub coin_creator_vault_authority: AccountInfo<'info>,
    #[account(seeds=[b"global_volume_accumulator"],bump)]
    pub global_volume_accumulator: Box<Account<'info, GlobalVolumeAccumulator>>,
    #[account(init_if_needed,payer=user,space=137,seeds=[b"user_volume_accumulator",user.key().as_ref()],bump)]
    pub user_volume_accumulator: Box<Account<'info, UserVolumeAccumulator>>,
    #[account(seeds=[b"fee_config",crate::ID.as_ref()],bump,seeds::program=fee_program.key())]
    pub fee_config: Account<'info, trade::TradeFeeConfig>,
    pub fee_program: Program<'info, LegacyBuyFeesProgram>,
}
pub use crate::legacy_sell::FeesProgram as LegacyBuyFeesProgram;
fn number(n: u128) -> Result<u64> {
    u64::try_from(n).map_err(|_| error!(PumpAmmError::Truncation))
}
use crate::curve::fees::fee;
fn transfer<'info>(
    program: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amount: u64,
    seeds: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    if from.key == to.key {
        let source = TokenAccount::try_deserialize(&mut &from.try_borrow_data()?[..])?;
        require!(source.owner == authority.key(), PumpAmmError::SelfTransferNotAllowed);
    }
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            program.clone(),
            token_interface::TransferChecked {
                from: from.clone(),
                to: to.clone(),
                mint: mint.to_account_info(),
                authority: authority.clone(),
            },
            seeds,
        ),
        amount,
        mint.decimals,
    )
}






















#[event]
#[derive(Clone)]
pub struct BuyEvent {
    pub timestamp: i64,
    pub base_amount_out: u64,
    pub max_quote_amount_in: u64,
    pub user_base_token_reserves: u64,
    pub user_quote_token_reserves: u64,
    pub pool_base_token_reserves: u64,
    pub pool_quote_token_reserves: u64,
    pub quote_amount_in: u64,
    pub lp_fee_basis_points: u64,
    pub lp_fee: u64,
    pub protocol_fee_basis_points: u64,
    pub protocol_fee: u64,
    pub quote_amount_in_with_lp_fee: u64,
    pub user_quote_amount_in: u64,
    pub pool: Pubkey,
    pub user: Pubkey,
    pub user_base_token_account: Pubkey,
    pub user_quote_token_account: Pubkey,
    pub protocol_fee_recipient: Pubkey,
    pub protocol_fee_recipient_token_account: Pubkey,
    pub coin_creator: Pubkey,
    pub coin_creator_fee_basis_points: u64,
    pub coin_creator_fee: u64,
    pub track_volume: bool,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
    pub min_base_amount_out: u64,
    pub ix_name: String,
    pub cashback_fee_basis_points: u64,
    pub cashback: u64,
    pub buyback_fee_basis_points: u64,
    pub buyback_fee: u64,
    pub virtual_quote_reserves: i128,
    pub can_boost: bool,
    pub base_supply: u64,
    pub holder_rewards_bps: u64,
    pub holder_rewards: u64,
    pub creator_fee_unclaimed: u64,
    pub quote_transfer_fee: u64,
}
#[inline(never)]
pub fn buy<'info>(
    ctx: Context<'_, '_, '_, 'info, LegacyBuy<'info>>,
    base_amount_out: u64,
    max_quote_amount_in: u64,
    track_volume: bool,
) -> Result<()> {
    let a = &mut *ctx.accounts;































    require!(
        a.global_config.disable_flags & 8 == 0,
        PumpAmmError::DisabledBuy
    );
    crate::instructions::swap::prepare_pool(&a.pool, a.system_program.as_ref(), a.user.as_ref())?;
    crate::instructions::swap::validate_protocol_fee_recipient(
        &a.global_config, &a.protocol_fee_recipient, a.pool.is_mayhem_mode as u8,
    )?;

    require_gt!(base_amount_out, 0, PumpAmmError::ZeroBaseAmount);
    require_gt!(max_quote_amount_in, 0, PumpAmmError::ZeroQuoteAmount);
    let user_base_token_reserves = a.user_base_token_account.amount;
    let user_quote_token_reserves = a.user_quote_token_account.amount;
    let pool_base_token_reserves = a.pool_base_token_account.amount;
    let pool_quote_token_reserves = a.pool_quote_token_account.amount;
    a.pool.available_quote_reserves(pool_quote_token_reserves)?;
    let q = a.pool.effective_quote_reserves(pool_quote_token_reserves)?;
    let b = pool_base_token_reserves;
    let curve_initial = crate::curve::constant_product::quote_for_base(base_amount_out, b, q)?;
    let selected = crate::curve::fees::select_and_apply(
        &a.pool, &a.global_config, &a.fee_config, &a.fee_program, &a.program,
        &a.base_mint, b, q, curve_initial,
    )?;
    let mut selected = selected;
    if a.global_config.whitelist_pda == a.user.key() {
        selected.lp_fee_bps = 0;
        selected.protocol_fee_bps = 0;
        selected.creator_fee_bps = 0;
        selected.lp_fee = 0;
        selected.protocol_fee = 0;
        selected.creator_fee = 0;
    }
    let curve = curve_initial;
    let lp_fee = selected.lp_fee;
    let protocol_fee = selected.protocol_fee;
    let creator_fee = selected.creator_fee;
































    let pool_quote = curve
        .checked_add(lp_fee)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;

    let total = pool_quote
        .checked_add(creator_fee)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;

    let total = total
        .checked_add(protocol_fee)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    let quote_fee_data = crate::utils::transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let gross_pool_quote = crate::utils::transfer_fee::amount_before_fee(&quote_fee_data, pool_quote)?;




    let quote_transfer_fee = gross_pool_quote
        .checked_sub(pool_quote)
        .ok_or_else(|| error!(PumpAmmError::Overflow))?;
    require_gte!(
        max_quote_amount_in,
        total
            .checked_add(quote_transfer_fee)
            .ok_or_else(|| error!(PumpAmmError::Overflow))?,
        PumpAmmError::ExceededSlippage
    );
    crate::utils::token::transfer_from_pool(
        a.base_token_program.as_ref(),
        &a.base_mint,
        &a.pool_base_token_account,
        &a.user_base_token_account,
        base_amount_out,
        &a.pool,
    )?;
    crate::utils::token::transfer_checked(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
        &a.pool_quote_token_account, a.user.as_ref(), gross_pool_quote, &[],
    )?;
    let (buyback_fee, net_protocol) = a.global_config.split_buyback(
        protocol_fee,
        a.pool.is_mayhem_mode,
    );
    crate::utils::token::transfer_checked(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
        &a.protocol_fee_recipient_token_account, a.user.as_ref(), net_protocol, &[],
    )?;
    let mut is_cashback = false;
    let mut next = 0;
    if creator_fee > 0 {
        if a.pool.is_cashback_coin {
            pay_cashback(
                ctx.remaining_accounts,
                &mut a.user_volume_accumulator,
                &a.user_quote_token_account,
                &a.quote_mint,
                &a.base_mint.key(),
                &a.user,
                &a.quote_token_program,
                &a.system_program,
                &a.associated_token_program,
                creator_fee,
            )?;
            next = 2;
            is_cashback = true;
        } else {
            crate::instructions::swap::sell::validate_pool_v2(ctx.remaining_accounts, &a.base_mint.key(), 0)?;
            next = 1;
            crate::utils::token::transfer_checked(
                a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
                &a.coin_creator_vault_ata, a.user.as_ref(), creator_fee, &[],
            )?;
        }
    }
    let (owner, dest) = get_buyback_accounts(
        ctx.remaining_accounts, &a.global_config, next, &a.quote_mint.key(), &a.quote_token_program.key(),
    )?;
    if buyback_fee > 0 {
        crate::utils::token::transfer_checked_with_ata(
            a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
            owner, dest, a.user.as_ref(), &a.system_program, &a.associated_token_program,
            buyback_fee, &[],
        )?;
    }
    a.user_volume_accumulator.user = a.user.key();
    let total_unclaimed_tokens = a.user_volume_accumulator.total_unclaimed_tokens;
    let total_claimed_tokens = a.user_volume_accumulator.total_claimed_tokens;
    let current_sol_volume = a.user_volume_accumulator.current_sol_volume;
    let last_update_timestamp = a.user_volume_accumulator.last_update_timestamp;
    let event = BuyEvent {
        timestamp: Clock::get()?.unix_timestamp,
        base_amount_out,
        max_quote_amount_in,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves,
        pool_quote_token_reserves,
        quote_amount_in: curve,
        lp_fee_basis_points: selected.lp_fee_bps,
        lp_fee,
        protocol_fee_basis_points: selected.protocol_fee_bps,
        protocol_fee,
        quote_amount_in_with_lp_fee: pool_quote,
        user_quote_amount_in: total,
        pool: a.pool.key(),
        user: a.user.key(),
        user_base_token_account: a.user_base_token_account.key(),
        user_quote_token_account: a.user_quote_token_account.key(),
        protocol_fee_recipient: a.protocol_fee_recipient.key(),
        protocol_fee_recipient_token_account: a.protocol_fee_recipient_token_account.key(),
        coin_creator: a.pool.coin_creator,
        coin_creator_fee_basis_points: if is_cashback {
            0
        } else {
            selected.creator_fee_bps
        },
        coin_creator_fee: if is_cashback {
            0
        } else {
            creator_fee
        },
        holder_rewards_bps: if a.pool.is_holder_reward {
            selected.creator_fee_bps
        } else {
            0
        },
        holder_rewards: if a.pool.is_holder_reward {
            creator_fee
        } else {
            0
        },
        track_volume,
        total_unclaimed_tokens,
        total_claimed_tokens,
        current_sol_volume,
        last_update_timestamp,
        min_base_amount_out: base_amount_out,
        ix_name: "buy".to_string(),
        cashback_fee_basis_points: if is_cashback {
            selected.creator_fee_bps
        } else {
            0
        },
        cashback: if is_cashback {
            creator_fee
        } else {
            0
        },
        buyback_fee_basis_points: if a.pool.is_mayhem_mode { 0 } else { a.global_config.buyback_basis_points },
        buyback_fee: crate::utils::transfer_fee::amount_after_fee(&quote_fee_data, buyback_fee)?,
        virtual_quote_reserves: a.pool.virtual_quote_reserves,
        can_boost: a.pool.can_boost(),
        base_supply: if a.pool.is_mayhem_mode { 1_000_000_000_000_000 } else { a.base_mint.supply },
        creator_fee_unclaimed: a.pool.creator_fees,
        quote_transfer_fee,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

fn pay_cashback<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    user_volume_accumulator: &mut Account<'info, UserVolumeAccumulator>,
    user_quote_token_account: &InterfaceAccount<'info, TokenAccount>,
    quote_mint: &InterfaceAccount<'info, Mint>,
    base_mint: &Pubkey,
    user: &Signer<'info>,
    quote_token_program: &Interface<'info, TokenInterface>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, AssociatedToken>,
    amount: u64,
) -> Result<()> {
    distribute_cashback(
        remaining_accounts,
        user,
        user_volume_accumulator,
        user_quote_token_account,
        quote_mint.decimals,
        quote_mint,
        quote_token_program,
        system_program,
        associated_token_program,
        amount,
        base_mint,
    )
}













































#[inline(never)]
pub fn buy_exact_quote_in<'info>(
    ctx: Context<'_, '_, '_, 'info, LegacyBuy<'info>>,
    spendable_quote_in: u64,
    min_base_amount_out: u64,
    track_volume: bool,
) -> Result<()> {
    let a = &mut *ctx.accounts;
    require!(
        a.global_config.disable_flags & 8 == 0,
        PumpAmmError::DisabledBuy
    );
    crate::instructions::swap::prepare_pool(&a.pool, a.system_program.as_ref(), a.user.as_ref())?;
    crate::instructions::swap::validate_protocol_fee_recipient(
        &a.global_config, &a.protocol_fee_recipient, a.pool.is_mayhem_mode as u8,
    )?;



    require_gt!(min_base_amount_out, 0, PumpAmmError::ZeroBaseAmount);

    require_gt!(spendable_quote_in, 0, PumpAmmError::ZeroQuoteAmount);
    let quote_fee_data = crate::utils::transfer_fee::from_mint(&a.quote_mint.to_account_info())?;
    let available = crate::utils::transfer_fee::amount_after_fee(&quote_fee_data, spendable_quote_in)?;





    require_gt!(available, 0, PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees);
    let user_base_token_reserves = a.user_base_token_account.amount;
    let user_quote_token_reserves = a.user_quote_token_account.amount;
    let pool_base_token_reserves = a.pool_base_token_account.amount;
    let pool_quote_token_reserves = a.pool_quote_token_account.amount;
    a.pool.available_quote_reserves(pool_quote_token_reserves)?;
    let q = a.pool.effective_quote_reserves(pool_quote_token_reserves)?;
    let b = pool_base_token_reserves;
    let mut selected = crate::curve::fees::select_and_apply(
        &a.pool, &a.global_config, &a.fee_config, &a.fee_program, &a.program,
        &a.base_mint, b, q, available,
    )?;
    if a.global_config.whitelist_pda == a.user.key() {
        selected.lp_fee_bps = 0;
        selected.protocol_fee_bps = 0;
        selected.creator_fee_bps = 0;
    }
    let creator_bps = if a.pool.coin_creator == Pubkey::default() { 0 } else { selected.creator_fee_bps };

    let total_fee_bps = selected.protocol_fee_bps
        .saturating_add(selected.lp_fee_bps)
        .saturating_add(creator_bps);
    let estimate = if total_fee_bps == 0 {
        available
    } else {
        let sum = (total_fee_bps as u128) + 10000;
        number((available as u128) * 10000 / sum)?
    };


























    require_gt!(estimate, 0, PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees);
    let lp_fee = fee(estimate, selected.lp_fee_bps)?;
    let protocol_fee = fee(estimate, selected.protocol_fee_bps)?;
    let creator_fee = if a.pool.coin_creator != Pubkey::default() {
        fee(estimate, selected.creator_fee_bps)?
    } else {
        0
    };
    let total_fees = lp_fee
        .checked_add(protocol_fee)
        .and_then(|n| n.checked_add(creator_fee))
        .ok_or(PumpAmmError::Overflow)?;
    let total = estimate.checked_add(total_fees).ok_or(PumpAmmError::Overflow)?;
    let mut curve = estimate;
    if total > available {
        curve = curve
            .checked_sub(total.saturating_sub(available))
            .ok_or(PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees)?;















        require_gt!(curve, 0, PumpAmmError::BuySlippageBelowMinBaseAmountOut);
    }
    let gross_pool_quote = |quote: u64| {
        crate::utils::transfer_fee::amount_before_fee(
            &quote_fee_data,
            quote.checked_add(lp_fee).ok_or(PumpAmmError::Overflow)?,
        )
    };
    let gross_total = gross_pool_quote(curve)?
        .checked_add(protocol_fee)
        .and_then(|n| n.checked_add(creator_fee))
        .ok_or(PumpAmmError::Overflow)?;
    if gross_total > spendable_quote_in {
        curve = curve
            .checked_sub(gross_total.saturating_sub(spendable_quote_in))
            .ok_or(PumpAmmError::BuyNotEnoughQuoteTokensToCoverFees)?;





        require_gt!(curve, 0, PumpAmmError::BuySlippageBelowMinBaseAmountOut);
    }
    let pool_quote = curve.checked_add(lp_fee).ok_or(PumpAmmError::Overflow)?;
    let gross_pool_quote = gross_pool_quote(curve)?;
    let quote_transfer_fee = gross_pool_quote
        .checked_sub(pool_quote)
        .ok_or(PumpAmmError::Overflow)?;
    let curve = curve;
    let base_out = crate::curve::constant_product::base_for_quote(curve, b, q)?;













    require_gte!(base_out, min_base_amount_out, PumpAmmError::BuySlippageBelowMinBaseAmountOut);




    require_gte!(b, base_out, PumpAmmError::TooLittlePoolTokenLiquidity);
    crate::utils::token::transfer_from_pool(
        a.base_token_program.as_ref(),
        &a.base_mint,
        &a.pool_base_token_account,
        &a.user_base_token_account,
        base_out,
        &a.pool,
    )?;
    crate::utils::token::transfer_checked(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
        &a.pool_quote_token_account, a.user.as_ref(), gross_pool_quote, &[],
    )?;
    let (buyback_fee, net_protocol) = a.global_config.split_buyback(
        protocol_fee,
        a.pool.is_mayhem_mode,
    );
    crate::utils::token::transfer_checked(
        a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
        &a.protocol_fee_recipient_token_account, a.user.as_ref(), net_protocol, &[],
    )?;
    let mut is_cashback = false;
    let next;
    if creator_fee == 0 {
        next = 0;
    } else if a.pool.is_cashback_coin {
            pay_cashback_exact(
                ctx.remaining_accounts,
                &mut a.user_volume_accumulator,
                &a.user_quote_token_account,
                &a.quote_mint,
                &a.base_mint.key(),
                &a.user,
                &a.quote_token_program,
                &a.system_program,
                &a.associated_token_program,
                creator_fee,
            )?;
        next = 2;
        is_cashback = true;
    } else {
            crate::instructions::swap::sell::validate_pool_v2(ctx.remaining_accounts, &a.base_mint.key(), 0)?;
        next = 1;
            crate::utils::token::transfer_checked(
                a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
                &a.coin_creator_vault_ata, a.user.as_ref(), creator_fee, &[],
            )?;
    }
    let (owner, dest) = get_buyback_accounts(
        ctx.remaining_accounts, &a.global_config, next, &a.quote_mint.key(), &a.quote_token_program.key(),
    )?;
    if buyback_fee > 0 {
        crate::utils::token::transfer_checked_with_ata(
            a.quote_token_program.as_ref(), &a.quote_mint, &a.user_quote_token_account,
            owner, dest, a.user.as_ref(), &a.system_program, &a.associated_token_program,
            buyback_fee, &[],
        )?;
    }
    a.user_volume_accumulator.user = a.user.key();
    let total_unclaimed_tokens = a.user_volume_accumulator.total_unclaimed_tokens;
    let total_claimed_tokens = a.user_volume_accumulator.total_claimed_tokens;
    let current_sol_volume = a.user_volume_accumulator.current_sol_volume;
    let last_update_timestamp = a.user_volume_accumulator.last_update_timestamp;
    let event = BuyEvent {
        timestamp: Clock::get()?.unix_timestamp,
        base_amount_out: base_out,
        max_quote_amount_in: spendable_quote_in,
        user_base_token_reserves,
        user_quote_token_reserves,
        pool_base_token_reserves,
        pool_quote_token_reserves,
        quote_amount_in: spendable_quote_in,
        lp_fee_basis_points: selected.lp_fee_bps,
        lp_fee,
        protocol_fee_basis_points: selected.protocol_fee_bps,
        protocol_fee,
        quote_amount_in_with_lp_fee: pool_quote,
        user_quote_amount_in: curve,
        pool: a.pool.key(),
        user: a.user.key(),
        user_base_token_account: a.user_base_token_account.key(),
        user_quote_token_account: a.user_quote_token_account.key(),
        protocol_fee_recipient: a.protocol_fee_recipient.key(),
        protocol_fee_recipient_token_account: a.protocol_fee_recipient_token_account.key(),
        coin_creator: a.pool.coin_creator,
        coin_creator_fee_basis_points: if is_cashback {
            0
        } else {
            selected.creator_fee_bps
        },
        coin_creator_fee: if is_cashback {
            0
        } else {
            creator_fee
        },
        holder_rewards_bps: if a.pool.is_holder_reward {
            selected.creator_fee_bps
        } else {
            0
        },
        holder_rewards: if a.pool.is_holder_reward {
            creator_fee
        } else {
            0
        },
        track_volume,
        total_unclaimed_tokens,
        total_claimed_tokens,
        current_sol_volume,
        last_update_timestamp,
        min_base_amount_out,
        ix_name: "buy_exact_quote_in".to_string(),
        cashback_fee_basis_points: if is_cashback {
            selected.creator_fee_bps
        } else {
            0
        },
        cashback: if is_cashback {
            creator_fee
        } else {
            0
        },
        buyback_fee_basis_points: if a.pool.is_mayhem_mode { 0 } else { a.global_config.buyback_basis_points },
        buyback_fee: crate::utils::transfer_fee::amount_after_fee(&quote_fee_data, buyback_fee)?,
        virtual_quote_reserves: a.pool.virtual_quote_reserves,
        can_boost: a.pool.can_boost(),
        base_supply: if a.pool.is_mayhem_mode { 1_000_000_000_000_000 } else { a.base_mint.supply },
        creator_fee_unclaimed: a.pool.creator_fees,
        quote_transfer_fee,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

fn pay_cashback_exact<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    user_volume_accumulator: &mut Account<'info, UserVolumeAccumulator>,
    user_quote_token_account: &InterfaceAccount<'info, TokenAccount>,
    quote_mint: &InterfaceAccount<'info, Mint>,
    base_mint: &Pubkey,
    user: &Signer<'info>,
    quote_token_program: &Interface<'info, TokenInterface>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, AssociatedToken>,
    amount: u64,
) -> Result<()> {
    distribute_cashback(
        remaining_accounts,
        user,
        user_volume_accumulator,
        user_quote_token_account,
        quote_mint.decimals,
        quote_mint,
        quote_token_program,
        system_program,
        associated_token_program,
        amount,
        base_mint,
    )
}
fn transfer_quote<'info>(
    a: &LegacyBuy<'info>,
    dest: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
) -> Result<()> {
    let extra = crate::quote_transfer_fee(&a.quote_mint.to_account_info(), amount, true)?;
    let gross = amount.checked_add(extra).ok_or(PumpAmmError::Overflow)?;
    crate::utils::token::transfer_checked(
        a.quote_token_program.as_ref(),
        &a.quote_mint,
        &a.user_quote_token_account,
        dest,
        a.user.as_ref(),
        gross,
        &[],
    )
}
fn create_ata<'info>(
    a: &LegacyBuy<'info>,
    ata: &AccountInfo<'info>,
    owner: &AccountInfo<'info>,
    idempotent: bool,
) -> Result<()> {
    let create = if idempotent {
        anchor_spl::associated_token::create_idempotent
    } else {
        anchor_spl::associated_token::create
    };
    create(CpiContext::new(
        a.associated_token_program.to_account_info(),
        anchor_spl::associated_token::Create {
            payer: a.user.to_account_info(),
            associated_token: ata.clone(),
            authority: owner.clone(),
            mint: a.quote_mint.to_account_info(),
            system_program: a.system_program.to_account_info(),
            token_program: a.quote_token_program.to_account_info(),
        },
    ))
}

fn transfer_fee<'info>(a: &LegacyBuy<'info>, dest: &AccountInfo<'info>, amount: u64) -> Result<()> {
    transfer(
        &a.quote_token_program.to_account_info(),
        &a.quote_mint,
        &a.user_quote_token_account.to_account_info(),
        dest,
        &a.user.to_account_info(),
        amount,
        &[],
    )
}
