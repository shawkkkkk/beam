use crate::*;
use anchor_lang::Discriminator;
use anchor_spl::metadata::MetadataAccount;
pub const FEES_PROGRAM: Pubkey = anchor_lang::solana_program::pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub use crate::pump_fees::types::{ConfigStatus, Shareholder};
pub use crate::pump_fees::accounts::SharingConfig;
pub use crate::pump::accounts::BondingCurve;
pub use crate::instructions::coin_creator::migrate_pool_coin_creator::MigratePoolCoinCreator;
pub(crate) use crate::instructions::coin_creator::migrate_pool_coin_creator::{__client_accounts_migrate_pool_coin_creator, __cpi_client_accounts_migrate_pool_coin_creator};
pub use crate::instructions::coin_creator::migrate_pool_coin_creator::migrate;






pub use crate::instructions::coin_creator::set_coin_creator::SetCoinCreator;
pub(crate) use crate::instructions::coin_creator::set_coin_creator::{__client_accounts_set_coin_creator, __cpi_client_accounts_set_coin_creator};
pub use crate::instructions::coin_creator::set_coin_creator::set;












#[event]#[derive(Clone)]pub struct MigratePoolCoinCreatorEvent{pub timestamp:i64,pub base_mint:Pubkey,pub pool:Pubkey,pub sharing_config:Pubkey,pub old_coin_creator:Pubkey,pub new_coin_creator:Pubkey}
#[event]#[derive(Clone)]pub struct SetMetaplexCoinCreatorEvent{pub timestamp:i64,pub base_mint:Pubkey,pub pool:Pubkey,pub metadata:Pubkey,pub coin_creator:Pubkey}
#[event]#[derive(Clone)]pub struct SetBondingCurveCoinCreatorEvent{pub timestamp:i64,pub base_mint:Pubkey,pub pool:Pubkey,pub bonding_curve:Pubkey,pub coin_creator:Pubkey}

