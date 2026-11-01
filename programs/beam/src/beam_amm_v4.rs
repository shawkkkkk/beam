//! Source-only Raydium AMM v4 `Initialize2` ABI. This module is not wired to a
//! Beam instruction and performs no CPI, liquidity disposal, or fee collection.
//!
//! Program reference: raydium-io/raydium-amm at
//! 91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6, `instruction.rs` and
//! `processor.rs::process_initialize2`. The recommended processor path takes
//! 19 accounts. raydium-sdk-V2 at 2aff0ed013ef52b80b46515415abe4def5302496
//! confirms the PDA seeds and program IDs, but its create-pool helper still
//! emits the accepted legacy 21-account layout with unused market placeholders.

use anchor_lang::solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey, sysvar,
};
use anchor_lang::{prelude::Pubkey, system_program};
use anchor_spl::token::spl_token::{
    native_mint,
    state::{Account as SplAccount, Mint as SplMint},
};

pub const MAINNET_PROGRAM_ID: Pubkey = pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
pub const DEVNET_PROGRAM_ID: Pubkey = pubkey!("DRaya7Kj3aMWQSy19kSjvmuwq9docCHofyP9kanQGaav");
pub const SPL_TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const MAINNET_CREATE_POOL_FEE_DESTINATION: Pubkey =
    pubkey!("7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5");
pub const DEVNET_CREATE_POOL_FEE_DESTINATION: Pubkey =
    pubkey!("9y8ENuuZ3b19quffx9hQvRVygG5ky6snHfRvGpuSfeJy");

pub const INITIALIZE2_TAG: u8 = 1;
pub const INITIALIZE2_DATA_LEN: usize = 26;
pub const INITIALIZE2_ACCOUNT_COUNT: usize = 19;
/// `AmmConfig` is a packed 544-byte struct in the pinned Raydium AMM v4 source.
/// Its last u64 is `create_pool_fee` (state.rs::AmmConfig).
pub const AMM_CONFIG_DATA_LEN: usize = 544;
const CREATE_POOL_FEE_OFFSET: usize = 536;
/// The pinned AMM v4 initializer multiplies `10^max(coin,pc decimals)` by 5
/// in a u64 before dividing. A mint with 19 decimals would overflow there.
pub const MAX_SAFE_MINT_DECIMALS: u8 = 18;

const AUTHORITY_SEED: &[u8] = b"amm authority";
const AMM_SEED: &[u8] = b"amm_associated_seed";
const LP_MINT_SEED: &[u8] = b"lp_mint_associated_seed";
const COIN_VAULT_SEED: &[u8] = b"coin_vault_associated_seed";
const PC_VAULT_SEED: &[u8] = b"pc_vault_associated_seed";
const TARGET_ORDERS_SEED: &[u8] = b"target_associated_seed";
const CONFIG_SEED: &[u8] = b"amm_config_account_seed";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmV4AbiError {
    UnsupportedProgram,
    UnsupportedMintProgram,
    IdenticalMints,
    InvalidInstructionData,
    InvalidNonce,
    InvalidAccounts,
    InvalidUserLpAta,
    InvalidConfigAccount,
    InvalidConfigData,
    CreatePoolFeeExceedsCap,
    InvalidProgramAccount,
    InvalidMigrationSigner,
    InvalidMintAccount,
    UnsupportedDecimals,
    InvalidSourceAccount,
    InvalidSourceAmount,
    InvalidNativeBacking,
    InsufficientInitialLiquidity,
}

