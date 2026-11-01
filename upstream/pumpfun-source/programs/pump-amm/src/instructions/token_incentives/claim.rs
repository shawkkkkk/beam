use crate::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};

/// Claims the incentive tokens earned by `user` from the global incentive
/// token account. The ATA of the user is created when needed and paid by
/// `payer`, so anybody can crank a claim on behalf of a user.
///
/// Accounts:
/// - `global_volume_accumulator` holds the daily token supply and the daily
///   SOL volume of the incentive range,
/// - `user_volume_accumulator` holds the SOL volume of the user for the last
///   day it traded and its unclaimed / claimed token totals,
/// - `global_incentive_token_account` is the vault the tokens are paid from.
#[derive(Accounts)]
pub struct ClaimTokenIncentives<'info> {
    /// CHECK: Destination token-account authority and PDA seed.
    pub user: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    /// Token account of `user` receiving the incentive tokens.
    /// Created on the first claim, rent paid by `payer`.
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"global_volume_accumulator"],
        bump,
        has_one = mint @ PumpAmmError::InvalidIncentiveMint
    )]
    /// Global incentive schedule; signs the transfer out of
    /// `global_incentive_token_account` with its PDA seeds.
    pub global_volume_accumulator: Account<'info, GlobalVolumeAccumulator>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = global_volume_accumulator,
        associated_token::token_program = token_program
    )]
    /// Associated token account of the global volume accumulator holding
    /// the incentive tokens to distribute.
    pub global_incentive_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"user_volume_accumulator", user.key().as_ref()],
        bump
    )]
    /// Per-user volume tracking, synced with the global schedule before
    /// the unclaimed tokens are paid out.
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,
    /// Incentive token mint, must match `global_volume_accumulator.mint`.
    pub mint: InterfaceAccount<'info, Mint>,
    /// Token program owning `mint` (SPL Token or Token-2022).
    pub token_program: Interface<'info, TokenInterface>,
    /// Required to create `user_ata`.
    pub system_program: Program<'info, System>,
    /// Required to create `user_ata`.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Anchor event authority PDA.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, crate::program::PumpAmmRecovered>,
    #[account(mut)]
    /// Pays the rent of `user_ata` when it has to be created.
    pub payer: Signer<'info>,
}

#[event]
#[derive(Clone)]
pub struct ClaimTokenIncentivesEvent {
    /// User the tokens were claimed for.
    pub user: Pubkey,
    /// Incentive token mint.
    pub mint: Pubkey,
    /// Amount of incentive tokens transferred.
    pub amount: u64,
    /// Unix timestamp of the claim.
    pub timestamp: i64,
    /// Total incentive tokens claimed by the user so far.
    pub total_claimed_tokens: u64,
    /// SOL volume of the user in the current day, not yet rewarded.
    pub current_sol_volume: u64,
}

