use anchor_lang::prelude::*;
use crate::donation::ConfigStatus;
use crate::types::Shareholder;
use crate::{FeeError, pump_global};
use crate::sharing::{Pump, Amm, SharingPool};
use crate::donation::{BondingCurve, PUMP, AMM};
use crate::ID;





























#[account]
pub struct SharingConfig {pub bump:u8,pub version:u8,pub status:ConfigStatus,pub mint:Pubkey,pub admin:Pubkey,pub admin_revoked:bool,pub shareholders:Vec<Shareholder>}















#[derive(Accounts)]
pub struct CreateFeeSharingConfig<'info>{
 /// CHECK: event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
 #[account(mut)]pub payer:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=PUMP)]pub global:Account<'info,pump_global::Global>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(init_if_needed,payer=payer,space=1024,seeds=[b"sharing-config",mint.key().as_ref()],bump)]pub sharing_config:Account<'info,SharingConfig>,
 pub system_program:Program<'info,System>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP)]pub bonding_curve:Account<'info,BondingCurve>,
 pub pump_program:Program<'info,Pump>,
 /// CHECK: external event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=PUMP)]pub pump_event_authority:AccountInfo<'info>,
 /// CHECK: optional pool validated in handler.
 #[account(mut)]pub pool:Option<Account<'info,SharingPool>>,
 pub pump_amm_program:Option<Program<'info,Amm>>,
 /// CHECK: external event PDA.
 #[account(seeds=[b"__event_authority"],bump,seeds::program=AMM)]pub pump_amm_event_authority:Option<AccountInfo<'info>>,
}













































#[event]
pub struct CreateFeeSharingConfigEvent{pub timestamp:i64,pub mint:Pubkey,pub bonding_curve:Pubkey,pub pool:Option<Pubkey>,pub sharing_config:Pubkey,pub admin:Pubkey,pub initial_shareholders:Vec<Shareholder>,pub status:ConfigStatus}











































































































































pub fn create_sharing(ctx:Context<CreateFeeSharingConfig>)->Result<()> {
  if ctx.accounts.sharing_config.version!=0{return Ok(());} let mint_info:&AccountInfo=ctx.accounts.mint.as_ref();
  let graduated=ctx.accounts.bonding_curve.real_sol_reserves==0
   && ctx.accounts.bonding_curve.virtual_sol_reserves==0
   && ctx.accounts.bonding_curve.real_token_reserves==0
   && ctx.accounts.bonding_curve.virtual_token_reserves==0;  let payer_info:&AccountInfo=ctx.accounts.payer.as_ref(); let payer_key=payer_info.key; let is_admin=payer_key.to_owned()==ctx.accounts.global.admin_set_creator_authority;
  let accounts=&*ctx.accounts; (move || -> Result<()> { if graduated {
   let pool=accounts.pool.as_ref().ok_or(FeeError::AmmAccountsRequiredForGraduatedCoin)?;
   let state=pool;




   require!(state.base_mint==mint_info.key(),FeeError::InvalidPool);
   let authority=Pubkey::find_program_address(&[b"pool-authority",mint_info.key().as_ref()],&crate::pump::ID).0;





   require!(state.creator==authority,FeeError::InvalidPool);












   let expected=Pubkey::create_program_address(&[b"pool",&state.index.to_le_bytes(),state.creator.as_ref(),state.base_mint.as_ref(),state.quote_mint.as_ref(),&[state.pool_bump]],&crate::pump_amm::ID).map_err(|_|FeeError::InvalidPool)?;



   require!(pool.key()==expected,FeeError::InvalidPool);




   require!(is_admin || payer_key.to_owned()==state.coin_creator,FeeError::NotAuthorized);
   
  }else{


   require!(is_admin || payer_key.to_owned()==accounts.bonding_curve.creator,FeeError::NotAuthorized);
   
  }; Ok(()) })()?;
  let state=&mut ctx.accounts.sharing_config;state.bump=ctx.bumps.sharing_config;state.version=2;state.status=ConfigStatus::Active;state.mint=ctx.accounts.mint.key();state.admin=ctx.accounts.bonding_curve.creator;state.admin_revoked=false;state.shareholders=vec![Shareholder{address:ctx.accounts.bonding_curve.creator,share_bps:10000}];state.exit(&ID)?;
  crate::pump::cpi::extend_account(CpiContext::new(
      ctx.accounts.pump_program.to_account_info(),
      crate::pump::cpi::accounts::ExtendAccount {
          account: ctx.accounts.bonding_curve.to_account_info(),
          user: ctx.accounts.payer.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          event_authority: ctx.accounts.pump_event_authority.to_account_info(),
          program: ctx.accounts.pump_program.to_account_info(),
      },
  ))?;
  let migration_ctx_1=CpiContext::new(ctx.accounts.pump_program.to_account_info(),crate::pump::cpi::accounts::MigrateBondingCurveCreator {
          mint: ctx.accounts.mint.to_account_info(),
          bonding_curve: ctx.accounts.bonding_curve.to_account_info(),
          sharing_config: state.to_account_info(),
          event_authority: ctx.accounts.pump_event_authority.to_account_info(),
          program: ctx.accounts.pump_program.to_account_info(),
      });crate::pump::cpi::migrate_bonding_curve_creator(migration_ctx_1)?;



  if graduated {
   let pool=ctx.accounts.pool.as_ref().ok_or(FeeError::AmmAccountsRequiredForGraduatedCoin)?;
   state.admin=pool.coin_creator;state.shareholders=vec![Shareholder{address:pool.coin_creator,share_bps:10000}];state.exit(&ID)?;
   let program=ctx.accounts.pump_amm_program.as_ref().ok_or(FeeError::AmmAccountsRequiredForGraduatedCoin)?;
   let event=ctx.accounts.pump_amm_event_authority.as_ref().ok_or(FeeError::AmmAccountsRequiredForGraduatedCoin)?;
   crate::pump_amm::cpi::extend_account(CpiContext::new(
       program.to_account_info(),
       crate::pump_amm::cpi::accounts::ExtendAccount {
           account: pool.to_account_info(),
           user: ctx.accounts.payer.to_account_info(),
           system_program: ctx.accounts.system_program.to_account_info(),
           event_authority: event.to_account_info(),
           program: program.to_account_info(),
       },
   ))?;
   let migration_ctx_3=CpiContext::new(program.to_account_info(),crate::pump_amm::cpi::accounts::MigratePoolCoinCreator {
           pool: pool.to_account_info(),
           program: program.to_account_info(),
           sharing_config: state.to_account_info(),
           event_authority: event.to_account_info(),
       });crate::pump_amm::cpi::migrate_pool_coin_creator(migration_ctx_3)?;



  }
  let event=CreateFeeSharingConfigEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),bonding_curve:ctx.accounts.bonding_curve.key(),pool:ctx.accounts.pool.as_ref().map(|x|x.key()),sharing_config:state.key(),admin:state.admin,initial_shareholders:state.shareholders.clone(),status:state.status.clone()};emit!(event);emit_cpi!(event);Ok(())
 }
