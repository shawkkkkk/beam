//! Partial PumpSwap reconstruction (not the historical source or a byte-identical replacement).
use anchor_lang::prelude::*;
mod errors; pub use errors::*;
pub mod curve;
mod instructions;
mod trade;
mod state;
use instructions::admin::{admin_cto_pool, admin_cto_pool::*, admin_update_token_incentives, admin_update_token_incentives::*, create_config::*, set_reserved_fee_recipients, set_reserved_fee_recipients::*};
pub use instructions::token_incentives::{init_user_volume_accumulator::{InitUserVolumeAccumulator, init_user_volume_accumulator as init_uva_handler}, close_user_volume_accumulator::close_user_volume_accumulator as close_uva_handler, sync_user_volume_accumulator::sync_user_volume_accumulator as sync_uva_handler};
use instructions::token_incentives::*;
use instructions::swap::buy as legacy_buy;
use legacy_buy::*;
pub use instructions::swap::sell as legacy_sell;
use legacy_sell::*;
pub use instructions::token_incentives::claim::{ClaimTokenIncentives, claim_token_incentives as claim_token_incentives_handler};
pub use instructions::token_incentives::claim::ClaimCashback;
mod incentives;
use incentives::*;
mod creator_reassignment;
use creator_reassignment::*;
mod multi_hop; mod create; pub mod utils;
anchor_lang::declare_program!(pump);
anchor_lang::declare_program!(pump_fees);


pub use instructions::{boost, liquidity, swap::{multi_hop_swap::execute as multi_hop_swap_execute, trade_v2}, coin_creator::{migrate_pool_coin_creator, set_coin_creator}};

pub use instructions::sweep_fees as sweep;
pub use instructions::extend_account as extend;
pub use instructions::coin_creator::collect_coin_creator_fee as collect;
pub use instructions::coin_creator::transfer_creator_fees_to_pump as transfer_creator;
use anchor_spl::{token::Token, token_2022::Token2022, token_interface::{self, Mint, TokenAccount}};
pub use instructions::admin;
declare_id!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

#[program]
pub mod pump_amm_recovered {
    use super::*;
    // Static-only reconstruction: no authentic successful initializer replay exists.
    pub fn create_config(ctx: Context<CreateConfig>, lp_fee_basis_points: u64, protocol_fee_basis_points: u64, protocol_fee_recipients: [Pubkey; 8], coin_creator_fee_basis_points: u64, admin_set_coin_creator_authority: Pubkey) -> Result<()> { instructions::admin::create_config::create_config(ctx, lp_fee_basis_points, protocol_fee_basis_points, protocol_fee_recipients, coin_creator_fee_basis_points, admin_set_coin_creator_authority) }

    pub fn disable(ctx: Context<AdminGlobal>, disable_create_pool: bool, disable_deposit: bool, disable_withdraw: bool, disable_buy: bool, disable_sell: bool) -> Result<()> { instructions::admin::disable::disable(ctx, disable_create_pool, disable_deposit, disable_withdraw, disable_buy, disable_sell) }
    pub fn update_admin(ctx: Context<UpdateAdmin>) -> Result<()> { instructions::admin::update_admin::update_admin(ctx) }
    pub fn update_fee_config(ctx: Context<AdminGlobal>, lp_fee_basis_points: u64, protocol_fee_basis_points: u64, protocol_fee_recipients: [Pubkey; 8], coin_creator_fee_basis_points: u64, admin_set_coin_creator_authority: Pubkey) -> Result<()> { instructions::admin::update_fee_config::update_fee_config(ctx, lp_fee_basis_points, protocol_fee_basis_points, protocol_fee_recipients, coin_creator_fee_basis_points, admin_set_coin_creator_authority) }
    pub fn update_creator_fee_config(ctx: Context<UpdateCreatorFeeConfig>, creator_fee_configurable: bool, max_configurable_creator_fee_bps: u64) -> Result<()> { instructions::admin::update_creator_fee_config::update_creator_fee_config(ctx, creator_fee_configurable, max_configurable_creator_fee_bps) }
    pub fn create_pool(ctx: Context<CreatePool>, index: u16, base_amount_in: u64, quote_amount_in: u64, coin_creator: Pubkey, is_mayhem_mode: bool, is_cashback_coin: OptionBool, creator_fee_bps: OptionU64, can_edit_creator_fee: OptionBool, is_holder_reward: OptionBool) -> Result<()> { create::execute(ctx,index,base_amount_in,quote_amount_in,coin_creator,is_mayhem_mode,is_cashback_coin.0,creator_fee_bps.0,can_edit_creator_fee.0,is_holder_reward.0) }
    pub fn init_boost(ctx: Context<InitBoost>) -> Result<()> { boost::execute(ctx) }
    pub fn set_boost_authority(ctx: Context<SetBoostAuthority>) -> Result<()> { instructions::admin::set_boost_authority::set_boost_authority(ctx) }
    pub fn boost_buy_and_burn(ctx: Context<BoostBuyAndBurn>, quote_amount_in: u64, min_base_amount_burned: u64) -> Result<()> { boost::buy_and_burn(ctx, quote_amount_in, min_base_amount_burned) }
    /// Sets the reserved fee recipients (the first eight remaining accounts) and the
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    ///
    /// whitelist PDA. See `instructions::admin::set_reserved_fee_recipients`.
    pub fn set_reserved_fee_recipients(ctx: Context<SetReservedFeeRecipients>, whitelist_pda: Pubkey) -> Result<()> {
        set_reserved_fee_recipients::handler(ctx, whitelist_pda)
    }
    pub fn deposit(ctx: Context<Liquidity>, lp_token_amount_out: u64, max_base_amount_in: u64, max_quote_amount_in: u64) -> Result<()> {
        instructions::liquidity::deposit::deposit(ctx, lp_token_amount_out, max_base_amount_in, max_quote_amount_in)
    }
    pub fn withdraw(ctx: Context<Liquidity>, lp_token_amount_in: u64, min_base_amount_out: u64, min_quote_amount_out: u64) -> Result<()> {
        instructions::liquidity::withdraw::withdraw(ctx, lp_token_amount_in, min_base_amount_out, min_quote_amount_out)
    }

