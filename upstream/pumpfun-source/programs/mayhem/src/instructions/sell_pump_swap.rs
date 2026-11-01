//! Independently reconstructed PumpSwap handler and account constraints.
//! Byte-checked original parser 0x46378 (buy), 0x4cc98 (sell).
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use curve_accounts::token_interface::{Mint,TokenAccount,TokenInterface};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::MayhemStatus;
use anchor_lang::solana_program::{instruction::Instruction,program::invoke_signed};
use super::sell::{SellEvent,SellError};
use super::buy::BuyError;
use super::pump_swap_types::{SwapPool,SwapGlobalConfig,AMM_ID,PumpSwapError};
const WSOL:Pubkey=pubkey!("So11111111111111111111111111111111111111112");
use crate::state::MayhemState;
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};
const SPL_TOKEN_ID:Pubkey=pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
#[derive(Accounts)]
pub struct SellPumpSwap<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]
 pub mayhem_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 #[account(init_if_needed,payer=trader,associated_token::mint=quote_mint,associated_token::authority=sol_vault,associated_token::token_program=quote_token_program)]
 pub mayhem_quote_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: original PDA+mut only; no SystemAccount ownership check.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 // Original parses Interface then applies address constraint 2012 at 0x49568.
 #[account(address=SPL_TOKEN_ID)]pub quote_token_program:Interface<'info,TokenInterface>,
 pub base_token_program:Program<'info,Token2022>,
 pub associated_token_program:Program<'info,AssociatedToken>,
 // Original has no pool seeds or has_one checks, only typed owner/discriminator + mut.
 #[account(mut)]pub pool:Box<Account<'info,SwapPool>>,
 pub global_config:Box<Account<'info,SwapGlobalConfig>>,
 /// CHECK: original consumes AccountInfo directly at 0x47560, no Mint deserialize.
 pub quote_mint:UncheckedAccount<'info>,
 #[account(mut)]pub pool_base_token_account:Box<InterfaceAccount<'info,TokenAccount>>,
 #[account(mut)]pub pool_quote_token_account:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: original mut only; membership checked by AMM CPI.
 #[account(mut)]pub protocol_fee_recipient:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=quote_mint,associated_token::authority=protocol_fee_recipient,associated_token::token_program=quote_token_program)]
 pub protocol_fee_recipient_token_account:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: original unchecked, no outer seed check.
 pub pump_swap_event_authority:UncheckedAccount<'info>,
 /// CHECK: original fixed address only, not Program<...>.
 #[account(address=AMM_ID)]pub pump_swap_program:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=quote_mint,associated_token::authority=coin_creator_vault_authority,associated_token::token_program=quote_token_program)]
 pub coin_creator_vault_ata:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: original unchecked, actual AMM CPI validates authority.
 pub coin_creator_vault_authority:UncheckedAccount<'info>,
 /// CHECK: original unchecked.
 pub fee_config:UncheckedAccount<'info>,
 /// CHECK: original unchecked.
 pub fee_program:UncheckedAccount<'info>,
 /// CHECK: original Mayhem-derived event authority.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::MayhemRecovered>,
 /// CHECK: original PDA is derived under MAYHEM; not AMM; no mut/owner check.
 #[account(seeds=[b"pool-v2",mint.key().as_ref()],bump)]pub pool_v2:UncheckedAccount<'info>,
 /// CHECK: original unchecked and read-only.
 pub buyback_recipient:UncheckedAccount<'info>,
 /// CHECK: original unchecked + mut; actual AMM CPI validates ATA.
 #[account(mut)]pub buyback_recipient_ata:UncheckedAccount<'info>,
} #[inline(always)] fn effective_quote(amount:u64,virtual_quote:i128)->core::result::Result<u64,MayhemError> { let q=(amount as i128).checked_add(virtual_quote).ok_or(MayhemError::MathOverflow)?; u64::try_from(q).map_err(|_|MayhemError::MathOverflow) }
#[inline(always)] fn price_check(vt:u128,vs:u128,reference:u128,tolerance_bps:u16)->core::result::Result<(),MayhemError> { let price=(vs*1_000_000_000_000_000u128).checked_div(vt).ok_or(MayhemError::MathOverflow)?; let scaled=reference.checked_mul(tolerance_bps as u128); let delta=scaled.ok_or(MayhemError::MathOverflow)?/10_000; let Some(low)=reference.checked_sub(delta) else {return Err(MayhemError::MathOverflow)}; let Some(high)=reference.checked_add(delta) else {return Err(MayhemError::MathOverflow)}; if price<low||price>high{return Err(MayhemError::PumpCpiSlippage);} Ok(()) }
fn math()->Error {MayhemError::MathOverflow.into()}
pub fn handler(ctx:Context<SellPumpSwap>,reference:u128,tolerance_bps:u16,retained_reserve_bps:u16)->Result<()> {
 let a=&mut *ctx.accounts;
 a.mayhem_state.validate_trade(&a.global_params)?;
 let effective_quote=effective_quote(a.pool_quote_token_account.amount,a.pool.virtual_quote_reserves)?;
 a.mayhem_state.validate_quote(effective_quote,&a.global_params,&a.quote_mint.key())?;
 price_check(a.pool_base_token_account.amount as u128,effective_quote as u128,reference,tolerance_bps)?;
 let (input,minimum)=crate::state::sell_quote(&a.global_params,a.pool_base_token_account.amount,effective_quote,a.mayhem_token_vault.amount,retained_reserve_bps,a.quote_mint.key())?;
 let native=a.quote_mint.key()==WSOL;
 let before_sol=if native{a.sol_vault.lamports()}else{a.mayhem_quote_vault.amount};let before_tokens=a.mayhem_token_vault.amount;
 crate::amm_cpi::cpi::sell(CpiContext::new_with_signer(a.pump_swap_program.key(),crate::amm_cpi::cpi::accounts::Sell{user:a.sol_vault.to_account_info(),pool:a.pool.to_account_info(),global_config:a.global_config.to_account_info(),pool_base_token_account:a.pool_base_token_account.to_account_info(),pool_quote_token_account:a.pool_quote_token_account.to_account_info(),user_base_token_account:a.mayhem_token_vault.to_account_info(),user_quote_token_account:a.mayhem_quote_vault.to_account_info(),quote_token_program:a.quote_token_program.to_account_info(),base_token_program:a.base_token_program.to_account_info(),system_program:a.system_program.to_account_info(),quote_mint:a.quote_mint.to_account_info(),base_mint:a.mint.to_account_info(),associated_token_program:a.associated_token_program.to_account_info(),event_authority:a.pump_swap_event_authority.to_account_info(),program:a.pump_swap_program.to_account_info(),protocol_fee_recipient:a.protocol_fee_recipient.to_account_info(),protocol_fee_recipient_token_account:a.protocol_fee_recipient_token_account.to_account_info(),coin_creator_vault_ata:a.coin_creator_vault_ata.to_account_info(),coin_creator_vault_authority:a.coin_creator_vault_authority.to_account_info(),fee_config:a.fee_config.to_account_info(),fee_program:a.fee_program.to_account_info()},&[&[b"sol-vault",&[ctx.bumps.sol_vault]]])
 .with_remaining_accounts(vec![a.buyback_recipient.to_account_info(),a.buyback_recipient_ata.to_account_info()]),input,minimum).map_err(super::pump_amm_interface::sell_cpi_failed)?;
 if native {curve_accounts::token_interface::close_account(CpiContext::new_with_signer(a.quote_token_program.key(),curve_accounts::token_interface::CloseAccount{account:a.mayhem_quote_vault.to_account_info(),destination:a.sol_vault.to_account_info(),authority:a.sol_vault.to_account_info()},&[&[b"sol-vault",&[ctx.bumps.sol_vault]]]))?;}
 a.mayhem_token_vault.reload()?;a.pool_base_token_account.reload()?;a.pool_quote_token_account.reload()?;if !native{a.mayhem_quote_vault.reload()?;}
 let after_quote=if native{a.sol_vault.lamports()}else{a.mayhem_quote_vault.amount};
 let (sol,tokens)=super::pump_amm_interface::sold_amounts(before_sol,after_quote,before_tokens,a.mayhem_token_vault.amount);
 crate::state::record_sell(&mut a.global_params,&mut a.mayhem_state,sol,tokens,&a.quote_mint.key())?;
 let quote_vault_balance=if native{a.sol_vault.lamports()}else{a.mayhem_quote_vault.amount};let state_quote_deployed=a.mayhem_state.field_56;let global_sol_deployed=if native{a.global_params.sol_deployed}else{0};let virtual_quote=a.pool_quote_token_account.amount;let real_quote=a.pool_quote_token_account.amount;let global_quote_deployed=if native{a.global_params.sol_deployed}else{a.global_params.stable_deployed};let sol_amount=if native{sol}else{0};let sol_vault_balance=if native{quote_vault_balance}else{0};let state_sol_deployed=if native{a.mayhem_state.field_56}else{0};let virtual_sol=if native{a.pool_quote_token_account.amount}else{0};let real_sol=if native{a.pool_quote_token_account.amount}else{0};let quote_mint=if native{Pubkey::default()}else{a.quote_mint.key()}; let event=SellEvent{side:true,quote_vault_balance,state_quote_deployed,global_sol_deployed,virtual_quote,real_quote,global_quote_deployed,sol_amount,sol_vault_balance,state_sol_deployed,virtual_sol,real_sol,sol_vault:a.sol_vault.key(),mint:a.mint.key(),token_amount:tokens,token_vault_balance:a.mayhem_token_vault.amount,state_token_net:a.mayhem_state.field_72,virtual_tokens:a.pool_base_token_account.amount,end_timestamp:a.mayhem_state.end_timestamp,timestamp:Clock::get()?.unix_timestamp,real_tokens:a.pool_base_token_account.amount,version:a.global_params.version,retained_reserve_bps,quote_amount:sol,quote_mint};
 emit!(event);emit_cpi!(event);Ok(())




























}
