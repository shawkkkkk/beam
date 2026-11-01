use anchor_lang::prelude::*;
use crate::MayhemError;
use super::sell_v2::SellV2; use super::buy_v2::BuyV2;
use super::sell::SellEvent;
use super::buy::BuyError;
const WSOL:Pubkey=pubkey!("So11111111111111111111111111111111111111112");





































































































































































































































































































#[inline]
fn price_check(c:&curve_accounts::BondingCurve,reference:u128,tolerance_bps:u16)->std::result::Result<(),MayhemError> {
 let vt=c.virtual_token_reserves as u128;
 let vs=c.virtual_sol_reserves as u128;let price=(vs*1_000_000_000_000_000u128).checked_div(vt).ok_or(MayhemError::MathOverflow)?;
 let delta=match reference.checked_mul(tolerance_bps as u128){Some(v)=>v,None=>return Err(MayhemError::MathOverflow)}/10_000;
 let low=reference.checked_sub(delta).ok_or(MayhemError::MathOverflow)?;
 let high=match reference.checked_add(delta){Some(v)=>v,None=>return Err(MayhemError::MathOverflow)};
 if price<low||price>high{return Err(MayhemError::PumpCpiSlippage);}
 Ok(())
}













































pub fn buy_v2_handler(ctx:Context<BuyV2>,reference:u128,tolerance_bps:u16,retained_reserve_bps:u16)->Result<()> {
 let a=&mut *ctx.accounts;
 a.mayhem_state.validate_trade(&a.global_params)?;
 a.mayhem_state.validate_quote(a.bonding_curve.real_sol_reserves,&a.global_params,&a.quote_mint.key())?;
 let c=&a.bonding_curve;
 let price_check:std::result::Result<(),MayhemError> = (|| {
 let vt=c.virtual_token_reserves as u128;let vs=c.virtual_sol_reserves as u128;
 let Some(price)=(vs*1_000_000_000_000_000u128).checked_div(vt) else {return Err(MayhemError::MathOverflow)};
 let Some(scaled)=reference.checked_mul(tolerance_bps as u128) else {return Err(MayhemError::MathOverflow)};
 let delta=scaled/10_000;
 let Some(low)=reference.checked_sub(delta) else {return Err(MayhemError::MathOverflow)};
 let Some(high)=reference.checked_add(delta) else {return Err(MayhemError::MathOverflow)};
 if price<low||price>high{return Err(MayhemError::PumpCpiSlippage);}
 Ok(())
 })();price_check?;
 let vt=c.virtual_token_reserves as u128;let (input,output)=crate::state::buy_amounts(vt,c,crate::state::QuoteCap{global_params:&a.global_params,quote_mint:a.quote_mint.key()},a.mayhem_base_token_vault.amount as u128,retained_reserve_bps);
 let native=a.quote_mint.key()==WSOL;
 let before_sol=if native{a.sol_vault.lamports()}else{a.mayhem_quote_token_vault.amount};let before_tokens=a.mayhem_base_token_vault.amount;

 crate::pump_v2::buy_v2_recovered(CpiContext::new_with_signer(a.pump_program.key(),crate::pump_v2::BuyV2Accounts{global:inline_cpi_info(a.global.as_ref()),mint:inline_cpi_info(a.mint.as_ref().as_ref()),quote_mint:inline_cpi_info(a.quote_mint.as_ref()),token_program:inline_cpi_info(a.token_program.as_ref()),quote_token_program:inline_cpi_info(a.quote_token_program.as_ref()),associated_token_program:inline_cpi_info(a.associated_token_program.as_ref()),fee_recipient:inline_cpi_info(a.fee_recipient.as_ref()),fee_recipient_ata:inline_cpi_info(a.fee_recipient_ata.as_ref()),buyback_recipient:inline_cpi_info(a.buyback_recipient.as_ref()),buyback_recipient_ata:inline_cpi_info(a.buyback_recipient_ata.as_ref()),bonding_curve:inline_cpi_info(a.bonding_curve.as_ref().as_ref()),associated_bonding_curve:inline_cpi_info(a.associated_base_bonding_curve.as_ref().as_ref()),associated_quote_bonding_curve:inline_cpi_info(a.associated_quote_bonding_curve.as_ref()),user:inline_cpi_info(a.sol_vault.as_ref()),associated_user:inline_cpi_info(a.mayhem_base_token_vault.as_ref().as_ref()),associated_quote_user:inline_cpi_info(a.mayhem_quote_token_vault.as_ref().as_ref()),creator_vault:inline_cpi_info(a.creator_vault.as_ref()),associated_creator_vault:inline_cpi_info(a.associated_creator_vault.as_ref()),sharing_config:inline_cpi_info(a.sharing_config.as_ref()),global_volume_accumulator:inline_cpi_info(a.global_volume_accumulator.as_ref()),user_volume_accumulator:inline_cpi_info(a.user_volume_accumulator.as_ref()),associated_user_volume_accumulator:inline_cpi_info(a.associated_user_volume_accumulator.as_ref()),fee_config:inline_cpi_info(a.fee_config.as_ref()),fee_program:inline_cpi_info(a.fee_program.as_ref()),system_program:inline_cpi_info(a.system_program.as_ref()),event_authority:inline_cpi_info(a.pump_event_authority.as_ref()),program:inline_cpi_info(a.pump_program.as_ref())},&[&[b"sol-vault",&[ctx.bumps.sol_vault]]]),input,output,false)
  .map_err(|_| {
     error!(BuyError::PumpCpiFailed)
  })?;
 a.mayhem_base_token_vault.reload()?;a.bonding_curve.reload()?;if !native{a.mayhem_quote_token_vault.reload()?;}
 let after_quote=if native{a.sol_vault.lamports()}else{a.mayhem_quote_token_vault.amount};













    let sol=            before_sol-after_quote;
    let tokens=          a.mayhem_base_token_vault.amount-before_tokens;
 crate::state::record_amm_buy(&mut a.global_params,&mut a.mayhem_state,sol,tokens,&a.quote_mint.key())?;
 let quote_vault_balance;let state_quote_deployed;let virtual_quote;let global_quote_deployed;let real_quote;let sol_amount;let sol_vault_balance;let state_sol_deployed;let virtual_sol;let global_sol_deployed;let real_sol;if native{sol_amount=sol;sol_vault_balance=a.sol_vault.lamports();state_sol_deployed=a.mayhem_state.field_56;virtual_sol=a.bonding_curve.virtual_sol_reserves;global_sol_deployed=a.global_params.sol_deployed;real_sol=a.bonding_curve.real_sol_reserves;quote_vault_balance=sol_vault_balance;state_quote_deployed=state_sol_deployed;virtual_quote=virtual_sol;global_quote_deployed=global_sol_deployed;real_quote=real_sol;}else{sol_amount=0;sol_vault_balance=0;state_sol_deployed=0;virtual_sol=0;global_sol_deployed=0;real_sol=0;quote_vault_balance=a.mayhem_quote_token_vault.amount;state_quote_deployed=a.mayhem_state.field_56;virtual_quote=a.bonding_curve.virtual_sol_reserves;global_quote_deployed=a.global_params.stable_deployed;real_quote=a.bonding_curve.real_sol_reserves;}let quote_mint=if native{Pubkey::default()}else{a.quote_mint.key()};
 let event=SellEvent{side:false,quote_vault_balance,state_quote_deployed,global_sol_deployed,virtual_quote,real_quote,global_quote_deployed,sol_amount,sol_vault_balance,state_sol_deployed,virtual_sol,real_sol,sol_vault:a.sol_vault.key(),mint:a.mint.key(),token_amount:tokens,token_vault_balance:a.mayhem_base_token_vault.amount,state_token_net:a.mayhem_state.field_72,end_timestamp:a.mayhem_state.end_timestamp,virtual_tokens:a.bonding_curve.virtual_token_reserves,timestamp:Clock::get()?.unix_timestamp,real_tokens:a.bonding_curve.real_token_reserves,version:a.global_params.version,retained_reserve_bps,quote_amount:sol,quote_mint};
 emit!(event);emit_cpi!(event);Ok(())
}
















































































































































