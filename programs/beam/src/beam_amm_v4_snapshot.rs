//! Disabled, source-pinned AMM v4 post-Initialize2 account inspection.
//!
//! Layout evidence: raydium-io/raydium-amm at
//! 91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6, `state.rs::AmmInfo`
//! and `processor.rs::process_initialize2`. This raw packed layout has no
//! Anchor discriminator. A verified account owner and length are necessary,
//! but this does not prove a deployed program binary matches that source.
//! No Beam instruction calls this helper or enables migration.

use anchor_lang::solana_program::{
    account_info::AccountInfo, program_option::COption, program_pack::Pack, pubkey,
};
use anchor_lang::{prelude::Pubkey, system_program};
use anchor_spl::token::spl_token::{
    native_mint,
    state::{Account as SplAccount, AccountState, Mint as SplMint},
};

use crate::beam_amm_v4::{
    derive_pool_addresses, expected_initial_user_lp, initialize2_instruction,
    validate_create_pool_fee_account, AmmV4AbiError, Initialize2Accounts, Initialize2Args, MintKey,
    DEVNET_PROGRAM_ID, MAINNET_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID,
};

pub const AMM_INFO_DATA_LEN: usize = 752;
pub const TARGET_ORDERS_DATA_LEN: usize = 2208;
const MAINNET_AMM_OWNER: Pubkey = pubkey!("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ");
const DEVNET_AMM_OWNER: Pubkey = pubkey!("DRayqG9RXYi8WHgWEmRQGrUWRWbhjYWYkCRJDd6JBBak");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Abi(AmmV4AbiError),
    InvalidProgram,
    InvalidAccountMetas,
    AliasedAccounts,
    InvalidPoolAccount,
    InvalidPoolData,
    InvalidPoolState,
    InvalidTargetOrders,
    InvalidMint,
    InvalidTokenAccount,
    InvalidNativeBacking,
    InvalidBalance,
    InvalidClock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmV4Snapshot {
    pub source_pin: &'static str,
    pub program_id: Pubkey,
    pub pool: Pubkey,
    pub market_seed: Pubkey,
    pub status_at_cpi: u64,
    pub open_time: u64,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub lp_destination: Pubkey,
    pub pool_lp_amount: u64,
    pub user_lp_amount: u64,
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64, SnapshotError> {
    let bytes: [u8; 8] = data
        .get(offset..offset + 8)
        .ok_or(SnapshotError::InvalidPoolData)?
        .try_into()
        .map_err(|_| SnapshotError::InvalidPoolData)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_key(data: &[u8], offset: usize) -> Result<Pubkey, SnapshotError> {
    let bytes: [u8; 32] = data
        .get(offset..offset + 32)
        .ok_or(SnapshotError::InvalidPoolData)?
        .try_into()
        .map_err(|_| SnapshotError::InvalidPoolData)?;
    Ok(Pubkey::new_from_array(bytes))
}

fn unpack_mint(info: &AccountInfo<'_>) -> Result<SplMint, SnapshotError> {
    if info.owner != &SPL_TOKEN_PROGRAM_ID {
        return Err(SnapshotError::InvalidMint);
    }
    SplMint::unpack(
        &info
            .try_borrow_data()
            .map_err(|_| SnapshotError::InvalidMint)?,
    )
    .map_err(|_| SnapshotError::InvalidMint)
}

fn unpack_token(info: &AccountInfo<'_>) -> Result<SplAccount, SnapshotError> {
    if info.owner != &SPL_TOKEN_PROGRAM_ID {
        return Err(SnapshotError::InvalidTokenAccount);
    }
    SplAccount::unpack(
        &info
            .try_borrow_data()
            .map_err(|_| SnapshotError::InvalidTokenAccount)?,
    )
    .map_err(|_| SnapshotError::InvalidTokenAccount)
}

fn check_token(
    info: &AccountInfo<'_>,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    native: bool,
) -> Result<(), SnapshotError> {
    let token = unpack_token(info)?;
    if token.state != AccountState::Initialized
        || token.mint != *mint
        || token.owner != *authority
        || token.amount != amount
        || token.delegate != COption::None
        || token.close_authority != COption::None
        || token.is_native.is_some() != native
    {
        return Err(SnapshotError::InvalidTokenAccount);
    }
    if native {
        let COption::Some(reserve) = token.is_native else {
            return Err(SnapshotError::InvalidNativeBacking);
        };
        if reserve.checked_add(token.amount) != Some(info.lamports()) {
            return Err(SnapshotError::InvalidNativeBacking);
        }
    }
    Ok(())
}

fn no_alias(infos: &[AccountInfo<'_>]) -> Result<(), SnapshotError> {
    // All recommended 19 keys have different roles. Do not allow a writable
    // value account to alias a program, mint, source, vault, fee recipient, or
    // market seed in a synthetic or eventual real CPI.
    for (index, info) in infos.iter().enumerate() {
        if infos[index + 1..].iter().any(|other| other.key == info.key) {
            return Err(SnapshotError::AliasedAccounts);
        }
    }
    Ok(())
}

/// Read actual AccountInfos immediately after a *planned* native-SOL,
/// classic-SPL Initialize2 CPI. `expected_market_seed` must later come from a
/// stored Beam policy/receipt, and `current_unix_timestamp` from Clock sysvar.
/// No trusted caller exists yet. This does not authenticate a transaction,
/// authorize LP custody, account for setup rent, or execute Raydium.
pub fn validate_post_initialize2(
    program: &AccountInfo<'_>,
    infos: &[AccountInfo<'_>],
    beam_mint: &Pubkey,
    expected_market_seed: &Pubkey,
    args: Initialize2Args,
    current_unix_timestamp: i64,
) -> Result<AmmV4Snapshot, SnapshotError> {
    if infos.len() != 19 || !program.executable {
        return Err(SnapshotError::InvalidProgram);
    }
    let amm_owner = if program.key == &MAINNET_PROGRAM_ID {
        MAINNET_AMM_OWNER
    } else if program.key == &DEVNET_PROGRAM_ID {
        DEVNET_AMM_OWNER
    } else {
        return Err(SnapshotError::InvalidProgram);
    };
    if current_unix_timestamp < 0 {
        return Err(SnapshotError::InvalidClock);
    }
    no_alias(infos)?;
    if infos[7].key != beam_mint
        || infos[8].key != &native_mint::id()
        || infos[14].key != expected_market_seed
    {
        return Err(SnapshotError::InvalidAccountMetas);
    }
    let (signer, _) =
        Pubkey::find_program_address(&[b"pool-authority", beam_mint.as_ref()], &crate::ID);
    if infos[15].key != &signer
        || infos[15].owner != &system_program::ID
        || !infos[15].data_is_empty()
    {
        return Err(SnapshotError::InvalidAccountMetas);
    }
    for index in [0, 1, 2] {
        if !infos[index].executable {
            return Err(SnapshotError::InvalidProgram);
        }
    }
    let coin_mint = unpack_mint(&infos[7])?;
    let pc_mint = unpack_mint(&infos[8])?;
    if coin_mint.decimals > crate::beam_amm_v4::MAX_SAFE_MINT_DECIMALS || pc_mint.decimals != 9 {
        return Err(SnapshotError::InvalidMint);
    }
    let p = derive_pool_addresses(program.key, expected_market_seed).map_err(SnapshotError::Abi)?;
    let accounts = Initialize2Accounts {
        coin_mint: MintKey {
            key: *infos[7].key,
            owner: *infos[7].owner,
        },
        pc_mint: MintKey {
            key: *infos[8].key,
            owner: *infos[8].owner,
        },
        market_seed: *expected_market_seed,
        user_wallet: *infos[15].key,
        user_coin_source: *infos[16].key,
        user_pc_source: *infos[17].key,
        user_lp_ata: *infos[18].key,
    };
    let expected =
        initialize2_instruction(*program.key, &accounts, args).map_err(SnapshotError::Abi)?;
    for (info, meta) in infos.iter().zip(expected.accounts.iter()) {
        if info.key != &meta.pubkey || (meta.is_writable && !info.is_writable) {
            return Err(SnapshotError::InvalidAccountMetas);
        }
    }
    validate_create_pool_fee_account(program.key, &infos[12], u64::MAX)
        .map_err(SnapshotError::Abi)?;

    let liquidity = u128::from(args.init_coin_amount)
        .checked_mul(u128::from(args.init_pc_amount))
        .ok_or(SnapshotError::InvalidBalance)?
        .isqrt();
    let pool_lp_amount: u64 = liquidity
        .try_into()
        .map_err(|_| SnapshotError::InvalidBalance)?;
    let user_lp_amount = expected_initial_user_lp(
        args.init_coin_amount,
        args.init_pc_amount,
        coin_mint.decimals,
    )
    .map_err(SnapshotError::Abi)?;
    let pool = &infos[4];
    if pool.owner != program.key || pool.data_len() != AMM_INFO_DATA_LEN {
        return Err(SnapshotError::InvalidPoolAccount);
    }
    let data = pool
        .try_borrow_data()
        .map_err(|_| SnapshotError::InvalidPoolData)?;
    let status = read_u64(&data, 0)?;
    let expected_status = if args.open_time > current_unix_timestamp as u64 {
        7
    } else {
        6
    };
    if status != expected_status
        || read_u64(&data, 8)? != u64::from(p.nonce)
        || read_u64(&data, 32)? != u64::from(coin_mint.decimals)
        || read_u64(&data, 40)? != u64::from(pc_mint.decimals)
        || read_u64(&data, 48)? != 1
        || read_u64(&data, 56)? != 0
        || read_u64(&data, 120)?
            != 10u64
                .checked_pow(u32::from(pc_mint.decimals.max(coin_mint.decimals)))
                .ok_or(SnapshotError::InvalidPoolState)?
        || read_u64(&data, 224)? != args.open_time
        || read_u64(&data, 720)? != pool_lp_amount
    {
        return Err(SnapshotError::InvalidPoolState);
    }
    for (offset, value) in [
        (128, 5),
        (136, 10_000),
        (144, 25),
        (152, 10_000),
        (160, 12),
        (168, 100),
        (176, 25),
        (184, 10_000),
    ] {
        if read_u64(&data, offset)? != value {
            return Err(SnapshotError::InvalidPoolState);
        }
    }
    for (offset, value) in [
        (336, p.coin_vault),
        (368, p.pc_vault),
        (400, *beam_mint),
        (432, native_mint::id()),
        (464, p.lp_mint),
        (496, Pubkey::default()),
        (528, *expected_market_seed),
        (560, Pubkey::default()),
        (592, p.target_orders),
        (688, amm_owner),
    ] {
        if read_key(&data, offset)? != value {
            return Err(SnapshotError::InvalidPoolState);
        }
    }
    drop(data);
    let target = &infos[11];
    if target.owner != program.key || target.data_len() != TARGET_ORDERS_DATA_LEN {
        return Err(SnapshotError::InvalidTargetOrders);
    }
    if read_key(
        &target
            .try_borrow_data()
            .map_err(|_| SnapshotError::InvalidTargetOrders)?,
        0,
    )? != p.amm
    {
        return Err(SnapshotError::InvalidTargetOrders);
    }
    check_token(&infos[16], beam_mint, &signer, 0, false)?;
    check_token(&infos[17], &native_mint::id(), &signer, 0, true)?;
    check_token(
        &infos[9],
        beam_mint,
        &p.authority,
        args.init_coin_amount,
        false,
    )?;
    check_token(
        &infos[10],
        &native_mint::id(),
        &p.authority,
        args.init_pc_amount,
        true,
    )?;
    let lp_mint = unpack_mint(&infos[6])?;
    if lp_mint.decimals != coin_mint.decimals
        || lp_mint.mint_authority != COption::Some(p.authority)
        || lp_mint.freeze_authority != COption::None
        || lp_mint.supply != user_lp_amount
    {
        return Err(SnapshotError::InvalidMint);
    }
    check_token(&infos[18], &p.lp_mint, &signer, user_lp_amount, false)?;
    let fee_destination = unpack_token(&infos[13])?;
    if fee_destination.state != AccountState::Initialized
        || fee_destination.mint != native_mint::id()
        || fee_destination.is_native == COption::None
    {
        return Err(SnapshotError::InvalidTokenAccount);
    }
    let COption::Some(fee_rent) = fee_destination.is_native else {
        return Err(SnapshotError::InvalidNativeBacking);
    };
    if fee_rent.checked_add(fee_destination.amount) != Some(infos[13].lamports()) {
        return Err(SnapshotError::InvalidNativeBacking);
    }
    Ok(AmmV4Snapshot {
        source_pin: "raydium-amm@91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6",
        program_id: *program.key,
        pool: p.amm,
        market_seed: *expected_market_seed,
        status_at_cpi: status,
        open_time: args.open_time,
        coin_mint: *beam_mint,
        pc_mint: native_mint::id(),
        coin_vault: p.coin_vault,
        pc_vault: p.pc_vault,
        lp_mint: p.lp_mint,
        lp_destination: *infos[18].key,
        pool_lp_amount,
        user_lp_amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beam_amm_v4::{user_lp_ata, AMM_CONFIG_DATA_LEN};

    fn info(
        key: Pubkey,
        owner: Pubkey,
        data: Vec<u8>,
        lamports: u64,
        writable: bool,
        executable: bool,
    ) -> AccountInfo<'static> {
        AccountInfo::new(
            Box::leak(Box::new(key)),
            false,
            writable,
            Box::leak(Box::new(lamports)),
            Box::leak(data.into_boxed_slice()),
            Box::leak(Box::new(owner)),
            executable,
            0,
        )
    }

    fn mint_data(decimals: u8, supply: u64, authority: COption<Pubkey>) -> Vec<u8> {
        let mut data = vec![0; SplMint::LEN];
        SplMint::pack(
            SplMint {
                mint_authority: authority,
                supply,
                decimals,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            &mut data,
        )
        .unwrap();
        data
    }

    fn token(mint: Pubkey, authority: Pubkey, amount: u64, native: bool) -> Vec<u8> {
        let mut data = vec![0; SplAccount::LEN];
        SplAccount::pack(
            SplAccount {
                mint,
                owner: authority,
                amount,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: if native {
                    COption::Some(1)
                } else {
                    COption::None
                },
                delegated_amount: 0,
                close_authority: COption::None,
            },
            &mut data,
        )
        .unwrap();
        data
    }

    fn put_u64(data: &mut [u8], offset: usize, value: u64) {
        data[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn put_key(data: &mut [u8], offset: usize, value: Pubkey) {
        data[offset..offset + 32].copy_from_slice(value.as_ref());
    }

    fn fixture(
        open_time: u64,
        clock: i64,
    ) -> (
        AccountInfo<'static>,
        Vec<AccountInfo<'static>>,
        Pubkey,
        Pubkey,
        Initialize2Args,
    ) {
        let program_id = DEVNET_PROGRAM_ID;
        let beam_mint = Pubkey::new_from_array([42; 32]);
        let market_seed = Pubkey::new_from_array([43; 32]);
        let signer =
            Pubkey::find_program_address(&[b"pool-authority", beam_mint.as_ref()], &crate::ID).0;
        let p = derive_pool_addresses(&program_id, &market_seed).unwrap();
        let args = Initialize2Args {
            open_time,
            init_coin_amount: 2_000_000,
            init_pc_amount: 2_000_000,
        };
        let accounts = Initialize2Accounts {
            coin_mint: MintKey {
                key: beam_mint,
                owner: SPL_TOKEN_PROGRAM_ID,
            },
            pc_mint: MintKey {
                key: native_mint::id(),
                owner: SPL_TOKEN_PROGRAM_ID,
            },
            market_seed,
            user_wallet: signer,
            user_coin_source: user_lp_ata(&signer, &beam_mint),
            user_pc_source: user_lp_ata(&signer, &native_mint::id()),
            user_lp_ata: user_lp_ata(&signer, &p.lp_mint),
        };
        let ix = initialize2_instruction(program_id, &accounts, args).unwrap();
        let mut pool = vec![0; AMM_INFO_DATA_LEN];
        for (offset, value) in [
            (0, if open_time > clock as u64 { 7 } else { 6 }),
            (8, u64::from(p.nonce)),
            (32, 6),
            (40, 9),
            (48, 1),
            (56, 0),
            (120, 1_000_000_000),
            (224, open_time),
            (720, 2_000_000),
        ] {
            put_u64(&mut pool, offset, value);
        }
        for (offset, value) in [
            (128, 5),
            (136, 10_000),
            (144, 25),
            (152, 10_000),
            (160, 12),
            (168, 100),
            (176, 25),
            (184, 10_000),
        ] {
            put_u64(&mut pool, offset, value);
        }
        for (offset, value) in [
            (336, p.coin_vault),
            (368, p.pc_vault),
            (400, beam_mint),
            (432, native_mint::id()),
            (464, p.lp_mint),
            (496, Pubkey::default()),
            (528, market_seed),
            (560, Pubkey::default()),
            (592, p.target_orders),
            (688, DEVNET_AMM_OWNER),
        ] {
            put_key(&mut pool, offset, value);
        }
        let mut target = vec![0; TARGET_ORDERS_DATA_LEN];
        put_key(&mut target, 0, p.amm);
        let infos = ix
            .accounts
            .iter()
            .enumerate()
            .map(|(index, meta)| {
                let owner = match index {
                    4 | 11 | 12 => program_id,
                    6 | 7 | 8 | 9 | 10 | 13 | 16 | 17 | 18 => SPL_TOKEN_PROGRAM_ID,
                    _ => system_program::ID,
                };
                let (data, lamports) = match index {
                    4 => (pool.clone(), 1),
                    6 => (mint_data(6, 1_000_000, COption::Some(p.authority)), 1),
                    7 => (mint_data(6, 1_000_000, COption::None), 1),
                    8 => (mint_data(9, 1_000_000, COption::None), 1),
                    9 => (token(beam_mint, p.authority, 2_000_000, false), 1),
                    10 => (
                        token(native_mint::id(), p.authority, 2_000_000, true),
                        2_000_001,
                    ),
                    11 => (target.clone(), 1),
                    12 => (vec![0; AMM_CONFIG_DATA_LEN], 1),
                    13 => (
                        token(native_mint::id(), Pubkey::new_from_array([51; 32]), 7, true),
                        8,
                    ),
                    15 => (Vec::new(), 1),
                    16 => (token(beam_mint, signer, 0, false), 1),
                    17 => (token(native_mint::id(), signer, 0, true), 1),
                    18 => (token(p.lp_mint, signer, 1_000_000, false), 1),
                    _ => (Vec::new(), 1),
                };
                info(
                    meta.pubkey,
                    owner,
                    data,
                    lamports,
                    meta.is_writable,
                    index <= 2,
                )
            })
            .collect();
        let program = info(program_id, system_program::ID, Vec::new(), 1, false, true);
        (program, infos, beam_mint, market_seed, args)
    }

    #[test]
    fn accepts_source_pinned_fresh_pool_and_waiting_status() {
        for (open_time, clock, expected_status) in [(0, 100, 6), (200, 100, 7)] {
            let (program, infos, mint, seed, args) = fixture(open_time, clock);
            let s = validate_post_initialize2(&program, &infos, &mint, &seed, args, clock).unwrap();
            assert_eq!(s.status_at_cpi, expected_status);
            assert_eq!((s.pool_lp_amount, s.user_lp_amount), (2_000_000, 1_000_000));
        }
    }

    #[test]
    fn rejects_pool_owner_length_status_identity_and_alias_substitutions() {
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        let original_data = infos[4].try_borrow_data().unwrap().to_vec();
        infos[4] = info(
            *infos[4].key,
            system_program::ID,
            original_data,
            1,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolAccount)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[4] = info(*infos[4].key, DEVNET_PROGRAM_ID, vec![0; 8], 1, true, false);
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolAccount)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[4].try_borrow_mut_data().unwrap()[0..8].copy_from_slice(&1u64.to_le_bytes());
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolState)
        );
        let (program, infos, mint, seed, args) = fixture(0, 100);
        infos[4].try_borrow_mut_data().unwrap()[0..8].copy_from_slice(b"ANCHOR00");
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolState)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[4].try_borrow_mut_data().unwrap()[400..432]
            .copy_from_slice(Pubkey::new_unique().as_ref());
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolState)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[9] = infos[16].clone();
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::AliasedAccounts)
        );
    }

    #[test]
    fn rejects_mint_authority_native_backing_and_fee_destination_substitutions() {
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[9] = info(
            *infos[9].key,
            SPL_TOKEN_PROGRAM_ID,
            token(mint, Pubkey::new_unique(), 2_000_000, false),
            1,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTokenAccount)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[10] = info(
            *infos[10].key,
            SPL_TOKEN_PROGRAM_ID,
            token(native_mint::id(), Pubkey::new_unique(), 2_000_000, true),
            2_000_001,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTokenAccount)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[10] = info(
            *infos[10].key,
            SPL_TOKEN_PROGRAM_ID,
            token(
                native_mint::id(),
                derive_pool_addresses(&DEVNET_PROGRAM_ID, &seed)
                    .unwrap()
                    .authority,
                2_000_000,
                true,
            ),
            2_000_002,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidNativeBacking)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[13] = info(
            Pubkey::new_unique(),
            SPL_TOKEN_PROGRAM_ID,
            token(native_mint::id(), Pubkey::new_unique(), 7, true),
            8,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidAccountMetas)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[6] = info(
            *infos[6].key,
            SPL_TOKEN_PROGRAM_ID,
            mint_data(6, 1_000_000, COption::Some(Pubkey::new_unique())),
            1,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidMint)
        );
    }

    #[test]
    fn rejects_target_fee_state_market_and_token_program_substitutions() {
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[11] = info(
            *infos[11].key,
            system_program::ID,
            vec![0; TARGET_ORDERS_DATA_LEN],
            1,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTargetOrders)
        );
        let (program, infos, mint, seed, args) = fixture(0, 100);
        infos[11].try_borrow_mut_data().unwrap()[0] ^= 1;
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTargetOrders)
        );
        let (program, infos, mint, seed, args) = fixture(0, 100);
        infos[4].try_borrow_mut_data().unwrap()[144..152].copy_from_slice(&26u64.to_le_bytes());
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolState)
        );
        let (program, infos, mint, seed, args) = fixture(0, 100);
        infos[4].try_borrow_mut_data().unwrap()[688] ^= 1;
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidPoolState)
        );
        let (program, infos, mint, seed, args) = fixture(0, 100);
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &Pubkey::new_unique(), args, 100),
            Err(SnapshotError::InvalidAccountMetas)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        infos[7] = info(
            mint,
            crate::beam_amm_v4::TOKEN_2022_PROGRAM_ID,
            mint_data(6, 1_000_000, COption::None),
            1,
            false,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidMint)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        let pool_authority = derive_pool_addresses(&DEVNET_PROGRAM_ID, &seed)
            .unwrap()
            .authority;
        infos[9] = info(
            *infos[9].key,
            crate::beam_amm_v4::TOKEN_2022_PROGRAM_ID,
            token(mint, pool_authority, 2_000_000, false),
            1,
            true,
            false,
        );
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTokenAccount)
        );
        let (program, mut infos, mint, seed, args) = fixture(0, 100);
        let mut frozen = SplAccount::unpack(&infos[16].try_borrow_data().unwrap()).unwrap();
        frozen.state = AccountState::Frozen;
        let mut data = vec![0; SplAccount::LEN];
        SplAccount::pack(frozen, &mut data).unwrap();
        infos[16] = info(*infos[16].key, SPL_TOKEN_PROGRAM_ID, data, 1, true, false);
        assert_eq!(
            validate_post_initialize2(&program, &infos, &mint, &seed, args, 100),
            Err(SnapshotError::InvalidTokenAccount)
        );
    }
}
