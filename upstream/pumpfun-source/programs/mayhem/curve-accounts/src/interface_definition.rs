macro_rules! recovered_interfaces { () => {
use anchor_lang::prelude::*;






    #[account("account")] pub struct BondingCurve {pub virtual_token_reserves:u64,pub virtual_sol_reserves:u64, pub real_token_reserves:u64,pub real_sol_reserves:u64, pub token_total_supply:u64,pub complete:bool,pub creator:Pubkey, pub is_mayhem_mode:bool,pub is_cashback_coin:bool,pub quote_mint:Pubkey, pub creator_fee_bps:u64,pub can_edit_creator_fee:bool,pub is_holder_reward:bool, pub creator_fee:u64,pub protocol_fees:u64,} pub fn try_deserialize_curve(buf:&mut &[u8])->Result<BondingCurve>{if buf.len()<8{return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound.into());}if &buf[..8]!=BondingCurve::DISCRIMINATOR{return Err(annotate(error!(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch),"BondingCurve"));}BondingCurve::try_deserialize_unchecked(buf)}






impl Default for BondingCurve {fn default()->Self{Self{virtual_token_reserves:0,virtual_sol_reserves:0,real_token_reserves:0,real_sol_reserves:0,token_total_supply:0,complete:false,creator:Pubkey::new_from_array([0;32]),is_mayhem_mode:false,is_cashback_coin:false,quote_mint:Pubkey::new_from_array([0;32]),creator_fee_bps:0,can_edit_creator_fee:false,is_holder_reward:false,creator_fee:0,protocol_fees:0}}}
pub static PUMP_ID:Pubkey=pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub static AMM_ID:Pubkey=pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
impl Owner for BondingCurve {fn owner()->Pubkey{PUMP_ID}}
impl Owner for Pool {#[inline] fn owner()->Pubkey{AMM_ID}}
#[inline(always)] fn annotate(error:Error,name:&str)->Error{error.with_account_name(name)}

pub mod account_wrapper;
pub use account_wrapper::AccountWrapper;
pub trait CheckedDeserialize:AccountDeserialize {fn deserialize_checked(buf:&mut &[u8])->Result<Self>;}
impl CheckedDeserialize for Pool {#[inline] fn deserialize_checked(buf:&mut &[u8])->Result<Self>{try_deserialize_pool(buf)}}

pub mod cpi_interface {
use anchor_lang::prelude::*;
historical_anchor_program::declare_program!(pump);
pub use pump::*;
pub struct BuyV2Accounts<'info> {
 pub global:AccountInfo<'info>,
 pub mint:AccountInfo<'info>,
 pub quote_mint:AccountInfo<'info>,
 pub token_program:AccountInfo<'info>,
 pub quote_token_program:AccountInfo<'info>,
 pub associated_token_program:AccountInfo<'info>,
 pub fee_recipient:AccountInfo<'info>,
 pub fee_recipient_ata:AccountInfo<'info>,
 pub buyback_recipient:AccountInfo<'info>,
 pub buyback_recipient_ata:AccountInfo<'info>,
 pub bonding_curve:AccountInfo<'info>,
 pub associated_bonding_curve:AccountInfo<'info>,
 pub associated_quote_bonding_curve:AccountInfo<'info>,
 pub user:AccountInfo<'info>,
 pub associated_user:AccountInfo<'info>,
 pub associated_quote_user:AccountInfo<'info>,
 pub creator_vault:AccountInfo<'info>,
 pub associated_creator_vault:AccountInfo<'info>,
 pub sharing_config:AccountInfo<'info>,
 pub global_volume_accumulator:AccountInfo<'info>,
 pub user_volume_accumulator:AccountInfo<'info>,
 pub associated_user_volume_accumulator:AccountInfo<'info>,
 pub fee_config:AccountInfo<'info>,
 pub fee_program:AccountInfo<'info>,
 pub system_program:AccountInfo<'info>,
 pub event_authority:AccountInfo<'info>,
 pub program:AccountInfo<'info>,
}
impl<'info> ToAccountMetas for BuyV2Accounts<'info> {
 fn to_account_metas(&self, _is_signer:Option<bool>)->Vec<anchor_lang::solana_program::instruction::AccountMeta> {
 let mut account_metas=vec![];
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.global), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.mint), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.quote_mint), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.token_program), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.quote_token_program), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.associated_token_program), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.fee_recipient), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.fee_recipient_ata), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.buyback_recipient), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.buyback_recipient_ata), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.bonding_curve), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_bonding_curve), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_quote_bonding_curve), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.user), true));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_user), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_quote_user), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.creator_vault), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_creator_vault), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.sharing_config), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.global_volume_accumulator), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.user_volume_accumulator), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(anchor_lang::Key::key(&self.associated_user_volume_accumulator), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.fee_config), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.fee_program), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.system_program), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.event_authority), false));
 account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::Key::key(&self.program), false));
 account_metas
 }
}
impl<'info> ToAccountInfos<'info> for BuyV2Accounts<'info> {
 fn to_account_infos(&self)->Vec<AccountInfo<'info>> {
 let mut account_infos=vec![];
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.global));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.mint));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.quote_mint));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.token_program));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.quote_token_program));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_token_program));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.fee_recipient));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.fee_recipient_ata));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.buyback_recipient));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.buyback_recipient_ata));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.bonding_curve));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_bonding_curve));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_quote_bonding_curve));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.user));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_user));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_quote_user));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.creator_vault));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_creator_vault));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.sharing_config));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.global_volume_accumulator));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.user_volume_accumulator));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.associated_user_volume_accumulator));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.fee_config));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.fee_program));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.system_program));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.event_authority));
 account_infos.extend(anchor_lang::ToAccountInfos::to_account_infos(&self.program));
 account_infos
 }
}
struct BuyV2WireArgs { amount:u64, max_sol_cost:u64, track_volume:bool }
impl BuyV2WireArgs {
 #[inline(never)]
 fn serialize(&self,writer:&mut Vec<u8>) {
  writer.extend_from_slice(&self.amount.to_le_bytes());
  writer.extend_from_slice(&self.max_sol_cost.to_le_bytes());
  writer.extend_from_slice(&[self.track_volume as u8]);
 }
}
pub fn buy_v2_recovered<'a,'b,'c,'info>(ctx:CpiContext<'a,'b,'c,'info,BuyV2Accounts<'info>>,amount:u64,max_sol_cost:u64,track_volume:bool)->Result<()> {
 let ix={
  let args=BuyV2WireArgs {amount,max_sol_cost,track_volume};
  let mut data=Vec::with_capacity(256);
  data.extend_from_slice(&[184,23,238,97,103,197,211,61]);
  args.serialize(&mut data);
  let accounts=ctx.to_account_metas(None);
  anchor_lang::solana_program::instruction::Instruction {program_id:ctx.program_id.key(),accounts,data}
 };
 let acc_infos=ctx.to_account_infos();
 anchor_lang::solana_program::program::invoke_signed(&ix,&acc_infos,ctx.signer_seeds).map_or_else(|e|Err(Into::into(e)),|_|Ok(()))
}

}
}; }
macro_rules! recovered_pool { () => {
    #[account("account")] #[derive(Default)] pub struct Pool {pub pool_bump:u8, pub index:u16, pub creator:Pubkey, pub base_mint:Pubkey, pub quote_mint:Pubkey, pub lp_mint:Pubkey, pub pool_base_token_account:Pubkey, pub pool_quote_token_account:Pubkey, pub lp_supply:u64, pub coin_creator:Pubkey, pub is_mayhem_mode:bool, pub is_cashback_coin:bool, pub virtual_quote_reserves:i128, pub creator_fee_bps:u64, pub can_edit_creator_fee:bool, pub is_holder_reward:bool, pub protocol_fees:u64, pub creator_fees:u64,} pub fn try_deserialize_pool(buf:&mut &[u8])->Result<Pool>{if buf.len()<8{return Err(anchor_lang::error::ErrorCode::AccountDiscriminatorNotFound.into());}if &buf[..8]!=Pool::DISCRIMINATOR{return Err(annotate(error!(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch),"Pool"));}Pool::try_deserialize_unchecked(buf)}

}; }
