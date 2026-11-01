pub use volume_accumulator::{GlobalVolumeAccumulator, UserVolumeAccumulator};
use anchor_lang::Discriminator;
use anchor_lang::prelude::*;
#[event]
#[derive(Clone)]
pub struct DisableEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub disable_create_pool: bool,
    pub disable_deposit: bool,
    pub disable_withdraw: bool,
    pub disable_buy: bool,
    pub disable_sell: bool,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub lp_fee_basis_points: u64,
    pub protocol_fee_basis_points: u64,
    pub disable_flags: u8,
    pub protocol_fee_recipients: [Pubkey; 8],
    pub coin_creator_fee_basis_points: u64,
    pub admin_set_coin_creator_authority: Pubkey,
    pub whitelist_pda: Pubkey,
    pub reserved_fee_recipient: Pubkey,
    pub mayhem_mode_enabled: bool,
    pub reserved_fee_recipients: [Pubkey; 7],
    pub is_cashback_enabled: bool,
    pub buyback_fee_recipients: [Pubkey; 8],
    pub buyback_basis_points: u64,
    pub boost_authority: Pubkey,
    pub boost_enabled: bool,
    pub creator_fee_configurable: bool,
    pub max_configurable_creator_fee_bps: u64,
}
#[event]
#[derive(Clone)]
pub struct UpdateAdminEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub new_admin: Pubkey,
}

impl Owner for GlobalConfig { fn owner() -> Pubkey { crate::ID } }
impl Discriminator for GlobalConfig { const DISCRIMINATOR: &'static [u8] = &[149, 8, 156, 202, 160, 252, 176, 217]; }
impl AccountDeserialize for GlobalConfig {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 { return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into()); }
        let data = *buf;
        *buf = &data[8..];
        if &data[..8] != Self::DISCRIMINATOR { return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into()); }
        if buf.len() < 899 { return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into()); }
        if buf.len() < 932 { return read_global_config_899(buf); }
        if buf.len() < 941 { return read_global_config_932(buf); }
        read_global_config_941(buf)
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 { return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into()); }
        *buf = &buf[8..];
        if buf.len() < 899 { return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into()); }
        if buf.len() < 932 { return read_global_config_899(buf); }
        if buf.len() < 941 { return read_global_config_932(buf); }
        read_global_config_941(buf)
    }
}

fn read_global_config_899(buf: &mut &[u8]) -> Result<GlobalConfig> {
    Ok(GlobalConfig {
        admin: Pubkey::deserialize(buf)?,
        lp_fee_basis_points: u64::deserialize(buf)?,
        protocol_fee_basis_points: u64::deserialize(buf)?,
        disable_flags: u8::deserialize(buf)?,
        protocol_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        coin_creator_fee_basis_points: u64::deserialize(buf)?,
        admin_set_coin_creator_authority: Pubkey::deserialize(buf)?,
        whitelist_pda: Pubkey::deserialize(buf)?,
        reserved_fee_recipient: Pubkey::deserialize(buf)?,
        mayhem_mode_enabled: bool::deserialize(buf)?,
        reserved_fee_recipients: <[Pubkey; 7]>::deserialize(buf)?,
        is_cashback_enabled: bool::deserialize(buf)?,
        buyback_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        buyback_basis_points: u64::deserialize(buf)?,
        boost_authority: Pubkey::default(),
        boost_enabled: false,
        creator_fee_configurable: false,
        max_configurable_creator_fee_bps: 0,
    })
}