    pub fn admin_update_token_incentives(ctx:Context<AdminUpdateTokenIncentives>,start_time:i64,end_time:i64,seconds_in_a_day:i64,day_number:u64,token_supply_per_day:u64)->Result<()>{admin_update_token_incentives::handler(ctx,start_time,end_time,seconds_in_a_day,day_number,token_supply_per_day)}
    pub fn claim_token_incentives(ctx:Context<ClaimTokenIncentives>)->Result<()>{instructions::token_incentives::claim::claim_token_incentives(ctx)}

    pub fn init_user_volume_accumulator(ctx: Context<InitUserVolumeAccumulator>) -> Result<()> { init_uva_handler(ctx) }

    pub fn close_user_volume_accumulator(ctx: Context<CloseUserVolumeAccumulator>) -> Result<()> { close_uva_handler(ctx) }

    pub fn sync_user_volume_accumulator(ctx: Context<SyncUserVolumeAccumulator>) -> Result<()> { sync_uva_handler(ctx) }
    pub fn buy<'info>(ctx:Context<'_,'_,'_,'info,LegacyBuy<'info>>,base_amount_out:u64,max_quote_amount_in:u64,track_volume:OptionBool)->Result<()>{legacy_buy::buy(ctx,base_amount_out,max_quote_amount_in,track_volume.0)}

    pub fn buy_exact_quote_in<'info>(ctx:Context<'_,'_,'_,'info,LegacyBuy<'info>>,spendable_quote_in:u64,min_base_amount_out:u64,track_volume:OptionBool)->Result<()>{legacy_buy::buy_exact_quote_in(ctx,spendable_quote_in,min_base_amount_out,track_volume.0)}
    pub fn sell<'info>(ctx:Context<'_,'_,'info,'info,LegacySell<'info>>,base_amount_in:u64,min_quote_amount_out:u64)->Result<()>{legacy_sell::execute(ctx,base_amount_in,min_quote_amount_out)}

    pub fn buy_v2(ctx: Context<TradeV2>, base_amount_out: u64, max_quote_amount_in: u64) -> Result<()> { trade::execute_buy(ctx, base_amount_out, max_quote_amount_in) }

    pub fn buy_exact_quote_in_v2(ctx: Context<TradeV2>, spendable_quote_in: u64, min_base_amount_out: u64) -> Result<()> { trade::execute_buy_exact_quote(ctx, spendable_quote_in, min_base_amount_out) }
    pub fn sell_v2(ctx: Context<TradeV2>, base_amount_in: u64, min_quote_amount_out: u64) -> Result<()> { trade::execute_sell(ctx, base_amount_in, min_quote_amount_out) }
    pub fn multi_hop_swap<'info>(ctx:Context<'_, '_, 'info, 'info, MultiHop<'info>>,amount_in:u64,min_amount_out:u64)->Result<()>{multi_hop::execute(ctx,amount_in,min_amount_out)}
    pub fn sweep_protocol_fee(ctx: Context<SweepFees>) -> Result<()> { sweep::execute(ctx,false) }
    pub fn sweep_creator_fee(ctx: Context<SweepFees>) -> Result<()> { sweep::execute(ctx,true) }

    pub fn extend_account(ctx: Context<ExtendAccount>) -> Result<()> { extend::execute(ctx) }
    pub fn collect_coin_creator_fee(ctx: Context<CollectCoinCreatorFee>) -> Result<()> { collect::execute(ctx) }

    pub fn set_coin_creator(ctx:Context<SetCoinCreator>)->Result<()>{creator_reassignment::set(ctx)}

    pub fn admin_cto_pool(ctx:Context<AdminCtoPool>,coin_creator:Pubkey,is_holder_reward:bool,creator_fee_bps:Option<u64>)->Result<()>{admin_cto_pool::handler(ctx,coin_creator,is_holder_reward,creator_fee_bps)}

    pub fn toggle_mayhem_mode(ctx: Context<AdminGlobal>, enabled: bool) -> Result<()> {
        instructions::admin::toggle_mayhem_mode::toggle_mayhem_mode(ctx, enabled)
    }

    pub fn toggle_boost(ctx: Context<AdminGlobalNoEvent>, enabled: bool) -> Result<()> {
        instructions::admin::toggle_boost::toggle_boost(ctx, enabled)
    }

    pub fn transfer_creator_fees_to_pump(ctx: Context<TransferCreatorFeesToPump>) -> Result<()> { transfer_creator::legacy(ctx) }

    pub fn transfer_creator_fees_to_pump_v2(ctx: Context<TransferCreatorFeesToPumpV2>) -> Result<()> { transfer_creator::v2(ctx) }
    pub fn migrate_pool_coin_creator(ctx:Context<MigratePoolCoinCreator>)->Result<()>{creator_reassignment::migrate(ctx)}
    pub fn toggle_cashback_enabled(ctx: Context<AdminGlobal>, enabled: bool) -> Result<()> {
        instructions::admin::toggle_cashback_enabled::toggle_cashback_enabled(ctx, enabled)
    }

    pub fn claim_cashback(ctx: Context<ClaimCashback>) -> Result<()> {
        instructions::token_incentives::claim::claim_cashback(ctx)
    }

    pub fn update_buyback_config(ctx: Context<AdminGlobal>, buyback_basis_points: Option<u64>) -> Result<()> {
        instructions::admin::update_buyback_config::handler(ctx, buyback_basis_points)
    }










}
use transfer_creator::*;
use collect::*;
use extend::*;
use sweep::*;
use boost::*;