/// Pass the mint account's verified owner, not a caller-selected token program.
/// The source program unpacks both mints as classic SPL Token mints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MintKey {
    pub key: Pubkey,
    pub owner: Pubkey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Initialize2Accounts {
    pub coin_mint: MintKey,
    pub pc_mint: MintKey,
    /// Current AMM v4 uses this key only as a unique PDA seed, not an OpenBook market.
    pub market_seed: Pubkey,
    /// Signer that funds pool creation and provides the initial liquidity.
    pub user_wallet: Pubkey,
    pub user_coin_source: Pubkey,
    pub user_pc_source: Pubkey,
    pub user_lp_ata: Pubkey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Initialize2Args {
    pub open_time: u64,
    pub init_pc_amount: u64,
    pub init_coin_amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedInitialize2 {
    pub nonce: u8,
    pub args: Initialize2Args,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolAddresses {
    pub amm: Pubkey,
    pub authority: Pubkey,
    pub nonce: u8,
    pub lp_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub target_orders: Pubkey,
    pub config: Pubkey,
    pub create_pool_fee_destination: Pubkey,
}

fn fee_destination(program_id: &Pubkey) -> Result<Pubkey, AmmV4AbiError> {
    if *program_id == MAINNET_PROGRAM_ID {
        Ok(MAINNET_CREATE_POOL_FEE_DESTINATION)
    } else if *program_id == DEVNET_PROGRAM_ID {
        Ok(DEVNET_CREATE_POOL_FEE_DESTINATION)
    } else {
        Err(AmmV4AbiError::UnsupportedProgram)
    }
}

/// Read Raydium's *current* pool-creation charge from its canonical config.
/// A future migration caller must supply `max_fee_lamports` from an authorized
/// Beam policy account and invoke this immediately before the Raydium CPI.
/// This cap does not include rent, transaction fees, or other setup funding.
/// Reading a config here neither pins future fees nor enables migration.
pub fn validate_create_pool_fee_account(
    program_id: &Pubkey,
    config: &AccountInfo<'_>,
    max_fee_lamports: u64,
) -> Result<u64, AmmV4AbiError> {
    fee_destination(program_id)?;
    let expected = Pubkey::find_program_address(&[CONFIG_SEED], program_id).0;
    if config.key != &expected || config.owner != program_id {
        return Err(AmmV4AbiError::InvalidConfigAccount);
    }
    let data = config
        .try_borrow_data()
        .map_err(|_| AmmV4AbiError::InvalidConfigData)?;
    if data.len() != AMM_CONFIG_DATA_LEN {
        return Err(AmmV4AbiError::InvalidConfigData);
    }
    let fee_bytes: [u8; 8] = data[CREATE_POOL_FEE_OFFSET..AMM_CONFIG_DATA_LEN]
        .try_into()
        .map_err(|_| AmmV4AbiError::InvalidConfigData)?;
    let fee = u64::from_le_bytes(fee_bytes);
    if fee > max_fee_lamports {
        return Err(AmmV4AbiError::CreatePoolFeeExceedsCap);
    }
    Ok(fee)
}

fn unpack_classic_mint(info: &AccountInfo<'_>) -> Result<SplMint, AmmV4AbiError> {
    if info.owner != &SPL_TOKEN_PROGRAM_ID {
        return Err(AmmV4AbiError::UnsupportedMintProgram);
    }
    let data = info
        .try_borrow_data()
        .map_err(|_| AmmV4AbiError::InvalidMintAccount)?;
    let mint = SplMint::unpack(&data).map_err(|_| AmmV4AbiError::InvalidMintAccount)?;
    if mint.decimals > MAX_SAFE_MINT_DECIMALS {
        return Err(AmmV4AbiError::UnsupportedDecimals);
    }
    Ok(mint)
}

fn check_source(
    info: &AccountInfo<'_>,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    require_native: bool,
) -> Result<(), AmmV4AbiError> {
    if amount == 0
        || info.owner != &SPL_TOKEN_PROGRAM_ID
        || info.key != &user_lp_ata(authority, mint)
    {
        return Err(AmmV4AbiError::InvalidSourceAccount);
    }
    let data = info
        .try_borrow_data()
        .map_err(|_| AmmV4AbiError::InvalidSourceAccount)?;
    let source = SplAccount::unpack(&data).map_err(|_| AmmV4AbiError::InvalidSourceAccount)?;
    if source.mint != *mint
        || source.owner != *authority
        || source.delegate != COption::None
        || source.close_authority != COption::None
        || source.is_native.is_some() != require_native
    {
        return Err(AmmV4AbiError::InvalidSourceAccount);
    }
    // Beam must not silently leave donated tokens in, or sweep them from, a
    // migration-source ATA. The planned pool amounts must exactly match.
    if source.amount != amount {
        return Err(AmmV4AbiError::InvalidSourceAmount);
    }
    if require_native {
        let COption::Some(rent_reserve) = source.is_native else {
            return Err(AmmV4AbiError::InvalidNativeBacking);
        };
        let backed_lamports = rent_reserve
            .checked_add(source.amount)
            .ok_or(AmmV4AbiError::InvalidNativeBacking)?;
        if info.lamports() != backed_lamports {
            return Err(AmmV4AbiError::InvalidNativeBacking);
        }
    }
    Ok(())
}

/// The pinned initializer reserves `10^coin_decimals` LP after taking the
/// integer square root of the two deposited token amounts. Reject a planned
/// pool that would yield no usable LP before any migration funds move.
pub fn expected_initial_user_lp(
    coin_amount: u64,
    pc_amount: u64,
    coin_decimals: u8,
) -> Result<u64, AmmV4AbiError> {
    let reserve = 10u64
        .checked_pow(coin_decimals.into())
        .ok_or(AmmV4AbiError::UnsupportedDecimals)?;
    let liquidity = u128::from(coin_amount)
        .checked_mul(u128::from(pc_amount))
        .ok_or(AmmV4AbiError::InsufficientInitialLiquidity)?
        .isqrt();
    let liquidity: u64 = liquidity
        .try_into()
        .map_err(|_| AmmV4AbiError::InsufficientInitialLiquidity)?;
    liquidity
        .checked_sub(reserve)
        .filter(|amount| *amount > 0)
        .ok_or(AmmV4AbiError::InsufficientInitialLiquidity)
}

/// Source-only preflight for a *native-SOL*, classic-SPL `Initialize2` CPI.
/// Build the instruction from actual AccountInfos, not caller-claimed owners.
/// Index 15 is Beam's PDA and is signed by `invoke_signed` during a future CPI;
/// it is not required to be an outer transaction signer. `max_fee_lamports`
/// must eventually come from a stored Beam policy, not an untrusted caller.
/// This helper does not move funds, invoke Raydium, verify pool creation, or
/// decide LP custody and cannot enable the gated migration by itself.
pub fn preflight_native_sol_initialize2<'info>(
    program: &AccountInfo<'info>,
    infos: &[AccountInfo<'info>],
    beam_mint: &Pubkey,
    args: Initialize2Args,
    max_fee_lamports: u64,
) -> Result<Initialize2Accounts, AmmV4AbiError> {
    fee_destination(program.key)?;
    if !program.executable || infos.len() != INITIALIZE2_ACCOUNT_COUNT {
        return Err(AmmV4AbiError::InvalidProgramAccount);
    }
    let (signer, _) =
        Pubkey::find_program_address(&[b"pool-authority", beam_mint.as_ref()], &crate::ID);
    if infos[15].key != &signer
        || infos[15].owner != &system_program::ID
        || !infos[15].data_is_empty()
    {
        return Err(AmmV4AbiError::InvalidMigrationSigner);
    }
    if infos[7].key != beam_mint || infos[8].key != &native_mint::id() {
        return Err(AmmV4AbiError::InvalidMintAccount);
    }
    for index in [0, 1, 2] {
        if !infos[index].executable {
            return Err(AmmV4AbiError::InvalidProgramAccount);
        }
    }
    let coin_mint = unpack_classic_mint(&infos[7])?;
    unpack_classic_mint(&infos[8])?;
    let accounts = Initialize2Accounts {
        coin_mint: MintKey {
            key: *infos[7].key,
            owner: *infos[7].owner,
        },
        pc_mint: MintKey {
            key: *infos[8].key,
            owner: *infos[8].owner,
        },
        market_seed: *infos[14].key,
        user_wallet: *infos[15].key,
        user_coin_source: *infos[16].key,
        user_pc_source: *infos[17].key,
        user_lp_ata: *infos[18].key,
    };
    let instruction = initialize2_instruction(*program.key, &accounts, args)?;
    for (info, meta) in infos.iter().zip(&instruction.accounts) {
        if info.key != &meta.pubkey || (meta.is_writable && !info.is_writable) {
            return Err(AmmV4AbiError::InvalidAccounts);
        }
    }
    validate_create_pool_fee_account(program.key, &infos[12], max_fee_lamports)?;
    check_source(&infos[16], beam_mint, &signer, args.init_coin_amount, false)?;
    check_source(
        &infos[17],
        &native_mint::id(),
        &signer,
        args.init_pc_amount,
        true,
    )?;
    expected_initial_user_lp(
        args.init_coin_amount,
        args.init_pc_amount,
        coin_mint.decimals,
    )?;
    Ok(accounts)
}

fn associated(program_id: &Pubkey, market_seed: &Pubkey, suffix: &[u8]) -> Pubkey {
    Pubkey::find_program_address(
        &[program_id.as_ref(), market_seed.as_ref(), suffix],
        program_id,
    )
    .0
}

pub fn derive_pool_addresses(
    program_id: &Pubkey,
    market_seed: &Pubkey,
) -> Result<PoolAddresses, AmmV4AbiError> {
    let create_pool_fee_destination = fee_destination(program_id)?;
    let (authority, nonce) = Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);
    Ok(PoolAddresses {
        amm: associated(program_id, market_seed, AMM_SEED),
        authority,
        nonce,
        lp_mint: associated(program_id, market_seed, LP_MINT_SEED),
        coin_vault: associated(program_id, market_seed, COIN_VAULT_SEED),
        pc_vault: associated(program_id, market_seed, PC_VAULT_SEED),
        target_orders: associated(program_id, market_seed, TARGET_ORDERS_SEED),
        config: Pubkey::find_program_address(&[CONFIG_SEED], program_id).0,
        create_pool_fee_destination,
    })
}

pub fn user_lp_ata(user_wallet: &Pubkey, lp_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            user_wallet.as_ref(),
            SPL_TOKEN_PROGRAM_ID.as_ref(),
            lp_mint.as_ref(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}

/// This is an ABI guard; execution must verify actual account owners again.
pub fn validate_mints(accounts: &Initialize2Accounts) -> Result<(), AmmV4AbiError> {
    if accounts.coin_mint.owner != SPL_TOKEN_PROGRAM_ID
        || accounts.pc_mint.owner != SPL_TOKEN_PROGRAM_ID
    {
        return Err(AmmV4AbiError::UnsupportedMintProgram);
    }
    if accounts.coin_mint.key == accounts.pc_mint.key {
        return Err(AmmV4AbiError::IdenticalMints);
    }
    Ok(())
}

/// Tag 1, authority bump, then three little-endian u64 values. CPMM uses an
/// eight-byte Anchor discriminator and is incompatible with this encoding.
pub fn encode_initialize2(nonce: u8, args: Initialize2Args) -> [u8; INITIALIZE2_DATA_LEN] {
    let mut data = [0u8; INITIALIZE2_DATA_LEN];
    data[0] = INITIALIZE2_TAG;
    data[1] = nonce;
    data[2..10].copy_from_slice(&args.open_time.to_le_bytes());
    data[10..18].copy_from_slice(&args.init_pc_amount.to_le_bytes());
    data[18..26].copy_from_slice(&args.init_coin_amount.to_le_bytes());
    data
}

pub fn decode_initialize2(data: &[u8]) -> Result<DecodedInitialize2, AmmV4AbiError> {
    if data.len() != INITIALIZE2_DATA_LEN || data[0] != INITIALIZE2_TAG {
        return Err(AmmV4AbiError::InvalidInstructionData);
    }
    let read_u64 = |start| u64::from_le_bytes(data[start..start + 8].try_into().unwrap());
    Ok(DecodedInitialize2 {
        nonce: data[1],
        args: Initialize2Args {
            open_time: read_u64(2),
            init_pc_amount: read_u64(10),
            init_coin_amount: read_u64(18),
        },
    })
}

/// Build only the recommended 19-account ABI. The signing and liquidity
/// policy belong to the eventual Beam migration instruction, not this module.
pub fn initialize2_instruction(
    program_id: Pubkey,
    accounts: &Initialize2Accounts,
    args: Initialize2Args,
) -> Result<Instruction, AmmV4AbiError> {
    validate_mints(accounts)?;
    let p = derive_pool_addresses(&program_id, &accounts.market_seed)?;
    if accounts.user_lp_ata != user_lp_ata(&accounts.user_wallet, &p.lp_mint) {
        return Err(AmmV4AbiError::InvalidUserLpAta);
    }
    let metas = vec![
        AccountMeta::new_readonly(SPL_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(sysvar::rent::ID, false),
        AccountMeta::new(p.amm, false),
        AccountMeta::new_readonly(p.authority, false),
        AccountMeta::new(p.lp_mint, false),
        AccountMeta::new_readonly(accounts.coin_mint.key, false),
        AccountMeta::new_readonly(accounts.pc_mint.key, false),
        AccountMeta::new(p.coin_vault, false),
        AccountMeta::new(p.pc_vault, false),
        AccountMeta::new(p.target_orders, false),
        AccountMeta::new_readonly(p.config, false),
        AccountMeta::new(p.create_pool_fee_destination, false),
        AccountMeta::new_readonly(accounts.market_seed, false),
        AccountMeta::new(accounts.user_wallet, true),
        AccountMeta::new(accounts.user_coin_source, false),
        AccountMeta::new(accounts.user_pc_source, false),
        AccountMeta::new(accounts.user_lp_ata, false),
    ];
    Ok(Instruction {
        program_id,
        accounts: metas,
        data: encode_initialize2(p.nonce, args).to_vec(),
    })
}

/// Pure full-layout check for instruction fixtures and future CPI preflight.
pub fn validate_initialize2_instruction(
    instruction: &Instruction,
    accounts: &Initialize2Accounts,
) -> Result<DecodedInitialize2, AmmV4AbiError> {
    let decoded = decode_initialize2(&instruction.data)?;
    let expected = initialize2_instruction(instruction.program_id, accounts, decoded.args)?;
    if decoded.nonce != expected.data[1] {
        return Err(AmmV4AbiError::InvalidNonce);
    }
    if instruction.accounts != expected.accounts {
        return Err(AmmV4AbiError::InvalidAccounts);
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_spl::token::spl_token::state::AccountState;

    fn synthetic_info(
        key: Pubkey,
        owner: Pubkey,
        data: Vec<u8>,
        writable: bool,
        executable: bool,
    ) -> AccountInfo<'static> {
        synthetic_info_with_lamports(key, owner, data, writable, executable, 1)
    }

    fn synthetic_info_with_lamports(
        key: Pubkey,
        owner: Pubkey,
        data: Vec<u8>,
        writable: bool,
        executable: bool,
        balance: u64,
    ) -> AccountInfo<'static> {
        // Tiny test-only allocations live until the process exits. No source
        // or production account is manufactured by this helper.
        let key = Box::leak(Box::new(key));
        let owner = Box::leak(Box::new(owner));
        let lamports = Box::leak(Box::new(balance));
        let data = Box::leak(data.into_boxed_slice());
        AccountInfo::new(key, false, writable, lamports, data, owner, executable, 0)
    }

    fn mint_data(decimals: u8) -> Vec<u8> {
        let mut data = vec![0; SplMint::LEN];
        SplMint::pack(
            SplMint {
                mint_authority: COption::None,
                supply: 1_000_000,
                decimals,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            &mut data,
        )
        .unwrap();
        data
    }

    fn source_data(mint: Pubkey, authority: Pubkey, amount: u64, native: bool) -> Vec<u8> {
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

    fn native_sol_fixture() -> (
        AccountInfo<'static>,
        Vec<AccountInfo<'static>>,
        Pubkey,
        Initialize2Args,
    ) {
        let program_id = DEVNET_PROGRAM_ID;
        let beam_mint = Pubkey::new_from_array([42; 32]);
        let signer =
            Pubkey::find_program_address(&[b"pool-authority", beam_mint.as_ref()], &crate::ID).0;
        let args = Initialize2Args {
            open_time: 0,
            init_pc_amount: 2_000_000,
            init_coin_amount: 2_000_000,
        };
        let market_seed = Pubkey::new_from_array([43; 32]);
        let p = derive_pool_addresses(&program_id, &market_seed).unwrap();
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
        let infos = ix
            .accounts
            .iter()
            .enumerate()
            .map(|(index, meta)| {
                let owner = match index {
                    7 | 8 | 16 | 17 => SPL_TOKEN_PROGRAM_ID,
                    12 => program_id,
                    _ => system_program::ID,
                };
                let data = match index {
                    7 => mint_data(6),
                    8 => mint_data(9),
                    12 => vec![0; AMM_CONFIG_DATA_LEN],
                    16 => source_data(beam_mint, signer, args.init_coin_amount, false),
                    17 => source_data(native_mint::id(), signer, args.init_pc_amount, true),
                    _ => Vec::new(),
                };
                synthetic_info_with_lamports(
                    meta.pubkey,
                    owner,
                    data,
                    meta.is_writable,
                    index <= 2,
                    if index == 17 {
                        args.init_pc_amount + 1
                    } else {
                        1
                    },
                )
            })
            .collect();
        let program = synthetic_info(program_id, system_program::ID, Vec::new(), false, true);
        (program, infos, beam_mint, args)
    }

    #[test]
    fn actual_account_preflight_accepts_classic_native_sol_without_outer_pda_signature() {
        let (program, infos, mint, args) = native_sol_fixture();
        assert!(!infos[15].is_signer);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0)
                .unwrap()
                .user_wallet,
            *infos[15].key,
        );
        assert_eq!(
            expected_initial_user_lp(args.init_coin_amount, args.init_pc_amount, 6),
            Ok(1_000_000)
        );
    }

    #[test]
    fn actual_account_preflight_rejects_mint_owner_decimals_and_source_substitution() {
        let (program, mut infos, mint, args) = native_sol_fixture();
        infos[7] = synthetic_info(mint, TOKEN_2022_PROGRAM_ID, mint_data(6), false, false);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::UnsupportedMintProgram)
        );

        let (program, mut infos, mint, args) = native_sol_fixture();
        infos[7] = synthetic_info(mint, SPL_TOKEN_PROGRAM_ID, mint_data(19), false, false);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::UnsupportedDecimals)
        );
        assert!(unpack_classic_mint(&synthetic_info(
            mint,
            SPL_TOKEN_PROGRAM_ID,
            mint_data(18),
            false,
            false
        ))
        .is_ok());
        infos[7] = synthetic_info(mint, SPL_TOKEN_PROGRAM_ID, mint_data(6), false, false);

        infos.swap(16, 17);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidSourceAccount)
        );
    }

    #[test]
    fn actual_account_preflight_rejects_unbacked_or_delegated_sources() {
        let (program, mut infos, mint, args) = native_sol_fixture();
        let signer = *infos[15].key;
        infos[16] = synthetic_info(
            *infos[16].key,
            SPL_TOKEN_PROGRAM_ID,
            source_data(mint, signer, args.init_coin_amount + 1, false),
            true,
            false,
        );
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidSourceAmount)
        );
        let mut delegated = SplAccount::unpack(&infos[16].try_borrow_data().unwrap()).unwrap();
        delegated.amount = args.init_coin_amount;
        delegated.delegate = COption::Some(Pubkey::new_from_array([55; 32]));
        let mut data = vec![0; SplAccount::LEN];
        SplAccount::pack(delegated, &mut data).unwrap();
        infos[16] = synthetic_info(*infos[16].key, SPL_TOKEN_PROGRAM_ID, data, true, false);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidSourceAccount)
        );
        let (_, infos, mint, mut args) = native_sol_fixture();
        args.init_coin_amount = 0;
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidSourceAccount)
        );
    }

    #[test]
    fn actual_account_preflight_requires_exact_wsol_backing_and_usable_lp() {
        let (program, mut infos, mint, args) = native_sol_fixture();
        let source = infos[17].clone();
        for balance in [args.init_pc_amount, args.init_pc_amount + 2] {
            infos[17] = synthetic_info_with_lamports(
                *source.key,
                SPL_TOKEN_PROGRAM_ID,
                source.try_borrow_data().unwrap().to_vec(),
                true,
                false,
                balance,
            );
            assert_eq!(
                preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
                Err(AmmV4AbiError::InvalidNativeBacking)
            );
        }
        let mut overflowing = SplAccount::unpack(&source.try_borrow_data().unwrap()).unwrap();
        overflowing.is_native = COption::Some(u64::MAX);
        let mut data = vec![0; SplAccount::LEN];
        SplAccount::pack(overflowing, &mut data).unwrap();
        infos[17] = synthetic_info_with_lamports(
            *source.key,
            SPL_TOKEN_PROGRAM_ID,
            data,
            true,
            false,
            u64::MAX,
        );
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidNativeBacking)
        );
        assert_eq!(
            expected_initial_user_lp(600_000, 900_000, 6),
            Err(AmmV4AbiError::InsufficientInitialLiquidity)
        );
        assert_eq!(
            expected_initial_user_lp(1_000_000, 1_000_000, 6),
            Err(AmmV4AbiError::InsufficientInitialLiquidity)
        );
    }

    #[test]
    fn actual_account_preflight_rejects_wrong_meta_signer_and_mutable_config_fee() {
        let (mut program, mut infos, mint, args) = native_sol_fixture();
        program.executable = false;
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidProgramAccount)
        );
        program.executable = true;
        infos[16].is_writable = false;
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 0),
            Err(AmmV4AbiError::InvalidAccounts)
        );
        infos[16].is_writable = true;
        infos[12].try_borrow_mut_data().unwrap()[CREATE_POOL_FEE_OFFSET..]
            .copy_from_slice(&2u64.to_le_bytes());
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 1),
            Err(AmmV4AbiError::CreatePoolFeeExceedsCap)
        );
        assert!(preflight_native_sol_initialize2(&program, &infos, &mint, args, 2).is_ok());
        let wrong = Pubkey::new_from_array([56; 32]);
        infos[15] = synthetic_info(wrong, system_program::ID, Vec::new(), true, false);
        assert_eq!(
            preflight_native_sol_initialize2(&program, &infos, &mint, args, 2),
            Err(AmmV4AbiError::InvalidMigrationSigner)
        );
    }

    fn with_config<R>(
        key: Pubkey,
        owner: Pubkey,
        mut data: Vec<u8>,
        f: impl FnOnce(&AccountInfo<'_>) -> R,
    ) -> R {
        let mut lamports = 1;
        let account = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        f(&account)
    }

    fn config_key(program_id: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[CONFIG_SEED], program_id).0
    }

    fn config_data(fee: u64) -> Vec<u8> {
        let mut data = vec![0; AMM_CONFIG_DATA_LEN];
        data[CREATE_POOL_FEE_OFFSET..].copy_from_slice(&fee.to_le_bytes());
        data
    }

    #[test]
    fn current_create_pool_fee_is_read_from_canonical_owned_config() {
        for program in [MAINNET_PROGRAM_ID, DEVNET_PROGRAM_ID] {
            let key = config_key(&program);
            for (fee, cap, expected) in [
                (0, 0, Ok(0)),
                (123, 123, Ok(123)),
                (124, 123, Err(AmmV4AbiError::CreatePoolFeeExceedsCap)),
                (u64::MAX, u64::MAX, Ok(u64::MAX)),
            ] {
                with_config(key, program, config_data(fee), |account| {
                    assert_eq!(
                        validate_create_pool_fee_account(&program, account, cap),
                        expected
                    );
                });
            }
        }
    }

    #[test]
    fn create_pool_fee_config_rejects_substitution_and_rechecks_changes() {
        let program = MAINNET_PROGRAM_ID;
        let key = config_key(&program);
        let wrong_key = Pubkey::new_from_array([21; 32]);
        let wrong_owner = Pubkey::new_from_array([22; 32]);
        let unknown_program = Pubkey::new_from_array([23; 32]);
        with_config(wrong_key, program, config_data(1), |account| {
            assert_eq!(
                validate_create_pool_fee_account(&program, account, 1),
                Err(AmmV4AbiError::InvalidConfigAccount)
            );
        });
        with_config(key, wrong_owner, config_data(1), |account| {
            assert_eq!(
                validate_create_pool_fee_account(&program, account, 1),
                Err(AmmV4AbiError::InvalidConfigAccount)
            );
        });
        with_config(key, program, config_data(1), |account| {
            assert_eq!(
                validate_create_pool_fee_account(&unknown_program, account, 1),
                Err(AmmV4AbiError::UnsupportedProgram)
            );
        });
        for len in [AMM_CONFIG_DATA_LEN - 1, AMM_CONFIG_DATA_LEN + 1] {
            with_config(key, program, vec![0; len], |account| {
                assert_eq!(
                    validate_create_pool_fee_account(&program, account, 1),
                    Err(AmmV4AbiError::InvalidConfigData)
                );
            });
        }
        with_config(key, program, config_data(1), |account| {
            assert_eq!(
                validate_create_pool_fee_account(&program, account, 1),
                Ok(1)
            );
            account.try_borrow_mut_data().unwrap()[CREATE_POOL_FEE_OFFSET..]
                .copy_from_slice(&2u64.to_le_bytes());
            assert_eq!(
                validate_create_pool_fee_account(&program, account, 1),
                Err(AmmV4AbiError::CreatePoolFeeExceedsCap)
            );
        });
    }

    fn accounts() -> Initialize2Accounts {
        let mut accounts = Initialize2Accounts {
            coin_mint: MintKey {
                key: Pubkey::new_from_array([1; 32]),
                owner: SPL_TOKEN_PROGRAM_ID,
            },
            pc_mint: MintKey {
                key: Pubkey::new_from_array([2; 32]),
                owner: SPL_TOKEN_PROGRAM_ID,
            },
            market_seed: Pubkey::new_from_array([3; 32]),
            user_wallet: Pubkey::new_from_array([4; 32]),
            user_coin_source: Pubkey::new_from_array([5; 32]),
            user_pc_source: Pubkey::new_from_array([6; 32]),
            user_lp_ata: Pubkey::new_from_array([7; 32]),
        };
        let p = derive_pool_addresses(&MAINNET_PROGRAM_ID, &accounts.market_seed).unwrap();
        accounts.user_lp_ata = user_lp_ata(&accounts.user_wallet, &p.lp_mint);
        accounts
    }

    fn args() -> Initialize2Args {
        Initialize2Args {
            open_time: 0x0102_0304_0506_0708,
            init_pc_amount: 0x1112_1314_1516_1718,
            init_coin_amount: 0x2122_2324_2526_2728,
        }
    }

    #[test]
    fn golden_initialize2_encoding_matches_raydium_tag_and_le_fields() {
        let bytes = encode_initialize2(7, args());
        assert_eq!(
            bytes,
            [
                1, 7, 8, 7, 6, 5, 4, 3, 2, 1, 24, 23, 22, 21, 20, 19, 18, 17, 40, 39, 38, 37, 36,
                35, 34, 33,
            ]
        );
        assert_eq!(
            decode_initialize2(&bytes),
            Ok(DecodedInitialize2 {
                nonce: 7,
                args: args()
            })
        );
    }

    #[test]
    fn recommended_account_order_and_flags_are_exact() {
        let a = accounts();
        let p = derive_pool_addresses(&MAINNET_PROGRAM_ID, &a.market_seed).unwrap();
        let ix = initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()).unwrap();
        let keys = [
            SPL_TOKEN_PROGRAM_ID,
            ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program::ID,
            sysvar::rent::ID,
            p.amm,
            p.authority,
            p.lp_mint,
            a.coin_mint.key,
            a.pc_mint.key,
            p.coin_vault,
            p.pc_vault,
            p.target_orders,
            p.config,
            MAINNET_CREATE_POOL_FEE_DESTINATION,
            a.market_seed,
            a.user_wallet,
            a.user_coin_source,
            a.user_pc_source,
            a.user_lp_ata,
        ];
        let writable = [
            false, false, false, false, true, false, true, false, false, true, true, true, false,
            true, false, true, true, true, true,
        ];
        assert_eq!(ix.accounts.len(), INITIALIZE2_ACCOUNT_COUNT);
        for (i, meta) in ix.accounts.iter().enumerate() {
            assert_eq!(meta.pubkey, keys[i], "account {i} key");
            assert_eq!(meta.is_writable, writable[i], "account {i} writable");
            assert_eq!(meta.is_signer, i == 15, "account {i} signer");
        }
        assert_eq!(ix.data[1], p.nonce);
        assert_eq!(
            validate_initialize2_instruction(&ix, &a).unwrap().args,
            args()
        );
    }

    #[test]
    fn pda_seeds_match_raydium_source_and_sdk() {
        let a = accounts();
        let p = derive_pool_addresses(&MAINNET_PROGRAM_ID, &a.market_seed).unwrap();
        assert_eq!(
            p.amm,
            Pubkey::find_program_address(
                &[
                    MAINNET_PROGRAM_ID.as_ref(),
                    a.market_seed.as_ref(),
                    b"amm_associated_seed"
                ],
                &MAINNET_PROGRAM_ID,
            )
            .0
        );
        assert_eq!(
            (p.authority, p.nonce),
            Pubkey::find_program_address(&[b"amm authority"], &MAINNET_PROGRAM_ID,)
        );
        assert_eq!(
            p.config,
            Pubkey::find_program_address(&[b"amm_config_account_seed"], &MAINNET_PROGRAM_ID,).0
        );
        for (actual, suffix) in [
            (p.lp_mint, b"lp_mint_associated_seed".as_slice()),
            (p.coin_vault, b"coin_vault_associated_seed".as_slice()),
            (p.pc_vault, b"pc_vault_associated_seed".as_slice()),
            (p.target_orders, b"target_associated_seed".as_slice()),
        ] {
            assert_eq!(
                actual,
                Pubkey::find_program_address(
                    &[MAINNET_PROGRAM_ID.as_ref(), a.market_seed.as_ref(), suffix],
                    &MAINNET_PROGRAM_ID,
                )
                .0
            );
        }
        let dev = derive_pool_addresses(&DEVNET_PROGRAM_ID, &a.market_seed).unwrap();
        assert_eq!(
            dev.create_pool_fee_destination,
            DEVNET_CREATE_POOL_FEE_DESTINATION
        );
        assert_ne!(dev.amm, p.amm);
    }

    #[test]
    fn rejects_cpmm_bytes_program_and_account_layouts() {
        let a = accounts();
        let mut ix = initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()).unwrap();
        // CPMM's Initialize discriminator plus three u64s is 32 bytes.
        let mut cpmm_data = vec![175, 175, 109, 31, 13, 152, 155, 237];
        cpmm_data.extend_from_slice(&[0; 24]);
        assert_eq!(
            decode_initialize2(&cpmm_data),
            Err(AmmV4AbiError::InvalidInstructionData)
        );
        assert_eq!(
            decode_initialize2(&[175; 26]),
            Err(AmmV4AbiError::InvalidInstructionData)
        );
        assert_eq!(
            decode_initialize2(&ix.data[..25]),
            Err(AmmV4AbiError::InvalidInstructionData)
        );
        ix.accounts.push(AccountMeta::new_readonly(
            Pubkey::new_from_array([8; 32]),
            false,
        ));
        assert_eq!(
            validate_initialize2_instruction(&ix, &a),
            Err(AmmV4AbiError::InvalidAccounts)
        );
        ix.accounts.pop();
        ix.accounts.swap(9, 10);
        assert_eq!(
            validate_initialize2_instruction(&ix, &a),
            Err(AmmV4AbiError::InvalidAccounts)
        );
        ix = initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()).unwrap();
        ix.accounts[15].is_signer = false;
        assert_eq!(
            validate_initialize2_instruction(&ix, &a),
            Err(AmmV4AbiError::InvalidAccounts)
        );
        ix = initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()).unwrap();
        ix.data[1] ^= 1;
        assert_eq!(
            validate_initialize2_instruction(&ix, &a),
            Err(AmmV4AbiError::InvalidNonce)
        );
        assert_eq!(
            initialize2_instruction(
                pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"),
                &a,
                args(),
            ),
            Err(AmmV4AbiError::UnsupportedProgram)
        );
    }

    #[test]
    fn rejects_token_2022_and_identical_mints() {
        let mut a = accounts();
        a.coin_mint.owner = TOKEN_2022_PROGRAM_ID;
        assert_eq!(
            initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()),
            Err(AmmV4AbiError::UnsupportedMintProgram)
        );
        a = accounts();
        a.pc_mint.owner = TOKEN_2022_PROGRAM_ID;
        assert_eq!(
            initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()),
            Err(AmmV4AbiError::UnsupportedMintProgram)
        );
        a = accounts();
        a.pc_mint.key = a.coin_mint.key;
        assert_eq!(
            initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()),
            Err(AmmV4AbiError::IdenticalMints)
        );
        a = accounts();
        a.user_lp_ata = Pubkey::new_from_array([7; 32]);
        assert_eq!(
            initialize2_instruction(MAINNET_PROGRAM_ID, &a, args()),
            Err(AmmV4AbiError::InvalidUserLpAta)
        );
    }
}