fn read_global_config_932(buf: &mut &[u8]) -> Result<GlobalConfig> {
    Ok(GlobalConfig {
        admin: Pubkey::deserialize(buf)?,
        lp_fee_basis_points: u64::deserialize(buf)?,
        protocol_fee_basis_points: u64::deserialize(buf)?,
        disable_flags: u8::deserialize(buf)?,
        protocol_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        coin_creator_fee_basis_points: u64::deserialize(buf)?,
        admin_set_coin_creator_authority: Pubkey::deserialize(buf)?,
        whitelist_pda: Pubkey::deserialize(buf)?,
        reserved_fee_recipient: Pubkey::deserialize(buf)?,
        mayhem_mode_enabled: bool::deserialize(buf)?,
        reserved_fee_recipients: <[Pubkey; 7]>::deserialize(buf)?,
        is_cashback_enabled: bool::deserialize(buf)?,
        buyback_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        buyback_basis_points: u64::deserialize(buf)?,
        boost_authority: Pubkey::deserialize(buf)?,
        boost_enabled: bool::deserialize(buf)?,
        creator_fee_configurable: false,
        max_configurable_creator_fee_bps: 0,
    })
}

fn read_global_config_941(buf: &mut &[u8]) -> Result<GlobalConfig> {
    Ok(GlobalConfig {
        admin: Pubkey::deserialize(buf)?,
        lp_fee_basis_points: u64::deserialize(buf)?,
        protocol_fee_basis_points: u64::deserialize(buf)?,
        disable_flags: u8::deserialize(buf)?,
        protocol_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        coin_creator_fee_basis_points: u64::deserialize(buf)?,
        admin_set_coin_creator_authority: Pubkey::deserialize(buf)?,
        whitelist_pda: Pubkey::deserialize(buf)?,
        reserved_fee_recipient: Pubkey::deserialize(buf)?,
        mayhem_mode_enabled: bool::deserialize(buf)?,
        reserved_fee_recipients: <[Pubkey; 7]>::deserialize(buf)?,
        is_cashback_enabled: bool::deserialize(buf)?,
        buyback_fee_recipients: <[Pubkey; 8]>::deserialize(buf)?,
        buyback_basis_points: u64::deserialize(buf)?,
        boost_authority: Pubkey::deserialize(buf)?,
        boost_enabled: bool::deserialize(buf)?,
        creator_fee_configurable: bool::deserialize(buf)?,
        max_configurable_creator_fee_bps: u64::deserialize(buf)?,
    })
}

impl AccountSerialize for GlobalConfig {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(Self::DISCRIMINATOR)?;
        self.serialize(writer)?;
        Ok(())
    }
}
mod global_config;

#[event]
#[derive(Clone)]
pub struct SetBoostAuthorityEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub old_boost_authority: Pubkey,
    pub new_boost_authority: Pubkey,
}

#[event]
#[derive(Clone)]
pub struct UpdateCreatorFeeConfigEvent {
    pub timestamp: i64,
    pub admin: Pubkey,
    pub creator_fee_configurable: bool,
    pub max_configurable_creator_fee_bps: u64,
}

#[event]
#[derive(Clone)]
pub struct ReservedFeeRecipientsEvent {
    pub timestamp: i64,
    pub reserved_fee_recipient: Pubkey,
    pub reserved_fee_recipients: [Pubkey; 7],
}

