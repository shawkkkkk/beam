//! Partial independently compiled Mayhem implementation: 24 recovered administration, sweep,
//! burn and recovered trade handlers. Unknown fields are preserved as byte arrays, not guessed names.
//! This is not a complete or byte-identical program reconstruction.
use anchor_lang::prelude::*;
macro_rules! declare_native_interface { () => { #[account("account")] pub struct BondingCurve { pub virtual_token_reserves: u64, pub virtual_sol_reserves: u64, pub real_token_reserves: u64, pub real_sol_reserves: u64, pub token_total_supply: u64, pub complete: bool, pub creator: Pubkey, pub is_mayhem_mode: bool, } impl Owner for BondingCurve { fn owner() -> Pubkey { crate::state::PUMP_ID } } declare_program!(pump); }; }
mod checked_program;
use checked_program::{CheckedSystemProgram, CheckedSystemProgramBumps, __client_accounts_checked_system_program, __cpi_client_accounts_checked_system_program};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};
declare_id!("MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e");
pub mod instructions; mod account_wrapper;
pub mod state;
pub use instructions::burn::*;
pub use instructions::sell::*;
pub use instructions::buy::*;
pub use instructions::sell_v2::*;
pub use instructions::buy_v2::*;
pub use instructions::buy_pump_swap::*;
pub use instructions::sell_pump_swap::*; pub use curve_accounts::cpi_interface as pump_v2; macro_rules! declare_interfaces { () => { declare_program!(pump_amm); declare_program!(amm_cpi); }; }
declare_native_interface!(); pub use state::{InitializeMayhemState,InitializeMayhemStateBumps}; use state::{__client_accounts_initialize_mayhem_state,__cpi_client_accounts_initialize_mayhem_state};
declare_interfaces!();

#[program]
pub mod mayhem_recovered {
    use super::*;
    pub fn initialize_mayhem_state(ctx: Context<InitializeMayhemState>) -> Result<()> { state::initialize(ctx) }     pub fn buy_pump_swap(ctx: Context<BuyPumpSwap>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::buy_pump_swap::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell_pump_swap(ctx: Context<SellPumpSwap>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::sell_pump_swap::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn buy_v2(ctx: Context<BuyV2>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::pump_interface_v2::buy_v2_handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell_v2(ctx: Context<SellV2>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::sell_v2::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn buy(ctx: Context<LegacyBuy>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> { instructions::buy::handler(ctx, reference, tolerance_bps, retained_reserve_bps) }
    pub fn sell(ctx: Context<LegacySell>, reference: u128, tolerance_bps: u16, retained_reserve_bps: u16) -> Result<()> {
        instructions::sell::handler(ctx, reference, tolerance_bps, retained_reserve_bps)
    }
    pub fn burn_tokens(ctx: Context<BurnTokens>) -> Result<()> {
        instructions::burn::handler(ctx)
    }
    pub fn sweep_fees<'info>(ctx: Context<'info, SweepFees<'info>>) -> Result<()> {
        instructions::fee_recipients::sweep_fees(ctx)

    }
    pub fn sweep_fees_admin<'info>(ctx: Context<'info, SweepFeesAdmin<'info>>) -> Result<()> {
        instructions::fee_recipients::sweep_fees_admin(ctx)


    }
    pub fn withdraw_quote_admin(ctx: Context<WithdrawQuoteAdmin>, amount: u64) -> Result<()> {
        ctx.accounts.global_params.validate_quote_withdraw()?;







        anchor_spl::token::transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.key(), anchor_spl::token::Transfer {
                from: ctx.accounts.sol_vault_quote_ata.to_account_info(),
                to: ctx.accounts.admin_quote_ata.to_account_info(),
                authority: ctx.accounts.sol_vault.to_account_info(),
            }, &[&[b"sol-vault", &[ctx.bumps.sol_vault]]]), amount)?; Ok(())
    }
    pub fn withdraw_admin(ctx: Context<WithdrawAdmin>, amount: u64) -> Result<()> {
        ctx.accounts.global_params.validate_native_withdraw()?;










        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(ctx.accounts.system_program.key(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.sol_vault.to_account_info(),
                    to: ctx.accounts.admin.to_account_info(),
                }, &[&[b"sol-vault", &[ctx.bumps.sol_vault]]]), amount)?; Ok(())
    }
    pub fn extend_account(ctx: Context<ExtendAccount>) -> Result<()> {
        instructions::extend_account::handler(ctx)
    }



































    pub fn initialize_fee_recipient(ctx: Context<InitializeFeeRecipient>, index: u8) -> Result<()> {
        instructions::fee_recipients::initialize(ctx,index)
    }



    pub fn initialize_global_params(ctx: Context<InitializeGlobalParams>) -> Result<()> {
let state = &mut ctx.accounts.global_params; state.field_0 = 0; state.field_1 = 0; state.field_2 = 0; state.field_3 = 0; state.field_4 = 0; state.field_5 = 0; state.paused = false; state.sol_deployed = 0; state.field_6 = 0; state.field_138 = 0; state.field_7 = 0; state.field_8 = 0; state.field_11 = 0; state.field_12 = 0; state.field_13 = 0; state.stable_deployed = 0; state.field_14 = 0; state.field_15 = 0; state.field_238 = 0; state.admin = ctx.accounts.admin.key();
        Ok(())
    }
    pub fn update_global_params(ctx: Context<Admin>, field_0: Option<u32>, field_1: Option<u32>, field_2: Option<u64>, field_3: Option<u64>, field_4: Option<u64>, field_5: Option<u64>, field_6: Option<u8>, field_7: Option<u64>, field_8: Option<u64>, field_9: Option<u64>, field_10: Option<u64>, field_11: Option<u64>, field_12: Option<u64>, field_13: Option<u64>, field_14: Option<u64>, field_15: Option<u64>) -> Result<()> {
        let state = &mut ctx.accounts.global_params;
        let old_version = state.version;
        state.apply_update(field_0,field_1,field_2,field_3,field_4,field_5,field_6,field_7,field_8,field_9,field_10,field_11,field_12,field_13,field_14,field_15)?;
















        let event = GlobalParamsUpdateEvent {
            old_version, new_version: state.version,
            field_0: state.field_0,
            field_1: state.field_1,
            field_2: state.field_2,
            field_3: state.field_3,
            field_4: state.field_4,
            field_5: state.field_5,
            field_6: state.field_6,
            field_7: state.field_7,
            field_8: state.field_8,
            field_9: state.field_9,
            field_10: state.field_10,
            field_11: state.field_11,
            field_12: state.field_12,
            field_13: state.field_13,
            field_14: state.field_14,
            field_15: state.field_15,
        };
        emit!(event);
        emit_cpi!(event);
        Ok(())
    }
    pub fn update_admin(ctx: Context<Admin>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.global_params.admin = new_admin;
        Ok(())
    }
    pub fn update_trader(ctx: Context<Admin>, new_trader: Pubkey) -> Result<()> {
        ctx.accounts.global_params.trader = new_trader;
        Ok(())
    }
    pub fn update_max_sol_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> {
        ctx.accounts.global_params.max_sol_per_sweep = value;
        Ok(())
    }
    pub fn update_max_stable_per_sweep(ctx: Context<Admin>, value: u64) -> Result<()> {
        ctx.accounts.global_params.max_stable_per_sweep = value;
        Ok(())
    }
    pub fn reset_sol_deployed(ctx: Context<Admin>) -> Result<()> {
        ctx.accounts.global_params.sol_deployed = 0;
        Ok(())
    }
    pub fn reset_stable_deployed(ctx: Context<Admin>) -> Result<()> {
        ctx.accounts.global_params.stable_deployed = 0;
        Ok(())
    }
    pub fn reset_state(ctx: Context<Admin>) -> Result<()> {
        let state = &mut ctx.accounts.global_params;
        state.field_0 = 0;
        state.field_1 = 0;
        state.field_2 = 0;
        state.field_3 = 0;
        state.field_4 = 0;
        state.field_5 = 0;
        state.paused = false;
        state.sol_deployed = 0;
        state.field_6 = 0;
        state.field_138 = 0;
        state.field_7 = 0;
        state.field_8 = 0;
        state.field_11 = 0;
        state.field_12 = 0;
        state.field_13 = 0;
        state.stable_deployed = 0;
        state.field_14 = 0;
        state.field_15 = 0;
        state.field_238 = 0;
        Ok(())
    }
    pub fn stop_trading(ctx: Context<Admin>) -> Result<()> {
        ctx.accounts.global_params.stop_trading()


    }
    pub fn resume_trading(ctx: Context<Admin>) -> Result<()> {
        ctx.accounts.global_params.resume_trading()



    }
}

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"global-params"], bump, has_one = admin)]
    pub global_params: Account<'info, GlobalParams>,
    pub system_program: Program<'info,System>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

