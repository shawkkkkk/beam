use anchor_lang::prelude::*;
use crate::checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};
use crate::state::{BondingCurve, MayhemState, MayhemStatus, PUMP_ID};





#[event]
pub struct BurnEvent {
    pub amount: u64,
    pub mint: Pubkey,
    pub mayhem_state: Pubkey,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut, seeds = [b"mayhem-state", mint.key().as_ref()], bump)]
    pub mayhem_state: Account<'info, MayhemState>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = sol_vault,
        associated_token::token_program = mint.to_account_info().owner)]
    pub mayhem_token_vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Address and writable flag are checked below; original parser uses an unchecked account.
    #[account(mut, seeds = [b"sol-vault"], bump)]
    pub sol_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"bonding-curve", mint.key().as_ref()], bump, seeds::program = PUMP_ID)]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    /// CHECK: The captured burn parser consumes this account without further constraints.
    pub global: UncheckedAccount<'info>,
    /// CHECK: The captured parser checks its address; it does not require an executable Program wrapper.
    #[account(address = PUMP_ID)]
    pub pump_program: UncheckedAccount<'info>,
    /// CHECK: The captured burn parser consumes this account without further constraints.
    pub pump_event_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info,System>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Original parser derives this PDA at runtime and returns ConstraintSeeds.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Original parser only consumes the program account.
    pub program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BurnTokens>) -> Result<()> {
    // 0xd8a8..0xda30: Clock failure follows unwrap panic, deadline failure is 6005.
























    let end_timestamp = ctx.accounts.mayhem_state.end_timestamp;
    let current_unix_timestamp: i64 = Clock::get().unwrap().unix_timestamp;
    if end_timestamp > current_unix_timestamp { return Err(anchor_lang::error!(BurnError::MayhemModeNotEnded)); }

    // 0xda38..0xdb90: a second burn is rejected with 6004.

    require!(ctx.accounts.mayhem_state.status != MayhemStatus::Ended,
        BurnError::MayhemModeEnded);
    let amount = ctx.accounts.mayhem_token_vault.amount;


    token_interface::burn(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        token_interface::Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.mayhem_token_vault.to_account_info(),
            authority: ctx.accounts.sol_vault.to_account_info(),
        }, &[&[b"sol-vault", &[ctx.bumps.sol_vault]]]), amount)?;
    ctx.accounts.mayhem_state.burned_amount = amount;
    // The binary reads Clock independently for the two event paths.






    emit!(BurnEvent { amount, mint: ctx.accounts.mint.key(),
        mayhem_state: ctx.accounts.mayhem_state.key(),
        timestamp: Clock::get().unwrap().unix_timestamp });



    emit_cpi!(BurnEvent { amount, mint: ctx.accounts.mint.key(),
        mayhem_state: ctx.accounts.mayhem_state.key(),
        timestamp: Clock::get().unwrap().unix_timestamp });
    ctx.accounts.mayhem_state.status = MayhemStatus::Ended;
    Ok(())
}



// Keep the published module path while using the shared error namespace.
pub use crate::MayhemError as BurnError;
