use crate::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token_interface::TokenInterface,
};
#[event_cpi]
#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub global_config: Box<Account<'info, GlobalConfig>>,
    #[account(mut,has_one=quote_mint @ PumpAmmError::InvalidQuoteMint,has_one=pool_quote_token_account @ PumpAmmError::InvalidPoolQuoteTokenAccount)]
    pub pool: Box<Account<'info, Pool>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub pool_quote_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: validated against configured recipients or creator vault PDA in handler.
    pub recipient: UncheckedAccount<'info>,
    /// CHECK: associated-token address constrained; token CPI validates initialized data.
    #[account(mut,seeds=[recipient.key().as_ref(),quote_token_program.key().as_ref(),quote_mint.key().as_ref()],bump,seeds::program=associated_token::ID)]
    pub recipient_token_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
/// Emitted when the accrued fees of one bucket of a pool are swept.
#[event]
#[derive(Clone)]
pub struct SweepPoolFeeEvent {
    /// Unix timestamp of the sweep.
    pub timestamp: i64,
    /// Pool the fees were accrued in.
    pub pool: Pubkey,
    /// Base mint of the pool.
    pub base_mint: Pubkey,
    /// Quote mint of the pool; the fees are always paid in quote.
    pub quote_mint: Pubkey,
    /// Owner of `recipient_token_account`.
    pub recipient: Pubkey,
    /// Signer that paid for the sweep (and the recipient ATA, if created).
    pub payer: Pubkey,
    /// Amount of quote tokens transferred to the recipient.
    pub amount: u64,
    /// `PROTOCOL_FEE_BUCKET` or `CREATOR_FEE_BUCKET`.
    pub bucket: u8,
}

/// Bucket of the fees owed to the protocol fee recipients.
pub const PROTOCOL_FEE_BUCKET: u8 = 0;
/// Bucket of the fees owed to the coin creator (paid to its creator vault).
pub const CREATOR_FEE_BUCKET: u8 = 1;

/// Sweeps the quote fees accrued by `pool` for one fee bucket.
///
/// * `creator == false`: protocol fees, `recipient` must be one of the
///   protocol fee recipients configured in `global_config` (or a mayhem
///   recipient for mayhem pools).
/// * `creator == true`: coin creator fees, `recipient` must be the creator
///   vault PDA of `pool.coin_creator`.
///
/// The recipient ATA is created when it does not exist yet. Nothing is
/// transferred (and no event is emitted) when the bucket is empty.
///
/// Steps:
/// 1. resize `pool` to the current `Pool` size if needed (paid by `payer`),
/// 2. validate `recipient` for the requested bucket,
/// 3. check that the pool quote vault still covers the virtual quote
///    reserves plus every accrued fee, so sweeping can never eat into the
///    liquidity of the pool,
/// 4. reset the accrued fees of the bucket and read the amount to sweep,
/// 5. create the recipient ATA if empty and transfer the fees, signed by
///    the pool PDA,
/// 6. emit `SweepPoolFeeEvent` (log and self-CPI).
///
///
/// Errors:
/// * `InvalidProtocolFeeRecipient` when `recipient` is the default pubkey or
///   not a configured protocol fee recipient,
/// * `ConstraintSeeds` when `recipient` is not the creator vault PDA,
/// * any error of the pool accounting helpers.
pub fn execute(ctx: Context<SweepFees>, creator: bool) -> Result<()> {
    let a = &mut *ctx.accounts;
    // Pools created before the account was extended are resized first.
    crate::instructions::swap::prepare_pool(
        &a.pool,
        a.system_program.as_ref(),
        a.payer.as_ref(),
    )?;

    let recipient = a.recipient.key();
    require_keys_neq!(
        recipient,
        Pubkey::default(),
        PumpAmmError::InvalidProtocolFeeRecipient
    );
    if creator {
        // Creator fees can only be swept into the creator vault PDA of the
        // pool's coin creator; the vault is owned by this program and
        // distributes the fees later on.
        //
        // The vault is derived from `pool.coin_creator`, which can be
        // changed by the admin / the creator itself (see `set_coin_creator`
        // and `admin_cto_pool`); fees accrued before a change are swept to
        // the vault of the current creator.
        //
        // The bump is not needed here, only the address is compared, and
        // the vault does not have to exist yet.
        let expected = Pubkey::find_program_address(
            &[b"creator_vault", a.pool.coin_creator.as_ref()],
            &crate::ID,
        )
        .0;
        require_keys_eq!(
            recipient,
            expected,
            anchor_lang::error::ErrorCode::ConstraintSeeds
        );
    } else {
        crate::instructions::swap::validate_protocol_fee_recipient(
            &a.global_config,
            &a.recipient,
            a.pool.is_mayhem_mode as u8,
        )?;
    }
    a.pool.available_quote_reserves(a.pool_quote_token_account.amount)?;
    let amount = a.pool.take_accrued_fees(creator)?;
    if amount == 0 {
        msg!("Nothing to sweep for fee bucket {}", u8::from(creator));
        return Ok(());
    }
    crate::utils::token::create_ata_if_empty(
        a.quote_token_program.as_ref(),
        &a.quote_mint,
        a.recipient.as_ref(),
        a.recipient_token_account.as_ref(),
        a.payer.as_ref(),
        &a.system_program,
        &a.associated_token_program,
    )?;
    crate::utils::token::transfer_from_pool_to_raw(
        a.quote_token_program.as_ref(),
        &a.quote_mint,
        &a.pool_quote_token_account,
        &a.pool,
        a.recipient_token_account.as_ref(),
        amount,
    )?;
    let event = SweepPoolFeeEvent {
        timestamp: Clock::get()?.unix_timestamp,
        pool: a.pool.key(),
        base_mint: a.pool.base_mint,
        quote_mint: a.pool.quote_mint,
        recipient,
        payer: a.payer.key(),
        amount,
        bucket: if creator { CREATOR_FEE_BUCKET } else { PROTOCOL_FEE_BUCKET },
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