/// Exact observed Borsh positions; opaque bytes remain named by behavior.
/// This is not an assertion of historical field names or Rust memory layout.
pub use state::GlobalParams;

#[error_code]
pub enum MayhemError {
    #[msg("Trading is paused")]
    TradingPaused = 0,
    #[msg("Trading is active")]
    TradingActive = 1,
    #[msg("Math overflow")]
    MathOverflow = 2,
    #[msg("Too little sol reserves")]
    SolReservesTooLow = 3,
    #[msg("Mayhem mode ended")]
    MayhemModeEnded = 4,
    #[msg("Mayhem mode not ended")]
    MayhemModeNotEnded = 5,
    #[msg("Overall risk tolerance exceeded")]
    OverallRiskExceeded = 6,
    #[msg("Per coin risk tolerance exceeded")]
    CoinRiskExceeded = 7,
    #[msg("Pump CPI call failed")]
    PumpCpiFailed = 8,
    #[msg("Math Overflow in Pump CPI call")]
    PumpCpiMathOverflow = 9,
    #[msg("Pump Swap CPI call failed")]
    PumpSwapCpiFailed = 10,
    #[msg("Math Overflow in Pump Swap CPI call")]
    PumpSwapCpiMathOverflow = 11,
    #[msg("Price impact exceeds slippage tolerance in Pump CPI call")]
    PumpCpiSlippage = 12,
    #[msg("Price impact exceeds slippage tolerance in Pump Swap CPI call")]
    PumpSwapCpiSlippage = 13,
    #[msg("Withdraw Rate limit")]
    WithdrawRateLimit = 14,
    #[msg("Trade Rate limit")]
    TradeRateLimit = 15,
    #[msg("Account type not supported")]
    AccountTypeNotSupported = 16,
    #[msg("Only 8 fee recipients supported")]
    MaxFeeRecipientsExceeded = 17,
    #[msg("Invalid bump provided")]
    InvalidBump = 18,
    #[msg("Sweep fee Rate limit")]
    SweepFeeRateLimit = 19,
}

