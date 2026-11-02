//! Test-only migration probe for the pinned Raydium AMM v4 program.
//!
//! This is not a Beam instruction, is not deployed, and does not enable
//! graduation. `solana-program-test` loads [`process_raydium`], which is
//! `raydium_amm::processor::Processor::process` from
//! raydium-io/raydium-amm@91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6, and this
//! probe cross-program invokes it with the 19-account `Initialize2` layout.
//! SPL Token and the associated-token program are the BPF programs bundled
//! with solana-program-test 3.0.0. That is local banks execution of the pinned
//! source, not a mock pool and not a claim that a deployed Raydium binary
//! matches this commit.

use raydium_amm::processor::{
    config_feature, get_associated_address_and_bump_seed, AMM_ASSOCIATED_SEED, AMM_CONFIG_SEED,
    AUTHORITY_AMM, COIN_VAULT_ASSOCIATED_SEED, LP_MINT_ASSOCIATED_SEED, PC_VAULT_ASSOCIATED_SEED,
    TARGET_ASSOCIATED_SEED,
};
use raydium_amm::state::{AmmConfig, AmmInfo, AmmStatus, TargetOrders};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

pub const SYSTEM_PROGRAM_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111111");

pub const SOURCE_REPOSITORY: &str = "raydium-io/raydium-amm";
pub const SOURCE_COMMIT: &str = "91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6";
pub const INITIALIZE2_ACCOUNTS: usize = 19;

pub const TAG_MIGRATE: u8 = 1;
pub const TAG_MIGRATE_THEN_ABORT: u8 = 2;
/// CPI only. Used to show the pinned program rejects a Token-2022 mint.
pub const TAG_FORWARD: u8 = 3;

pub const ERR_BAD_ACCOUNTS: u32 = 6101;
pub const ERR_CURVE_BALANCE: u32 = 6102;
pub const ERR_POOL_IDENTITY: u32 = 6103;
pub const ERR_RESERVE: u32 = 6104;
pub const ERR_FEE: u32 = 6105;
pub const ERR_LIABILITY: u32 = 6106;
pub const ERR_DONATION: u32 = 6107;
pub const ERR_ROLLBACK: u32 = 6108;
pub const ERR_LAUNCH: u32 = 6109;
pub const ERR_DATA: u32 = 6110;
pub const ERR_PRECREATED: u32 = 6111;

pub const CURVE_LAUNCH_PREFIX: &[u8] = b"beam-curve-launch-v1";
const MIGRATION_SEED: &[u8] = b"beam-amm-v4-migration";
const DATA_LEN: usize = 49;

pub const ID: Pubkey = Pubkey::new_from_array([
    0xBE, 0xA0, 0x04, 0x04, 0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22, 0x33, 0x33, 0x33, 0x33,
    0x44, 0x44, 0x44, 0x44, 0x55, 0x55, 0x55, 0x55, 0x66, 0x66, 0x66, 0x66, 0x77, 0x77, 0x77, 0x77,
]);

pub fn id() -> Pubkey {
    ID
}

