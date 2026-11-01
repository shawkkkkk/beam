//! Synthetic AMM v4 ABI/CPI exercise. The "Raydium" processor below is a
//! test-only mock registered under the official devnet ID *inside ProgramTest*.
//! It is not the Raydium binary or Beam's migration entrypoint; Beam's real
//! migration gate stays off. All balances and accounts are local fixtures.
use anchor_lang::solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
};
use beam_program::beam_amm_v4::{
    decode_initialize2, derive_pool_addresses, initialize2_instruction, user_lp_ata,
    validate_create_pool_fee_account, Initialize2Accounts, Initialize2Args, MintKey,
    AMM_CONFIG_DATA_LEN, ASSOCIATED_TOKEN_PROGRAM_ID, DEVNET_PROGRAM_ID, INITIALIZE2_ACCOUNT_COUNT,
    INITIALIZE2_DATA_LEN, SPL_TOKEN_PROGRAM_ID,
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Signer, system_instruction, system_program,
    sysvar, transaction::Transaction,
};

const CALLER_ID: Pubkey = Pubkey::new_from_array([201; 32]);
const WALLET_SEED: &[u8] = b"migration-wallet";
const MOCK_CREATE_FEE: u64 = 50_000;
const MOCK_POST_CPI_ERROR: u32 = 91;
const MOCK_LATE_ERROR: u32 = 92;
const START_WALLET: u64 = 5_000_000;
const START_FEE_DESTINATION: u64 = 1_000_000;

fn wallet_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[WALLET_SEED], &CALLER_ID)
}

fn read_u64(data: &[u8], start: usize) -> Result<u64, ProgramError> {
    let bytes: [u8; 8] = data
        .get(start..start + 8)
        .ok_or(ProgramError::InvalidInstructionData)?
        .try_into()
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(u64::from_le_bytes(bytes))
}

