use anchor_lang::prelude::*;
use crate::types::FeeProgramGlobal;
use crate::state::SocialFeePda;
use crate::FeeError;






#[derive(Accounts)]
#[instruction(user_id:String,platform:u8)]
pub struct CreateSocialFeePda<'info>{
 #[account(mut)]pub payer:Signer<'info>,
 #[account(init_if_needed,payer=payer,space=179,seeds=[b"social-fee-pda",user_id.as_bytes(),&[platform]],bump)]pub social_fee_pda:Account<'info,SocialFeePda>,
 pub system_program:Program<'info,System>,
 #[account(seeds=[b"fee-program-global"],bump=fee_program_global.bump)]pub fee_program_global:Account<'info,FeeProgramGlobal>,
 /// CHECK: event authority PDA validated.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:AccountInfo<'info>,
 /// CHECK: event-CPI program account; original imposes no typed-program constraint.
 pub program:AccountInfo<'info>,
}













#[event]
pub struct SocialFeePdaCreated{pub timestamp:i64,pub user_id:String,pub platform:u8,pub social_fee_pda:Pubkey,pub created_by:Pubkey}


























 pub fn create_social(ctx:Context<CreateSocialFeePda>,user_id:String,platform:u8)->Result<()> {
  if ctx.accounts.fee_program_global.disable_flags & 1 != 0 {msg!("CreateSocialFeePda is currently disabled");return Err(FeeError::FeatureDeactivated.into());}
  if ctx.accounts.social_fee_pda.version!=0{msg!("social_fee_pda already exists");return Ok(());}
  require!(user_id.len()<=20,FeeError::UserIdTooLong);
  let state=&mut ctx.accounts.social_fee_pda;
  state.bump=ctx.bumps.social_fee_pda;state.version=1;state.user_id=user_id.clone();state.platform=platform;
  state.total_claimed=0;state.last_claimed=0;state.total_stable_claimed=0;state._reserved=[0;120];
  let event=SocialFeePdaCreated{timestamp:Clock::get()?.unix_timestamp,user_id,platform,social_fee_pda:state.key(),created_by:ctx.accounts.payer.key()};
  emit!(event);emit_cpi!(event);Ok(())
 }
