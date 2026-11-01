//! Legacy sell recovered from the original dispatch, parsers and arithmetic.
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint,TokenAccount};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::{MayhemState,MayhemStatus,PUMP_ID};
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};

#[derive(Accounts)]
pub struct LegacySell<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 /// CHECK: Original unchecked PDA and mutable constraints.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]pub mayhem_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 pub token_program:Program<'info,Token2022>,
 pub system_program:Program<'info,System>,
 /// CHECK: Passed through to Pump; original unchecked account.
 pub global:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP_ID)]pub bonding_curve:Box<Account<'info,SellCurve>>,
 /// CHECK: Pump validates fee recipient.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=bonding_curve,associated_token::token_program=mint.to_account_info().owner)]pub associated_bonding_curve:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Pump validates creator vault.
 #[account(mut)]pub creator_vault:UncheckedAccount<'info>,
 /// CHECK: Pump event authority passed through.
 pub pump_event_authority:UncheckedAccount<'info>,
 /// CHECK: Original address constraint, without executable Program wrapper.
 #[account(address=PUMP_ID)]pub pump_program:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee config.
 pub fee_config:UncheckedAccount<'info>,
 /// CHECK: Pump validates its fee program.
 pub fee_program:UncheckedAccount<'info>,
 /// CHECK: Original runtime event PDA constraint.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::MayhemRecovered>,
 /// CHECK: Original derives this under Mayhem, not the Pump program.
 #[account(seeds=[b"bonding-curve-v2",mint.key().as_ref()],bump)]pub bonding_curve_v2:UncheckedAccount<'info>,
 /// CHECK: Pump validates the recipient when used.
 #[account(mut)]pub buyback_recipient:UncheckedAccount<'info>,
}
/// The sell path uses the padded external-account decoder at 0xac0, reached
/// from 0x5258 -> 0xa248 -> 0x66a0 and by reload at 0x5fb0. It pads short
/// input to 141 zero bytes before the ordinary decoder at 0x84108. This is
/// intentionally separate from the older, strict 82-byte burn account codec.
#[derive(Clone)]
pub struct SellCurve(pub curve_accounts::BondingCurve);
impl std::ops::Deref for SellCurve {type Target=curve_accounts::BondingCurve;fn deref(&self)->&Self::Target{&self.0}}
impl std::ops::DerefMut for SellCurve {fn deref_mut(&mut self)->&mut Self::Target{&mut self.0}}
impl Owner for SellCurve {fn owner()->Pubkey {curve_accounts::PUMP_ID}}
impl AccountSerialize for SellCurve {fn try_serialize<W:std::io::Write>(&self,writer:&mut W)->Result<()>{self.0.try_serialize(writer)}}











fn math()->Error {MayhemError::MathOverflow.into()}
pub use super::pump_interface::legacy_sell_handler as handler;
pub use super::pump_interface::SellEvent;








// Keep the published module path while using the shared error namespace.
pub use crate::MayhemError as SellError;