// This test-only caller stands in for a future migration instruction. It has
// no source or entrypoint relationship to Beam's gated migration candidate.
fn synthetic_caller<'a>(
    program_id: &Pubkey,
    infos: &[AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    if *program_id != CALLER_ID || infos.len() != INITIALIZE2_ACCOUNT_COUNT + 1 || data.len() != 33
    {
        return Err(ProgramError::InvalidInstructionData);
    }
    if infos[19].key != &DEVNET_PROGRAM_ID || !infos[19].executable {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mode = data[0];
    let max_fee = read_u64(data, 1)?;
    let args = Initialize2Args {
        open_time: read_u64(data, 9)?,
        init_pc_amount: read_u64(data, 17)?,
        init_coin_amount: read_u64(data, 25)?,
    };
    let (wallet, bump) = wallet_pda();
    if infos[15].key != &wallet {
        return Err(ProgramError::InvalidAccountData);
    }
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
        user_wallet: wallet,
        user_coin_source: *infos[16].key,
        user_pc_source: *infos[17].key,
        user_lp_ata: *infos[18].key,
    };
    validate_create_pool_fee_account(&DEVNET_PROGRAM_ID, &infos[12], max_fee)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let mut ix = initialize2_instruction(DEVNET_PROGRAM_ID, &accounts, args)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    for (given, expected) in infos.iter().zip(&ix.accounts) {
        if given.key != &expected.pubkey || (expected.is_writable && !given.is_writable) {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    if mode == 3 {
        ix.data[0] = 255; // malformed tag reaches the mock processor
    }
    if mode == 2 {
        // Remove signer privilege in the test-only CPI so the mock checks the
        // missing authority without ProgramTest's privilege-escalation panic.
        ix.accounts[15].is_signer = false;
        invoke(&ix, infos)?;
    } else {
        invoke_signed(&ix, infos, &[&[WALLET_SEED, &[bump]]])?;
    }
    if mode == 1 {
        return Err(ProgramError::Custom(MOCK_POST_CPI_ERROR));
    }
    Ok(())
}

// The mock checks the builder's real ABI and simulates just one pool write and
// one system transfer. It makes no assertion about LP minting, LP fee rights,
// real Raydium account parsing, or live setup economics.
fn mock_amm_v4<'a>(program_id: &Pubkey, infos: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    if *program_id != DEVNET_PROGRAM_ID || infos.len() != INITIALIZE2_ACCOUNT_COUNT {
        return Err(ProgramError::InvalidAccountData);
    }
    if data.len() != INITIALIZE2_DATA_LEN {
        return Err(ProgramError::InvalidInstructionData);
    }
    let decoded = decode_initialize2(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    let addresses = derive_pool_addresses(program_id, infos[14].key)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let expected = [
        SPL_TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program::ID,
        sysvar::rent::ID,
        addresses.amm,
        addresses.authority,
        addresses.lp_mint,
        *infos[7].key,
        *infos[8].key,
        addresses.coin_vault,
        addresses.pc_vault,
        addresses.target_orders,
        addresses.config,
        addresses.create_pool_fee_destination,
        *infos[14].key,
        wallet_pda().0,
        *infos[16].key,
        *infos[17].key,
        *infos[18].key,
    ];
    for (i, key) in expected.iter().enumerate() {
        if infos[i].key != key {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    if !infos[15].is_signer || !infos[4].is_writable || !infos[13].is_writable {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if decoded.nonce != addresses.nonce || infos[4].owner != program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let fee = validate_create_pool_fee_account(program_id, &infos[12], u64::MAX)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    infos[4].try_borrow_mut_data()?[0] = 1;
    let transfer = system_instruction::transfer(infos[15].key, infos[13].key, fee);
    invoke(
        &transfer,
        &[infos[15].clone(), infos[13].clone(), infos[2].clone()],
    )?;
    if decoded.args.open_time == 2 {
        return Err(ProgramError::Custom(MOCK_LATE_ERROR));
    }
    Ok(())
}

struct Fixture {
    bank: ProgramTestContext,
    expected: Instruction,
    amm: Pubkey,
    fee_destination: Pubkey,
}

fn dummy(owner: Pubkey, lamports: u64, data: Vec<u8>) -> Account {
    Account {
        lamports,
        data,
        owner,
        executable: false,
        rent_epoch: 0,
    }
}

async fn fixture(config_owner: Pubkey, create_fee: u64, truncated_config: bool) -> Fixture {
    let wallet = wallet_pda().0;
    let market_seed = Pubkey::new_unique();
    let mut accounts = Initialize2Accounts {
        coin_mint: MintKey {
            key: Pubkey::new_unique(),
            owner: SPL_TOKEN_PROGRAM_ID,
        },
        pc_mint: MintKey {
            key: Pubkey::new_unique(),
            owner: SPL_TOKEN_PROGRAM_ID,
        },
        market_seed,
        user_wallet: wallet,
        user_coin_source: Pubkey::new_unique(),
        user_pc_source: Pubkey::new_unique(),
        user_lp_ata: Pubkey::default(),
    };
    let addresses = derive_pool_addresses(&DEVNET_PROGRAM_ID, &market_seed).unwrap();
    accounts.user_lp_ata = user_lp_ata(&wallet, &addresses.lp_mint);
    let expected = initialize2_instruction(
        DEVNET_PROGRAM_ID,
        &accounts,
        Initialize2Args {
            open_time: 1,
            init_pc_amount: 200,
            init_coin_amount: 300,
        },
    )
    .unwrap();
    let keys = [
        SPL_TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
        system_program::ID,
        sysvar::rent::ID,
        addresses.amm,
        addresses.authority,
        addresses.lp_mint,
        accounts.coin_mint.key,
        accounts.pc_mint.key,
        addresses.coin_vault,
        addresses.pc_vault,
        addresses.target_orders,
        addresses.config,
        addresses.create_pool_fee_destination,
        market_seed,
        wallet,
        accounts.user_coin_source,
        accounts.user_pc_source,
        accounts.user_lp_ata,
    ];
    let writable = [
        false, false, false, false, true, false, true, false, false, true, true, true, false, true,
        false, true, true, true, true,
    ];
    assert_eq!(expected.accounts.len(), INITIALIZE2_ACCOUNT_COUNT);
    for (i, meta) in expected.accounts.iter().enumerate() {
        assert_eq!(meta.pubkey, keys[i], "account {i} key");
        assert_eq!(meta.is_writable, writable[i], "account {i} writable");
        assert_eq!(meta.is_signer, i == 15, "account {i} signer");
    }
    assert_eq!(
        expected.data,
        [
            1,
            addresses.nonce,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            200,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            44,
            1,
            0,
            0,
            0,
            0,
            0,
            0
        ]
    );

    let mut bank = ProgramTest::new(
        "synthetic_migration_caller",
        CALLER_ID,
        processor!(synthetic_caller),
    );
    bank.add_program(
        "local_mock_amm_v4",
        DEVNET_PROGRAM_ID,
        processor!(mock_amm_v4),
    );
    for (i, meta) in expected.accounts.iter().enumerate() {
        if meta.pubkey == system_program::ID || meta.pubkey == sysvar::rent::ID {
            continue; // native ProgramTest entries
        }
        let account = match i {
            4 => dummy(DEVNET_PROGRAM_ID, 1_000_000, vec![0; 8]),
            7 | 8 => dummy(SPL_TOKEN_PROGRAM_ID, 1_000_000, vec![0; 82]),
            12 => {
                let mut data = vec![0; AMM_CONFIG_DATA_LEN];
                data[536..544].copy_from_slice(&create_fee.to_le_bytes());
                if truncated_config {
                    data.truncate(AMM_CONFIG_DATA_LEN - 1);
                }
                dummy(config_owner, 1_000_000, data)
            }
            13 => dummy(system_program::ID, START_FEE_DESTINATION, Vec::new()),
            15 => dummy(system_program::ID, START_WALLET, Vec::new()),
            16..=18 => dummy(SPL_TOKEN_PROGRAM_ID, 1_000_000, vec![0; 8]),
            _ => dummy(system_program::ID, 1_000_000, Vec::new()),
        };
        bank.add_account(meta.pubkey, account);
    }
    Fixture {
        bank: bank.start_with_context().await,
        expected,
        amm: addresses.amm,
        fee_destination: addresses.create_pool_fee_destination,
    }
}

fn call(f: &Fixture, mode: u8, cap: u64, open_time: u64, corrupt_order: bool) -> Transaction {
    let mut metas = f.expected.accounts.clone();
    metas[15].is_signer = false; // only the caller can sign this PDA, via invoke_signed
    if corrupt_order {
        metas.swap(4, 6);
    }
    metas.push(AccountMeta::new_readonly(DEVNET_PROGRAM_ID, false));
    let mut data = vec![mode];
    data.extend_from_slice(&cap.to_le_bytes());
    data.extend_from_slice(&open_time.to_le_bytes());
    data.extend_from_slice(&200u64.to_le_bytes());
    data.extend_from_slice(&300u64.to_le_bytes());
    let ix = Instruction {
        program_id: CALLER_ID,
        accounts: metas,
        data,
    };
    Transaction::new_signed_with_payer(
        &[ix],
        Some(&f.bank.payer.pubkey()),
        &[&f.bank.payer],
        f.bank.last_blockhash,
    )
}

async fn balances(f: &Fixture) -> (u64, u64, Vec<u8>) {
    let wallet = f
        .bank
        .banks_client
        .get_account(wallet_pda().0)
        .await
        .unwrap()
        .unwrap();
    let fee = f
        .bank
        .banks_client
        .get_account(f.fee_destination)
        .await
        .unwrap()
        .unwrap();
    let amm = f
        .bank
        .banks_client
        .get_account(f.amm)
        .await
        .unwrap()
        .unwrap();
    (wallet.lamports, fee.lamports, amm.data)
}

#[tokio::test]
async fn exact_abi_pda_signature_and_mock_fee_transfer() {
    let f = fixture(DEVNET_PROGRAM_ID, MOCK_CREATE_FEE, false).await;
    assert_eq!(
        balances(&f).await,
        (START_WALLET, START_FEE_DESTINATION, vec![0; 8])
    );
    f.bank
        .banks_client
        .process_transaction(call(&f, 0, MOCK_CREATE_FEE, 1, false))
        .await
        .unwrap();
    assert_eq!(
        balances(&f).await,
        (
            START_WALLET - MOCK_CREATE_FEE,
            START_FEE_DESTINATION + MOCK_CREATE_FEE,
            vec![1, 0, 0, 0, 0, 0, 0, 0]
        )
    );
}

#[tokio::test]
async fn malformed_order_config_fee_and_missing_signature_leave_state_unchanged() {
    for (owner, fee, mode, cap, corrupt_order, truncated_config) in [
        (
            DEVNET_PROGRAM_ID,
            MOCK_CREATE_FEE,
            0,
            MOCK_CREATE_FEE,
            true,
            false,
        ),
        (
            system_program::ID,
            MOCK_CREATE_FEE,
            0,
            MOCK_CREATE_FEE,
            false,
            false,
        ),
        (
            DEVNET_PROGRAM_ID,
            MOCK_CREATE_FEE,
            0,
            MOCK_CREATE_FEE - 1,
            false,
            false,
        ),
        (
            DEVNET_PROGRAM_ID,
            MOCK_CREATE_FEE,
            0,
            MOCK_CREATE_FEE,
            false,
            true,
        ),
        (
            DEVNET_PROGRAM_ID,
            MOCK_CREATE_FEE,
            2,
            MOCK_CREATE_FEE,
            false,
            false,
        ),
        (
            DEVNET_PROGRAM_ID,
            MOCK_CREATE_FEE,
            3,
            MOCK_CREATE_FEE,
            false,
            false,
        ),
    ] {
        let f = fixture(owner, fee, truncated_config).await;
        let error = f
            .bank
            .banks_client
            .process_transaction(call(&f, mode, cap, 1, corrupt_order))
            .await
            .unwrap_err();
        let expected_error = match mode {
            2 => "MissingRequiredSignature",
            3 => "InvalidInstructionData",
            _ => "InvalidAccountData",
        };
        assert!(
            format!("{error:?}").contains(expected_error),
            "unexpected rejection for mode={mode} cap={cap} order={corrupt_order} truncated={truncated_config}: {error:?}"
        );
        assert_eq!(
            balances(&f).await,
            (START_WALLET, START_FEE_DESTINATION, vec![0; 8])
        );
    }
}

#[tokio::test]
async fn mock_late_error_and_caller_post_cpi_error_roll_back_balances_and_pool_write() {
    for (mode, open_time, expected_code) in [(0, 2, MOCK_LATE_ERROR), (1, 1, MOCK_POST_CPI_ERROR)] {
        let f = fixture(DEVNET_PROGRAM_ID, MOCK_CREATE_FEE, false).await;
        let error = f
            .bank
            .banks_client
            .process_transaction(call(&f, mode, MOCK_CREATE_FEE, open_time, false))
            .await
            .unwrap_err();
        assert!(
            format!("{error:?}").contains(&format!("Custom({expected_code})")),
            "{error:?}"
        );
        assert_eq!(
            balances(&f).await,
            (START_WALLET, START_FEE_DESTINATION, vec![0; 8])
        );
    }
}