/// Recovered serialized widths/order. Semantic names remain unresolved.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateGlobalParamsArgs {
    pub field_0: Option<u32>,
    pub field_1: Option<u32>,
    pub field_2: Option<u64>,
    pub field_3: Option<u64>,
    pub field_4: Option<u64>,
    pub field_5: Option<u64>,
    pub field_6: Option<u8>,
    pub field_7: Option<u64>,
    pub field_8: Option<u64>,
    pub field_9: Option<u64>,
    pub field_10: Option<u64>,
    pub field_11: Option<u64>,
    pub field_12: Option<u64>,
    pub field_13: Option<u64>,
    pub field_14: Option<u64>,
    pub field_15: Option<u64>,
}

// The discriminator is observed from original emitted data. Historical event
// type name remains unresolved; overriding it avoids pretending to know it.






















pub use instructions::admin::*;








pub use instructions::fee_recipients::FeeRecipient;






pub use instructions::fee_recipients::*;


















#[derive(Accounts)]
pub struct ExtendAccount<'info> {
    /// CHECK: Owner/mut checked here; recognized discriminator checked in handler.
    #[account(mut, owner = ID)]
    pub account: UncheckedAccount<'info>,
    pub user: Signer<'info>,
    pub system_program: Program<'info,System>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

pub use instructions::extend_account::ExtendAccountEvent;








#[derive(Accounts)]
pub struct WithdrawAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"global-params"], bump, has_one = admin)]
    pub global_params: Account<'info, GlobalParams>,
    /// CHECK: PDA/writability enforced here; System CPI enforces spending owner.
    #[account(mut, seeds = [b"sol-vault"], bump)]
    pub sol_vault: UncheckedAccount<'info>,
    pub system_program: Program<'info,System>,
}

#[derive(Accounts)]
pub struct WithdrawQuoteAdmin<'info> {
    #[account(mut)] pub admin: Signer<'info>,
    #[account(mut, seeds = [b"global-params"], bump, has_one = admin)]
    pub global_params: Account<'info, GlobalParams>,
    /// CHECK: PDA authorizes its token accounts.
    #[account(mut, seeds = [b"sol-vault"], bump)] pub sol_vault: UncheckedAccount<'info>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint = quote_mint, associated_token::authority = sol_vault, associated_token::token_program = token_program)]
    pub sol_vault_quote_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(init_if_needed, payer = admin, associated_token::mint = quote_mint, associated_token::authority = admin, associated_token::token_program = token_program)]
    pub admin_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info,System>,
}


































#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)] pub trader: Signer<'info>,
    #[account(mut, seeds=[b"global-params"], bump, has_one=trader)] pub global_params: Account<'info, GlobalParams>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint=quote_mint, associated_token::authority=trader, associated_token::token_program=token_program)] pub trader_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info,System>,
    #[account(address=anchor_spl::token::ID)] pub token_program: Interface<'info,TokenInterface>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SweepFeesAdmin<'info> {
    #[account(mut)] pub admin: Signer<'info>,
    #[account(mut, seeds=[b"global-params"], bump, has_one=admin)] pub global_params: Account<'info, GlobalParams>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint=quote_mint, associated_token::authority=admin, associated_token::token_program=token_program)] pub admin_quote_ata: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info,System>,
    #[account(address=anchor_spl::token::ID)] pub token_program: Interface<'info,TokenInterface>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    /// CHECK: Preserve the captured runtime PDA derivation and ConstraintSeeds error.
    #[account(seeds = [b"__event_authority"], bump)]
    pub event_authority: UncheckedAccount<'info>,
    /// CHECK: Captured parser consumes this account without an address constraint.
    pub program: UncheckedAccount<'info>,
}