pub fn sell_v2_handler(ctx:Context<SellV2>,reference:u128,tolerance_bps:u16,retained_reserve_bps:u16)->Result<()> {
 let a=&mut *ctx.accounts;
 a.mayhem_state.validate_trade(&a.global_params)?;





 a.mayhem_state.validate_quote(a.bonding_curve.real_sol_reserves,&a.global_params,&a.quote_mint.key())?;




 let c=&a.bonding_curve;

 let price_check:std::result::Result<(),MayhemError> = (|| {
 let vt=c.virtual_token_reserves as u128;let vs=c.virtual_sol_reserves as u128;let Some(price)=(vs*1_000_000_000_000_000u128).checked_div(vt) else {return Err(MayhemError::MathOverflow)};
 let Some(scaled)=reference.checked_mul(tolerance_bps as u128) else {return Err(MayhemError::MathOverflow)};
 let delta=scaled/10_000;
 let Some(low)=reference.checked_sub(delta) else {return Err(MayhemError::MathOverflow)};
 let Some(high)=reference.checked_add(delta) else {return Err(MayhemError::MathOverflow)};
 if price<low||price>high{return Err(MayhemError::PumpCpiSlippage);}
 Ok(())
 })();price_check?;

 let quote_mint=a.quote_mint.key();let (input,minimum)=crate::state::sell_amounts(c.virtual_token_reserves,c,crate::state::QuoteCap{global_params:&a.global_params,quote_mint},a.mayhem_base_token_vault.amount,retained_reserve_bps);
 let native=quote_mint==WSOL;
 let before_sol=if native{a.sol_vault.lamports()}else{a.mayhem_quote_token_vault.amount};let before_tokens=a.mayhem_base_token_vault.amount;

 
 crate::pump_v2::cpi::sell_v2(CpiContext::new_with_signer(a.pump_program.key(),crate::pump_v2::cpi::accounts::SellV2{global:inline_cpi_info(a.global.as_ref()),mint:inline_cpi_info(a.mint.as_ref().as_ref()),quote_mint:inline_cpi_info(a.quote_mint.as_ref()),token_program:inline_cpi_info(a.token_program.as_ref()),quote_token_program:inline_cpi_info(a.quote_token_program.as_ref()),associated_token_program:inline_cpi_info(a.associated_token_program.as_ref()),fee_recipient:inline_cpi_info(a.fee_recipient.as_ref()),fee_recipient_ata:inline_cpi_info(a.fee_recipient_ata.as_ref()),buyback_recipient:inline_cpi_info(a.buyback_recipient.as_ref()),buyback_recipient_ata:inline_cpi_info(a.buyback_recipient_ata.as_ref()),bonding_curve:inline_cpi_info(a.bonding_curve.as_ref().as_ref()),associated_bonding_curve:inline_cpi_info(a.associated_base_bonding_curve.as_ref().as_ref()),associated_quote_bonding_curve:inline_cpi_info(a.associated_quote_bonding_curve.as_ref()),user:inline_cpi_info(a.sol_vault.as_ref()),associated_user:inline_cpi_info(a.mayhem_base_token_vault.as_ref().as_ref()),associated_quote_user:inline_cpi_info(a.mayhem_quote_token_vault.as_ref().as_ref()),creator_vault:inline_cpi_info(a.creator_vault.as_ref()),associated_creator_vault:inline_cpi_info(a.associated_creator_vault.as_ref()),sharing_config:inline_cpi_info(a.sharing_config.as_ref()),user_volume_accumulator:inline_cpi_info(a.user_volume_accumulator.as_ref()),associated_user_volume_accumulator:inline_cpi_info(a.associated_user_volume_accumulator.as_ref()),fee_config:inline_cpi_info(a.fee_config.as_ref()),fee_program:inline_cpi_info(a.fee_program.as_ref()),system_program:inline_cpi_info(a.system_program.as_ref()),event_authority:inline_cpi_info(a.pump_event_authority.as_ref()),program:inline_cpi_info(a.pump_program.as_ref())},&[&[b"sol-vault",&[ctx.bumps.sol_vault]]]),input,minimum).map_err(|_|error!(BuyError::PumpCpiFailed))?;
 



 a.mayhem_base_token_vault.reload()?;a.bonding_curve.reload()?;if !native{a.mayhem_quote_token_vault.reload()?;}
 let after_quote=if native{a.sol_vault.lamports()}else{a.mayhem_quote_token_vault.amount};










    let sol=            after_quote-before_sol;
    let tokens=          before_tokens-a.mayhem_base_token_vault.amount;
 crate::state::record_sell(&mut a.global_params,&mut a.mayhem_state,sol,tokens,&a.quote_mint.key())?;
 let quote_vault_balance;let state_quote_deployed;let virtual_quote;let global_quote_deployed;let real_quote;let sol_amount;let sol_vault_balance;let state_sol_deployed;let virtual_sol;let global_sol_deployed;let real_sol;if native{sol_amount=sol;sol_vault_balance=a.sol_vault.lamports();state_sol_deployed=a.mayhem_state.field_56;virtual_sol=a.bonding_curve.virtual_sol_reserves;global_sol_deployed=a.global_params.sol_deployed;real_sol=a.bonding_curve.real_sol_reserves;quote_vault_balance=sol_vault_balance;state_quote_deployed=state_sol_deployed;virtual_quote=virtual_sol;global_quote_deployed=global_sol_deployed;real_quote=real_sol;}else{sol_amount=0;sol_vault_balance=0;state_sol_deployed=0;virtual_sol=0;global_sol_deployed=0;real_sol=0;quote_vault_balance=a.mayhem_quote_token_vault.amount;state_quote_deployed=a.mayhem_state.field_56;virtual_quote=a.bonding_curve.virtual_sol_reserves;global_quote_deployed=a.global_params.stable_deployed;real_quote=a.bonding_curve.real_sol_reserves;}
 let quote_mint=if native{Pubkey::default()}else{a.quote_mint.key()};
 let event=SellEvent{side:true,sol_vault:a.sol_vault.key(),mint:a.mint.key(),sol_amount,token_amount:tokens,sol_vault_balance,token_vault_balance:a.mayhem_base_token_vault.amount,state_sol_deployed,state_token_net:a.mayhem_state.field_72,end_timestamp:a.mayhem_state.end_timestamp,virtual_sol,virtual_tokens:a.bonding_curve.virtual_token_reserves,timestamp:Clock::get()?.unix_timestamp,global_sol_deployed,real_sol,real_tokens:a.bonding_curve.real_token_reserves,version:a.global_params.version,retained_reserve_bps,quote_amount:sol,quote_vault_balance,state_quote_deployed,virtual_quote,global_quote_deployed,real_quote,quote_mint};
 emit!(event);emit_cpi!(event);Ok(())
}
#[inline(always)]
fn inline_cpi_info<'info>(info:&AccountInfo<'info>)->AccountInfo<'info>{AccountInfo{key:info.key,lamports:std::rc::Rc::clone(&info.lamports),data:std::rc::Rc::clone(&info.data),owner:info.owner,_unused:info._unused,is_signer:info.is_signer,is_writable:info.is_writable,executable:info.executable}}