use create::*;
use trade::*;
use {multi_hop::*, state::*};
pub use instructions::admin::{AdminGlobal, UpdateAdmin, AdminGlobalNoEvent, SetBoostAuthority, UpdateCreatorFeeConfig};
pub(crate) use instructions::admin::{__client_accounts_admin_global, __client_accounts_update_admin, __client_accounts_admin_global_no_event, __client_accounts_set_boost_authority, __client_accounts_update_creator_fee_config};
pub(crate) use instructions::admin::{__cpi_client_accounts_admin_global, __cpi_client_accounts_update_admin, __cpi_client_accounts_admin_global_no_event, __cpi_client_accounts_set_boost_authority, __cpi_client_accounts_update_creator_fee_config};
pub use instructions::liquidity::Liquidity;
pub(crate) use instructions::liquidity::{__client_accounts_liquidity, __cpi_client_accounts_liquidity};

fn liquidity_total_fees(pool: &Pool) -> Result<u64> {
    pool.total_accrued_fees()
}
fn liquidity_quote_reserves(pool: &Pool, quote_token_amount: u64) -> Result<u64> {
    pool.available_quote_reserves(quote_token_amount)
}
fn liquidity_effective_virtual(pool: &Pool) -> Result<i128> {
    pool.total_virtual_quote_reserves()
}

fn quote_transfer_fee(mint: &AccountInfo, amount: u64, inverse: bool) -> Result<u64> {
    let fee = crate::utils::transfer_fee::from_mint(mint)?;
    let pre_fee_amount = if inverse {
        crate::utils::transfer_fee::amount_before_fee(&fee, amount)?
    } else {
        amount
    };
    crate::utils::transfer_fee::calculate_fee(&fee, pre_fee_amount)
}