/// Pays out the incentive tokens of `user`.
///
/// The user volume accumulator is first synced with the global schedule:
/// the SOL volume of a finished day is converted into unclaimed tokens
/// proportionally to the share of the day's total volume. Every unclaimed
/// token is then transferred from the global incentive token account to the
/// user's ATA and recorded as claimed.
///
/// Logs "No tokens to claim" and succeeds without a transfer when nothing is
/// claimable yet.
pub fn claim_token_incentives(ctx: Context<ClaimTokenIncentives>) -> Result<()> {
    let global_volume_accumulator = &mut ctx.accounts.global_volume_accumulator;
    let user_volume_accumulator = &mut ctx.accounts.user_volume_accumulator;
    let timestamp = Clock::get()?.unix_timestamp;
    user_volume_accumulator.user = ctx.accounts.user.key();
    global_volume_accumulator.update_user_volume_accumulator(user_volume_accumulator, timestamp, 0)?;

    let amount = user_volume_accumulator.total_unclaimed_tokens;
    if amount == 0 {
        msg!("No tokens to claim");
        return Ok(());
    }

    crate::utils::token::transfer_checked(
        ctx.accounts.token_program.as_ref(),
        &ctx.accounts.mint,
        &ctx.accounts.global_incentive_token_account,
        &ctx.accounts.user_ata,
        &global_volume_accumulator.to_account_info(),
        amount,
        &[&[
            b"global_volume_accumulator",
            &[ctx.bumps.global_volume_accumulator],
        ]],
    )?;
    user_volume_accumulator.claim_tokens(timestamp);

    let event = ClaimTokenIncentivesEvent {
        user: ctx.accounts.user.key(),
        mint: ctx.accounts.mint.key(),
        amount,
        timestamp,
        total_claimed_tokens: user_volume_accumulator.total_claimed_tokens,
        current_sol_volume: user_volume_accumulator.current_sol_volume,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}

/// Claims the SOL (wSOL) cashback accumulated by `user` in the token account
/// owned by its user volume accumulator.
///
/// Accounts:
/// - `user_volume_accumulator` is the PDA of `user`; it owns the cashback
///   vault and signs the transfer out of it,
/// - `user_volume_accumulator_wsol_token_account` is the cashback vault,
/// - `user_wsol_token_account` receives the cashback and must be a wSOL
///   account owned by `user`.
#[derive(Accounts)]
pub struct ClaimCashback<'info> {
    /// CHECK: Destination authority and accumulator seed; no signature required.
    #[account(mut)]
    pub user: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"user_volume_accumulator", user.key().as_ref()],
        bump
    )]
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,
    /// Quote (wSOL) mint the cashback is paid in.
    pub quote_mint: InterfaceAccount<'info, Mint>,
    /// Token program owning `quote_mint`.
    pub quote_token_program: Interface<'info, TokenInterface>,
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = user_volume_accumulator,
        associated_token::token_program = quote_token_program
    )]
    /// Cashback vault: wSOL account owned by the user volume accumulator.
    /// Its whole balance is transferred to `user_wsol_token_account`.
    pub user_volume_accumulator_wsol_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = quote_mint,
        token::authority = user,
        token::token_program = quote_token_program
    )]
    /// wSOL account of `user` receiving the cashback.
    pub user_wsol_token_account: InterfaceAccount<'info, TokenAccount>,
    /// System program.
    pub system_program: Program<'info, System>,
    /// CHECK: Anchor event authority PDA.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: AccountInfo<'info>,
    pub program: Program<'info, crate::program::PumpAmmRecovered>,
}

#[event]
#[derive(Clone)]
pub struct ClaimCashbackEvent {
    /// User the cashback was claimed for.
    pub user: Pubkey,
    /// Amount of wSOL (lamports) transferred.
    pub amount: u64,
    /// Unix timestamp of the claim.
    pub timestamp: i64,
    /// Total cashback claimed by the user so far.
    pub total_claimed: u64,
    /// Total cashback earned by the user so far.
    pub total_cashback_earned: u64,
}

/// Pays out the cashback of `user`.
///
/// Cashback is accrued in wSOL, during trades on cashback coins, into a token
/// account owned by the user volume accumulator PDA. The full balance of that
/// account is transferred to the user's wSOL account, signed by the
/// accumulator seeds, and added to `total_cashback_claimed`.
///
/// Emits `ClaimCashbackEvent` both as a log and through a self-CPI.
#[inline(never)]
pub fn claim_cashback(ctx: Context<ClaimCashback>) -> Result<()> {
    let user_volume_accumulator = &mut ctx.accounts.user_volume_accumulator;
    let timestamp = Clock::get()?.unix_timestamp;
    let amount = ctx.accounts.user_volume_accumulator_wsol_token_account.amount;

    crate::utils::token::transfer_from_user_volume_accumulator(
        &ctx.accounts.quote_token_program,
        &ctx.accounts.quote_mint,
        &ctx.accounts.user_volume_accumulator_wsol_token_account,
        &ctx.accounts.user_wsol_token_account,
        amount,
        &ctx.accounts.user.key(),
        user_volume_accumulator,
        ctx.bumps.user_volume_accumulator,
    )?;
    user_volume_accumulator.total_cashback_claimed = user_volume_accumulator
        .total_cashback_claimed
        .checked_add(amount)
        .ok_or(error!(PumpAmmError::Overflow))?;
    let total_cashback_earned = user_volume_accumulator.cashback_earned;
    let event = ClaimCashbackEvent {
        user: ctx.accounts.user.key(),
        amount,
        timestamp,
        total_claimed: user_volume_accumulator.total_cashback_claimed,
        total_cashback_earned,
    };
    emit!(event);
    emit_cpi!(event);
    Ok(())
}
