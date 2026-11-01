//! Recovered sharing reset, tested through the genuine captured Pump admin_cto caller.
use anchor_lang::prelude::*;
use crate::{FeeError, ID, pump_global};
use crate::types::Shareholder;
use crate::donation::{ConfigStatus, SharingConfig};


#[derive(Accounts)]
pub struct AdminCtoSharingConfig<'info>{
 #[account(seeds=[b"pool-authority",mint.key().as_ref()],bump,seeds::program=pump_global::PUMP_ID)]pub pool_authority:Signer<'info>,
 pub authority:Signer<'info>,
 #[account(seeds=[b"global"],bump,seeds::program=pump_global::PUMP_ID,constraint=global.admin_set_creator_authority==authority.key() @ FeeError::NotAuthorized)]pub global:Box<Account<'info,pump_global::Global>>,
 pub mint:InterfaceAccount<'info,anchor_spl::token_interface::Mint>,
 #[account(mut,seeds=[b"sharing-config",mint.key().as_ref()],bump=sharing_config.bump,has_one=mint @ FeeError::InvalidSharingConfig)]pub sharing_config:Box<Account<'info,SharingConfig>>,
 /// CHECK: canonical event PDA.
 #[account(seeds=[b"__event_authority"],bump)]pub event_authority:UncheckedAccount<'info>,
 /// CHECK: fixed self program used for event CPI.
 pub program:Program<'info,crate::program::PumpFeesCandidate>,
}

