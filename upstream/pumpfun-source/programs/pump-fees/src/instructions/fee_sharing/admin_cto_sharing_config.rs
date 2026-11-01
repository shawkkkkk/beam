{
use crate::types::*;
use crate::donation::ConfigStatus;
use crate::FeeError;
















































#[event]pub struct ResetFeeSharingConfigEvent{pub timestamp:i64,pub mint:Pubkey,pub sharing_config:Pubkey,pub old_admin:Pubkey,pub old_shareholders:Vec<Shareholder>,pub new_admin:Pubkey,pub new_shareholders:Vec<Shareholder>,pub old_version:u8,pub new_version:u8}




























  let state=&mut ctx.accounts.sharing_config;
  let old_admin=state.admin;let old_version=state.version;let old_shareholders=state.shareholders.clone();
  match new_admin {
   Some(admin)=>{
    require_keys_neq!(admin, Pubkey::default(), FeeError::InvalidAdmin);
    state.status=ConfigStatus::Active;state.admin=admin;state.admin_revoked=false;state.shareholders=vec![Shareholder{address:admin,share_bps:10000}];}
   None=>{state.status=ConfigStatus::Paused;state.admin=Pubkey::default();state.admin_revoked=true;state.shareholders=vec![];}
  }
  state.version=2;
  let event=ResetFeeSharingConfigEvent{timestamp:Clock::get()?.unix_timestamp,mint:ctx.accounts.mint.key(),sharing_config:state.key(),old_admin,old_shareholders,new_admin:new_admin.unwrap_or_default(),new_shareholders:state.shareholders.clone(),old_version,new_version:state.version};
  emit!(event);emit_cpi!(event);Ok(())
 }
