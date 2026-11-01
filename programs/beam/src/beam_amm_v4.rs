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
    instruction::{AccountMeta, Instruction},
    pubkey, sysvar,
};
use anchor_lang::{prelude::Pubkey, system_program};

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