pub fn process_raydium(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    raydium_amm::processor::Processor::process(program_id, accounts, input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MigrateInput {
    pub tag: u8,
    pub init_coin: u64,
    pub init_pc: u64,
    pub coin_retained: u64,
    pub pc_liability: u64,
    pub pc_donation: u64,
    pub open_time: u64,
}

impl MigrateInput {
    pub fn encode(self) -> [u8; DATA_LEN] {
        let mut data = [0u8; DATA_LEN];
        data[0] = self.tag;
        data[1..9].copy_from_slice(&self.init_coin.to_le_bytes());
        data[9..17].copy_from_slice(&self.init_pc.to_le_bytes());
        data[17..25].copy_from_slice(&self.coin_retained.to_le_bytes());
        data[25..33].copy_from_slice(&self.pc_liability.to_le_bytes());
        data[33..41].copy_from_slice(&self.pc_donation.to_le_bytes());
        data[41..49].copy_from_slice(&self.open_time.to_le_bytes());
        data
    }

    fn decode(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != DATA_LEN {
            return Err(ProgramError::Custom(ERR_DATA));
        }
        let word = |start: usize| u64::from_le_bytes(input[start..start + 8].try_into().unwrap());
        let tag = input[0];
        if tag != TAG_MIGRATE && tag != TAG_MIGRATE_THEN_ABORT && tag != TAG_FORWARD {
            return Err(ProgramError::Custom(ERR_DATA));
        }
        Ok(Self {
            tag,
            init_coin: word(1),
            init_pc: word(9),
            coin_retained: word(17),
            pc_liability: word(25),
            pc_donation: word(33),
            open_time: word(41),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivedPool {
    pub authority: Pubkey,
    pub bump: u8,
    pub amm: Pubkey,
    pub amm_authority: Pubkey,
    pub nonce: u8,
    pub lp_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub target_orders: Pubkey,
    pub config: Pubkey,
    pub fee_destination: Pubkey,
    pub openbook_program: Pubkey,
    pub user_lp: Pubkey,
}

pub fn derive_pool(market: &Pubkey) -> DerivedPool {
    let program_id = raydium_amm::id();
    let (authority, bump) = Pubkey::find_program_address(&[MIGRATION_SEED, market.as_ref()], &ID);
    let (amm_authority, nonce) = Pubkey::find_program_address(&[AUTHORITY_AMM], &program_id);
    let associated =
        |seed| get_associated_address_and_bump_seed(&program_id, market, seed, &program_id).0;
    let lp_mint = associated(LP_MINT_ASSOCIATED_SEED);
    DerivedPool {
        authority,
        bump,
        amm: associated(AMM_ASSOCIATED_SEED),
        amm_authority,
        nonce,
        lp_mint,
        coin_vault: associated(COIN_VAULT_ASSOCIATED_SEED),
        pc_vault: associated(PC_VAULT_ASSOCIATED_SEED),
        target_orders: associated(TARGET_ASSOCIATED_SEED),
        config: Pubkey::find_program_address(&[AMM_CONFIG_SEED], &program_id).0,
        fee_destination: config_feature::create_pool_fee_address::id(),
        openbook_program: config_feature::openbook_program::id(),
        user_lp: spl_associated_token_account::get_associated_token_address(&authority, &lp_mint),
    }
}

pub fn pinned_config_bytes(
    pnl_owner: &Pubkey,
    cancel_owner: &Pubkey,
    create_pool_fee: u64,
) -> Vec<u8> {
    let mut config = AmmConfig::default();
    unsafe {
        std::ptr::addr_of_mut!(config.pnl_owner).write_unaligned(*pnl_owner);
        std::ptr::addr_of_mut!(config.cancel_owner).write_unaligned(*cancel_owner);
        std::ptr::addr_of_mut!(config.create_pool_fee).write_unaligned(create_pool_fee);
    }
    let mut out = vec![0u8; std::mem::size_of::<AmmConfig>()];
    unsafe {
        std::ptr::copy_nonoverlapping(
            &config as *const AmmConfig as *const u8,
            out.as_mut_ptr(),
            out.len(),
        );
    }
    out
}

pub fn expected_lp(
    init_coin: u64,
    init_pc: u64,
    coin_decimals: u8,
) -> Result<(u64, u64), ProgramError> {
    let liquidity = u128::from(init_coin)
        .checked_mul(u128::from(init_pc))
        .ok_or(ProgramError::Custom(ERR_RESERVE))?
        .isqrt();
    let liquidity = u64::try_from(liquidity).map_err(|_| ProgramError::Custom(ERR_RESERVE))?;
    let locked = 10u64
        .checked_pow(u32::from(coin_decimals))
        .ok_or(ProgramError::Custom(ERR_RESERVE))?;
    let user_lp = liquidity
        .checked_sub(locked)
        .ok_or(ProgramError::Custom(ERR_RESERVE))?;
    Ok((liquidity, user_lp))
}

struct Accounts<'a, 'info> {
    authority: &'a AccountInfo<'info>,
    curve_coin: &'a AccountInfo<'info>,
    curve_pc: &'a AccountInfo<'info>,
    curve_launch: &'a AccountInfo<'info>,
    user_coin: &'a AccountInfo<'info>,
    user_pc: &'a AccountInfo<'info>,
    user_lp: &'a AccountInfo<'info>,
    amm: &'a AccountInfo<'info>,
    amm_authority: &'a AccountInfo<'info>,
    lp_mint: &'a AccountInfo<'info>,
    coin_mint: &'a AccountInfo<'info>,
    pc_mint: &'a AccountInfo<'info>,
    coin_vault: &'a AccountInfo<'info>,
    pc_vault: &'a AccountInfo<'info>,
    target_orders: &'a AccountInfo<'info>,
    config: &'a AccountInfo<'info>,
    fee_destination: &'a AccountInfo<'info>,
    market: &'a AccountInfo<'info>,
    token_program: &'a AccountInfo<'info>,
    ata_program: &'a AccountInfo<'info>,
    system_program: &'a AccountInfo<'info>,
    rent: &'a AccountInfo<'info>,
    raydium: &'a AccountInfo<'info>,
}

fn accounts<'a, 'info>(
    input: &'a [AccountInfo<'info>],
) -> Result<Accounts<'a, 'info>, ProgramError> {
    if input.len() != 23 {
        return Err(ProgramError::Custom(ERR_BAD_ACCOUNTS));
    }
    Ok(Accounts {
        authority: &input[0],
        curve_coin: &input[1],
        curve_pc: &input[2],
        curve_launch: &input[3],
        user_coin: &input[4],
        user_pc: &input[5],
        user_lp: &input[6],
        amm: &input[7],
        amm_authority: &input[8],
        lp_mint: &input[9],
        coin_mint: &input[10],
        pc_mint: &input[11],
        coin_vault: &input[12],
        pc_vault: &input[13],
        target_orders: &input[14],
        config: &input[15],
        fee_destination: &input[16],
        market: &input[17],
        token_program: &input[18],
        ata_program: &input[19],
        system_program: &input[20],
        rent: &input[21],
        raydium: &input[22],
    })
}

fn require_key(info: &AccountInfo, expected: &Pubkey) -> Result<(), ProgramError> {
    if info.key != expected {
        Err(ProgramError::Custom(ERR_BAD_ACCOUNTS))
    } else {
        Ok(())
    }
}

fn token_amount(info: &AccountInfo) -> Result<u64, ProgramError> {
    Ok(spl_token::state::Account::unpack(&info.data.borrow())?.amount)
}

fn uncreated(info: &AccountInfo) -> Result<(), ProgramError> {
    if info.lamports() == 0 && info.data_is_empty() && info.owner == &SYSTEM_PROGRAM_ID {
        Ok(())
    } else {
        Err(ProgramError::Custom(ERR_PRECREATED))
    }
}

fn read_packed<T: Copy>(value: *const T) -> T {
    unsafe { value.read_unaligned() }
}

pub fn process_instruction(
    program_id: &Pubkey,
    account_list: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::Custom(ERR_BAD_ACCOUNTS));
    }
    let args = MigrateInput::decode(input)?;
    let a = accounts(account_list)?;
    let launch_before = a.curve_launch.data.borrow().to_vec();
    if !launch_before.starts_with(CURVE_LAUNCH_PREFIX) {
        return Err(ProgramError::Custom(ERR_LAUNCH));
    }
    let derived = derive_pool(a.market.key);
    require_key(a.authority, &derived.authority)?;
    require_key(a.amm, &derived.amm)?;
    require_key(a.amm_authority, &derived.amm_authority)?;
    require_key(a.lp_mint, &derived.lp_mint)?;
    require_key(a.coin_vault, &derived.coin_vault)?;
    require_key(a.pc_vault, &derived.pc_vault)?;
    require_key(a.target_orders, &derived.target_orders)?;
    require_key(a.config, &derived.config)?;
    require_key(a.fee_destination, &derived.fee_destination)?;
    require_key(a.user_lp, &derived.user_lp)?;
    require_key(a.raydium, &raydium_amm::id())?;
    require_key(a.token_program, &spl_token::id())?;
    require_key(a.ata_program, &spl_associated_token_account::id())?;
    require_key(a.system_program, &SYSTEM_PROGRAM_ID)?;
    require_key(a.rent, &solana_program::sysvar::rent::ID)?;
    if a.coin_mint.key == a.pc_mint.key {
        return Err(ProgramError::Custom(ERR_BAD_ACCOUNTS));
    }
    let openbook = derived.openbook_program;
    for key in [
        *a.token_program.key,
        *a.ata_program.key,
        *a.system_program.key,
        *a.rent.key,
        *a.amm.key,
        *a.amm_authority.key,
        *a.lp_mint.key,
        *a.coin_mint.key,
        *a.pc_mint.key,
        *a.coin_vault.key,
        *a.pc_vault.key,
        *a.target_orders.key,
        *a.config.key,
        *a.fee_destination.key,
        *a.market.key,
        *a.authority.key,
        *a.user_coin.key,
        *a.user_pc.key,
        *a.user_lp.key,
    ] {
        if key == openbook {
            return Err(ProgramError::Custom(ERR_BAD_ACCOUNTS));
        }
    }

    let create_pool_fee = {
        let config = AmmConfig::load_checked(a.config, &raydium_amm::id())?;
        read_packed(std::ptr::addr_of!(config.create_pool_fee))
    };
    let fee_lamports_before = a.fee_destination.lamports();
    let fee_amount_before = token_amount(a.fee_destination)?;
    let user_coin_before = token_amount(a.user_coin)?;
    let user_pc_before = token_amount(a.user_pc)?;

    if args.tag != TAG_FORWARD {
        for info in [
            a.amm,
            a.lp_mint,
            a.coin_vault,
            a.pc_vault,
            a.target_orders,
            a.user_lp,
        ] {
            uncreated(info)?;
        }
        let curve_coin = token_amount(a.curve_coin)?;
        let curve_pc = token_amount(a.curve_pc)?;
        let expected_coin = args
            .init_coin
            .checked_add(args.coin_retained)
            .ok_or(ProgramError::Custom(ERR_CURVE_BALANCE))?;
        let expected_pc = args
            .init_pc
            .checked_add(args.pc_liability)
            .and_then(|sum| sum.checked_add(args.pc_donation))
            .ok_or(ProgramError::Custom(ERR_CURVE_BALANCE))?;
        if curve_coin != expected_coin || curve_pc != expected_pc {
            return Err(ProgramError::Custom(ERR_CURVE_BALANCE));
        }
        let seeds: &[&[u8]] = &[MIGRATION_SEED, a.market.key.as_ref(), &[derived.bump]];
        invoke_signed(
            &spl_token::instruction::transfer(
                a.token_program.key,
                a.curve_coin.key,
                a.user_coin.key,
                a.authority.key,
                &[],
                args.init_coin,
            )?,
            &[
                a.curve_coin.clone(),
                a.user_coin.clone(),
                a.authority.clone(),
                a.token_program.clone(),
            ],
            &[seeds],
        )?;
        invoke_signed(
            &spl_token::instruction::transfer(
                a.token_program.key,
                a.curve_pc.key,
                a.user_pc.key,
                a.authority.key,
                &[],
                args.init_pc,
            )?,
            &[
                a.curve_pc.clone(),
                a.user_pc.clone(),
                a.authority.clone(),
                a.token_program.clone(),
            ],
            &[seeds],
        )?;
    }

    let mut data = vec![1u8, derived.nonce];
    data.extend_from_slice(&args.open_time.to_le_bytes());
    data.extend_from_slice(&args.init_pc.to_le_bytes());
    data.extend_from_slice(&args.init_coin.to_le_bytes());
    if data.len() != 26 {
        return Err(ProgramError::Custom(ERR_DATA));
    }
    let metas = vec![
        solana_program::instruction::AccountMeta::new_readonly(*a.token_program.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.ata_program.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.system_program.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.rent.key, false),
        solana_program::instruction::AccountMeta::new(*a.amm.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.amm_authority.key, false),
        solana_program::instruction::AccountMeta::new(*a.lp_mint.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.coin_mint.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.pc_mint.key, false),
        solana_program::instruction::AccountMeta::new(*a.coin_vault.key, false),
        solana_program::instruction::AccountMeta::new(*a.pc_vault.key, false),
        solana_program::instruction::AccountMeta::new(*a.target_orders.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.config.key, false),
        solana_program::instruction::AccountMeta::new(*a.fee_destination.key, false),
        solana_program::instruction::AccountMeta::new_readonly(*a.market.key, false),
        solana_program::instruction::AccountMeta::new(*a.authority.key, true),
        solana_program::instruction::AccountMeta::new(*a.user_coin.key, false),
        solana_program::instruction::AccountMeta::new(*a.user_pc.key, false),
        solana_program::instruction::AccountMeta::new(*a.user_lp.key, false),
    ];
    if metas.len() != INITIALIZE2_ACCOUNTS {
        return Err(ProgramError::Custom(ERR_BAD_ACCOUNTS));
    }
    let seeds: &[&[u8]] = &[MIGRATION_SEED, a.market.key.as_ref(), &[derived.bump]];
    invoke_signed(
        &solana_program::instruction::Instruction {
            program_id: *a.raydium.key,
            accounts: metas,
            data,
        },
        &[
            a.token_program.clone(),
            a.ata_program.clone(),
            a.system_program.clone(),
            a.rent.clone(),
            a.amm.clone(),
            a.amm_authority.clone(),
            a.lp_mint.clone(),
            a.coin_mint.clone(),
            a.pc_mint.clone(),
            a.coin_vault.clone(),
            a.pc_vault.clone(),
            a.target_orders.clone(),
            a.config.clone(),
            a.fee_destination.clone(),
            a.market.clone(),
            a.authority.clone(),
            a.user_coin.clone(),
            a.user_pc.clone(),
            a.user_lp.clone(),
            a.raydium.clone(),
        ],
        &[seeds],
    )?;

    if args.tag == TAG_FORWARD {
        if a.curve_launch.data.borrow().as_ref() != launch_before.as_slice() {
            return Err(ProgramError::Custom(ERR_LAUNCH));
        }
        return Ok(());
    }

    let (liquidity, user_lp) = {
        let mint = spl_token::state::Mint::unpack(&a.coin_mint.data.borrow())?;
        expected_lp(args.init_coin, args.init_pc, mint.decimals)?
    };
    let identity = {
        let amm = AmmInfo::load_mut_checked(a.amm, &raydium_amm::id())?;
        (
            read_packed(std::ptr::addr_of!(amm.status)),
            read_packed(std::ptr::addr_of!(amm.nonce)),
            read_packed(std::ptr::addr_of!(amm.coin_vault)),
            read_packed(std::ptr::addr_of!(amm.pc_vault)),
            read_packed(std::ptr::addr_of!(amm.coin_vault_mint)),
            read_packed(std::ptr::addr_of!(amm.pc_vault_mint)),
            read_packed(std::ptr::addr_of!(amm.lp_mint)),
            read_packed(std::ptr::addr_of!(amm.market)),
            read_packed(std::ptr::addr_of!(amm.market_program)),
            read_packed(std::ptr::addr_of!(amm.target_orders)),
            read_packed(std::ptr::addr_of!(amm.amm_owner)),
            read_packed(std::ptr::addr_of!(amm.lp_amount)),
        )
    };
    if a.amm.owner != &raydium_amm::id()
        || a.amm.data_len() != std::mem::size_of::<AmmInfo>()
        || a.target_orders.owner != &raydium_amm::id()
        || a.target_orders.data_len() != std::mem::size_of::<TargetOrders>()
        || identity.0 != AmmStatus::SwapOnly.into_u64()
        || identity.1 != u64::from(derived.nonce)
        || identity.2 != *a.coin_vault.key
        || identity.3 != *a.pc_vault.key
        || identity.4 != *a.coin_mint.key
        || identity.5 != *a.pc_mint.key
        || identity.6 != *a.lp_mint.key
        || identity.7 != *a.market.key
        || identity.8 != Pubkey::default()
        || identity.9 != *a.target_orders.key
        || identity.10 != config_feature::amm_owner::id()
        || identity.11 != liquidity
    {
        return Err(ProgramError::Custom(ERR_POOL_IDENTITY));
    }
    if token_amount(a.coin_vault)? != args.init_coin || token_amount(a.pc_vault)? != args.init_pc {
        return Err(ProgramError::Custom(ERR_RESERVE));
    }
    let lp_mint = spl_token::state::Mint::unpack(&a.lp_mint.data.borrow())?;
    let user_lp_account = spl_token::state::Account::unpack(&a.user_lp.data.borrow())?;
    if lp_mint.supply != user_lp
        || lp_mint.decimals != spl_token::state::Mint::unpack(&a.coin_mint.data.borrow())?.decimals
        || lp_mint.mint_authority != COption::Some(*a.amm_authority.key)
        || lp_mint.freeze_authority != COption::None
        || user_lp_account.amount != user_lp
        || user_lp_account.mint != *a.lp_mint.key
        || user_lp_account.owner != *a.authority.key
    {
        return Err(ProgramError::Custom(ERR_RESERVE));
    }
    let fee_lamports_after = fee_lamports_before
        .checked_add(create_pool_fee)
        .ok_or(ProgramError::Custom(ERR_FEE))?;
    let fee_amount_after = fee_amount_before
        .checked_add(create_pool_fee)
        .ok_or(ProgramError::Custom(ERR_FEE))?;
    if a.fee_destination.lamports() != fee_lamports_after
        || token_amount(a.fee_destination)? != fee_amount_after
    {
        return Err(ProgramError::Custom(ERR_FEE));
    }
    if token_amount(a.curve_pc)? != args.pc_liability.saturating_add(args.pc_donation)
        || token_amount(a.curve_pc)? < args.pc_liability
    {
        return Err(ProgramError::Custom(ERR_LIABILITY));
    }
    if token_amount(a.curve_coin)? != args.coin_retained
        || token_amount(a.user_coin)? != user_coin_before
        || token_amount(a.user_pc)? != user_pc_before
    {
        return Err(ProgramError::Custom(ERR_DONATION));
    }
    if a.curve_launch.data.borrow().as_ref() != launch_before.as_slice() {
        return Err(ProgramError::Custom(ERR_LAUNCH));
    }
    if args.tag == TAG_MIGRATE_THEN_ABORT {
        return Err(ProgramError::Custom(ERR_ROLLBACK));
    }
    Ok(())
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn pinned_ids_and_layout_match_the_source_commit() {
        assert_eq!(
            raydium_amm::id().to_string(),
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        );
        assert_eq!(
            config_feature::amm_owner::id().to_string(),
            "GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ"
        );
        assert_eq!(
            config_feature::create_pool_fee_address::id().to_string(),
            "7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5"
        );
        assert_eq!(
            config_feature::openbook_program::id().to_string(),
            "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX"
        );
        assert_eq!(std::mem::size_of::<AmmConfig>(), 544);
        assert_eq!(std::mem::size_of::<AmmInfo>(), 752);
        assert_eq!(std::mem::size_of::<TargetOrders>(), 2208);
        let bytes = pinned_config_bytes(
            &Pubkey::new_from_array([3; 32]),
            &Pubkey::new_from_array([4; 32]),
            25_000_000,
        );
        assert_eq!(bytes.len(), 544);
        assert_eq!(
            u64::from_le_bytes(bytes[536..544].try_into().unwrap()),
            25_000_000
        );
        assert_eq!(&bytes[..32], &[3; 32]);
    }

    #[test]
    fn nineteen_account_plan_excludes_openbook() {
        let market = Pubkey::new_from_array([9; 32]);
        let pool = derive_pool(&market);
        assert_ne!(pool.openbook_program, pool.amm);
        assert_ne!(pool.openbook_program, pool.config);
        assert_ne!(pool.openbook_program, pool.fee_destination);
        assert_ne!(pool.openbook_program, market);
        assert_eq!(
            pool.user_lp,
            spl_associated_token_account::get_associated_token_address(
                &pool.authority,
                &pool.lp_mint
            )
        );
        let (liquidity, user_lp) = expected_lp(1_000_000_000_000, 1_000_000_000_000, 6).unwrap();
        assert_eq!(liquidity, 1_000_000_000_000);
        assert_eq!(user_lp, 999_999_000_000);
    }
}
