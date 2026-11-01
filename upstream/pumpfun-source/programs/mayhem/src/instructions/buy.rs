//! Legacy buy recovered from original dispatch 0x813f8, parser 0x525f8,
//! handler 0x1bc08 and signed-counter/risk helper 0x2c598.
//! Fixture validation is synthetic and ordinary-signed; no historical prestate claim.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction,program::invoke_signed};
use anchor_spl::token_interface::{Mint,TokenAccount};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::{MayhemState,MayhemStatus,PUMP_ID};
use super::sell::{SellCurve,SellEvent,SellError};
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};

#[derive(Accounts)]
pub struct LegacyBuy<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]pub mayhem_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Original writable PDA constraint.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 pub token_program:Program<'info,Token2022>,
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
 /// CHECK: Pump validates global volume state.
 #[account(mut)]pub global_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates the sol-vault user volume account.
 #[account(mut)]pub user_volume_accumulator:UncheckedAccount<'info>,
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
#[inline(always)]
pub(super) fn clone_info<'a>(info: &AccountInfo<'a>) -> AccountInfo<'a> {
 info.clone()
}
fn math()->Error {MayhemError::MathOverflow.into()}
pub use super::pump_interface::legacy_buy_handler as handler;
// Keep the published module path while using the shared error namespace.
pub use crate::MayhemError as BuyError;
