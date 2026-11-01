//! BuyV2 recovered from wrapper 0x81a78, parser 0x5c6d8 and handler 0x224e0.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction,program::invoke_signed};
use anchor_spl::token_interface::{Mint,TokenAccount,TokenInterface};
use anchor_spl::token_2022::Token2022;
use crate::{GlobalParams,MayhemError};
use crate::state::{MayhemState,MayhemStatus,PUMP_ID};
use crate::checked_program::{CheckedSystemProgram,CheckedSystemProgramBumps,__client_accounts_checked_system_program,__cpi_client_accounts_checked_system_program};

use anchor_spl::associated_token::AssociatedToken;
use super::sell::{SellCurve,SellEvent,SellError};
use super::buy::BuyError;
const WSOL:Pubkey=pubkey!("So11111111111111111111111111111111111111112");
#[derive(Accounts)]
pub struct BuyV2<'info> {
 #[account(mut)]pub trader:Signer<'info>,
 #[account(mut,seeds=[b"global-params"],bump,has_one=trader)]pub global_params:Box<Account<'info,GlobalParams>>,
 #[account(mut,seeds=[b"mayhem-state",mint.key().as_ref()],bump)]pub mayhem_state:Box<Account<'info,MayhemState>>,
 pub mint:Box<InterfaceAccount<'info,Mint>>,
 /// CHECK: Original unchecked quote account; ATA owner and Pump CPI constrain its use.
 pub quote_mint:UncheckedAccount<'info>,
 pub associated_token_program:Program<'info,AssociatedToken>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=sol_vault,associated_token::token_program=mint.to_account_info().owner)]pub mayhem_base_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 #[account(mut,associated_token::mint=quote_mint,associated_token::authority=sol_vault,associated_token::token_program=quote_mint.to_account_info().owner)]pub mayhem_quote_token_vault:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Original writable PDA.
 #[account(mut,seeds=[b"sol-vault"],bump)]pub sol_vault:UncheckedAccount<'info>,
 pub system_program:Program<'info,System>,
 pub token_program:Program<'info,Token2022>,
 pub quote_token_program:Interface<'info,TokenInterface>,
 /// CHECK: Passed through to Pump.
 pub global:UncheckedAccount<'info>,
 #[account(mut,seeds=[b"bonding-curve",mint.key().as_ref()],bump,seeds::program=PUMP_ID)]pub bonding_curve:Box<Account<'info,SellCurve>>,
 /// CHECK: Pump validates quote ATA.
 #[account(mut)]pub associated_quote_bonding_curve:UncheckedAccount<'info>,
 /// CHECK: Pump validates recipient membership.
 #[account(mut)]pub fee_recipient:UncheckedAccount<'info>,
 /// CHECK: Pump validates recipient ATA.
 #[account(mut)]pub fee_recipient_ata:UncheckedAccount<'info>,
 #[account(mut,associated_token::mint=mint,associated_token::authority=bonding_curve,associated_token::token_program=mint.to_account_info().owner)]pub associated_base_bonding_curve:Box<InterfaceAccount<'info,TokenAccount>>,
 /// CHECK: Pump validates creator PDA.
 #[account(mut)]pub creator_vault:UncheckedAccount<'info>,
 /// CHECK: Pump validates creator ATA.
 #[account(mut)]pub associated_creator_vault:UncheckedAccount<'info>,
 /// CHECK: Pump validates sharing config.
 pub sharing_config:UncheckedAccount<'info>,
 /// CHECK: Pump event authority.
 pub pump_event_authority:UncheckedAccount<'info>,
 /// CHECK: Original fixed address.
 #[account(address=PUMP_ID)]pub pump_program:UncheckedAccount<'info>,
 /// CHECK: Original unchecked read-only global volume account passed through to Pump.
 pub global_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates volume PDA.
 #[account(mut)]pub user_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates volume ATA.
 #[account(mut)]pub associated_user_volume_accumulator:UncheckedAccount<'info>,
 /// CHECK: Pump validates fee config.
 pub fee_config:UncheckedAccount<'info>,
 /// CHECK: Pump validates fee program.
 pub fee_program:UncheckedAccount<'info>,
 /// CHECK: Original event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 pub program:Program<'info,crate::program::MayhemRecovered>,
 /// CHECK: Pump validates buyback recipient.
 #[account(mut)]pub buyback_recipient:UncheckedAccount<'info>,
 /// CHECK: Pump validates buyback ATA.
 #[account(mut)]pub buyback_recipient_ata:UncheckedAccount<'info>,
}




















