#[event]
#[derive(Clone)]
pub struct CloseUserVolumeAccumulatorEvent {
    pub user: Pubkey,
    pub timestamp: i64,
    pub total_unclaimed_tokens: u64,
    pub total_claimed_tokens: u64,
    pub current_sol_volume: u64,
    pub last_update_timestamp: i64,
}
#[event]
#[derive(Clone)]
pub struct InitUserVolumeAccumulatorEvent {
    pub payer: Pubkey,
    pub user: Pubkey,
    pub timestamp: i64,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Pool {
    pub pool_bump: u8,
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
    pub virtual_quote_reserves: i128,
    pub creator_fee_bps: u64,
    pub can_edit_creator_fee: bool,
    pub is_holder_reward: bool,
    pub protocol_fees: u64,
    pub creator_fees: u64,
}

impl Owner for Pool { fn owner() -> Pubkey { crate::ID } }
impl Discriminator for Pool { const DISCRIMINATOR: &'static [u8] = &[241,154,109,4,17,177,109,188]; }
impl AccountDeserialize for Pool {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        let data = *buf;
        *buf = &data[8..];
        if &data[..8] != Self::DISCRIMINATOR {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        if buf.len() < 203 {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        if buf.len() < 235 { read_pool_203(buf) }
        else if buf.len() < 236 { read_pool_235(buf) }
        else if buf.len() < 237 { read_pool_236(buf) }
        else if buf.len() < 253 { read_pool_237(buf) }
        else if buf.len() < 262 { read_pool_253(buf) }
        else if buf.len() < 263 { read_pool_262(buf) }
        else if buf.len() < 279 { read_pool_263(buf) }
        else { read_pool_279(buf) }
    }
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < 8 {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        *buf = &buf[8..];
        if buf.len() < 203 {
            return Err(anchor_lang::solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        if buf.len() < 235 { read_pool_203(buf) }
        else if buf.len() < 236 { read_pool_235(buf) }
        else if buf.len() < 237 { read_pool_236(buf) }
        else if buf.len() < 253 { read_pool_237(buf) }
        else if buf.len() < 262 { read_pool_253(buf) }
        else if buf.len() < 263 { read_pool_262(buf) }
        else if buf.len() < 279 { read_pool_263(buf) }
        else { read_pool_279(buf) }
    }
}

fn read_pool_203(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::default(),
        is_mayhem_mode: false,
        is_cashback_coin: false,
        virtual_quote_reserves: 0,
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_235(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: false,
        is_cashback_coin: false,
        virtual_quote_reserves: 0,
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_236(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: false,
        virtual_quote_reserves: 0,
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_237(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: bool::deserialize(buf)?,
        virtual_quote_reserves: 0,
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_253(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: bool::deserialize(buf)?,
        virtual_quote_reserves: i128::deserialize(buf)?,
        creator_fee_bps: 0,
        can_edit_creator_fee: false,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_262(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: bool::deserialize(buf)?,
        virtual_quote_reserves: i128::deserialize(buf)?,
        creator_fee_bps: u64::deserialize(buf)?,
        can_edit_creator_fee: bool::deserialize(buf)?,
        is_holder_reward: false,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_263(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: bool::deserialize(buf)?,
        virtual_quote_reserves: i128::deserialize(buf)?,
        creator_fee_bps: u64::deserialize(buf)?,
        can_edit_creator_fee: bool::deserialize(buf)?,
        is_holder_reward: bool::deserialize(buf)?,
        protocol_fees: 0,
        creator_fees: 0,
    })
}

fn read_pool_279(buf: &mut &[u8]) -> Result<Pool> {
    Ok(Pool {
        pool_bump: u8::deserialize(buf)?,
        index: u16::deserialize(buf)?,
        creator: Pubkey::deserialize(buf)?,
        base_mint: Pubkey::deserialize(buf)?,
        quote_mint: Pubkey::deserialize(buf)?,
        lp_mint: Pubkey::deserialize(buf)?,
        pool_base_token_account: Pubkey::deserialize(buf)?,
        pool_quote_token_account: Pubkey::deserialize(buf)?,
        lp_supply: u64::deserialize(buf)?,
        coin_creator: Pubkey::deserialize(buf)?,
        is_mayhem_mode: bool::deserialize(buf)?,
        is_cashback_coin: bool::deserialize(buf)?,
        virtual_quote_reserves: i128::deserialize(buf)?,
        creator_fee_bps: u64::deserialize(buf)?,
        can_edit_creator_fee: bool::deserialize(buf)?,
        is_holder_reward: bool::deserialize(buf)?,
        protocol_fees: u64::deserialize(buf)?,
        creator_fees: u64::deserialize(buf)?,
    })
}

impl AccountSerialize for Pool {
    fn try_serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(Self::DISCRIMINATOR)?;
        self.serialize(writer)?;
        Ok(())
    }
}
mod pool;
mod volume_accumulator;
