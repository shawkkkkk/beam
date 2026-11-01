//! Local-bank gate, authority, and rollback checks for the retired CPMM adapter.
//! The user selected AMM v4. These synthetic transactions never invoke the
//! Raydium program, do not test a migration CPI, and do not establish safety
//! or behavior of any deployed Raydium binary.
use anchor_lang::{AccountDeserialize, AccountSerialize, Discriminator, InstructionData};
use anchor_lang::solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use beam_program::{beam_raydium::{BeamRaydiumConfig, RAYDIUM_ID}, Global};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

fn global_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"global"], &beam_program::ID).0
}

fn config_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"beam-raydium-config"], &beam_program::ID).0
}

fn set_enabled(authority: Pubkey, enabled: bool) -> Instruction {
    Instruction {
        program_id: beam_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(global_pda(), false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(config_pda(), false),
        ],
        data: beam_program::instruction::SetRaydiumEnabled { enabled }.data(),
    }
}

fn configure(authority: Pubkey, amm_config: Pubkey) -> Instruction {
    Instruction {
        program_id: beam_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(global_pda(), false),
            AccountMeta::new(authority, true),
            AccountMeta::new(config_pda(), false),
            AccountMeta::new_readonly(amm_config, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: beam_program::instruction::ConfigureRaydium {}.data(),
    }
}

fn synthetic_global(authority: Pubkey) -> Account {
    // Global has retained historical fields. Set the initialized flag and
    // authority; zeroed remaining fields are valid Borsh values. Extra bytes
    // are harmless account padding.
    let mut data = vec![0u8; 2048];
    data[..8].copy_from_slice(Global::DISCRIMINATOR);
    data[8] = 1;
    data[9..41].copy_from_slice(authority.as_ref());
    assert_eq!(Global::try_deserialize(&mut &data[..]).unwrap().authority, authority);
    Account {
        lamports: 100_000_000,
        data,
        owner: beam_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

fn beam_processor<'a>(program_id: &Pubkey, accounts: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    // SAFETY: ProgramTest invokes this processor synchronously. Anchor's
    // generated entrypoint requires the slice borrow to share AccountInfo's
    // lifetime; neither Beam nor ProgramTest retains the slice after this
    // call, and ProgramResult contains no borrowed data.
    let accounts: &'a [AccountInfo<'a>] = unsafe { std::mem::transmute(accounts) };
    beam_program::entry(program_id, accounts, data)
}

async fn fixture(initially_enabled: bool) -> (ProgramTestContext, Keypair, Keypair) {
    let admin = Keypair::new();
    let other = Keypair::new();
    let mut config_data = Vec::new();
    BeamRaydiumConfig {
        amm_config: Pubkey::new_unique(),
        raydium_program: RAYDIUM_ID,
        enabled: initially_enabled,
    }
    .try_serialize(&mut config_data)
    .unwrap();

    let mut bank = ProgramTest::new("beam_program", beam_program::ID, processor!(beam_processor));
    bank.add_account(global_pda(), synthetic_global(admin.pubkey()));
    bank.add_account(config_pda(), Account {
        lamports: 100_000_000,
        data: config_data,
        owner: beam_program::ID,
        executable: false,
        rent_epoch: 0,
    });
    for signer in [&admin, &other] {
        bank.add_account(signer.pubkey(), Account {
            lamports: 1_000_000_000,
            data: Vec::new(),
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        });
    }
    (bank.start_with_context().await, admin, other)
}

async fn configure_fixture(disable_create_pool: bool) -> (ProgramTestContext, Keypair, Pubkey) {
    let admin = Keypair::new();
    let index = 7u16;
    let amm_config = Pubkey::find_program_address(&[b"amm_config", &index.to_be_bytes()], &RAYDIUM_ID).0;
    let mut data = vec![0u8; 236];
    data[..8].copy_from_slice(&anchor_lang::solana_program::hash::hash(b"account:AmmConfig").to_bytes()[..8]);
    data[9] = u8::from(disable_create_pool);
    data[10..12].copy_from_slice(&index.to_le_bytes());
    let mut bank = ProgramTest::new("beam_program", beam_program::ID, processor!(beam_processor));
    bank.add_account(global_pda(), synthetic_global(admin.pubkey()));
    bank.add_account(amm_config, Account {
        lamports: 100_000_000,
        data,
        owner: RAYDIUM_ID,
        executable: false,
        rent_epoch: 0,
    });
    bank.add_account(admin.pubkey(), Account {
        lamports: 1_000_000_000,
        data: Vec::new(),
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    });
    (bank.start_with_context().await, admin, amm_config)
}

async fn config_enabled(bank: &mut ProgramTestContext) -> bool {
    let account = bank.banks_client.get_account(config_pda()).await.unwrap().unwrap();
    BeamRaydiumConfig::try_deserialize(&mut &account.data[..]).unwrap().enabled
}

#[tokio::test]
async fn invalid_amm_config_rolls_back_account_init_and_valid_config_starts_disabled() {
    let (mut rejected_bank, admin, amm_config) = configure_fixture(true).await;
    let rejected = signed_tx(&rejected_bank, &admin, &[configure(admin.pubkey(), amm_config)]);
    let error = rejected_bank.banks_client.process_transaction(rejected).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(7003)"), "unexpected config error: {error:?}");
    assert!(rejected_bank.banks_client.get_account(config_pda()).await.unwrap().is_none());

    let (mut valid_bank, admin, amm_config) = configure_fixture(false).await;
    let valid = signed_tx(&valid_bank, &admin, &[configure(admin.pubkey(), amm_config)]);
    valid_bank.banks_client.process_transaction(valid).await.unwrap();
    let account = valid_bank.banks_client.get_account(config_pda()).await.unwrap().unwrap();
    let config = BeamRaydiumConfig::try_deserialize(&mut &account.data[..]).unwrap();
    assert_eq!(config.amm_config, amm_config);
    assert_eq!(config.raydium_program, RAYDIUM_ID);
    assert!(!config.enabled);
}

fn signed_tx(bank: &ProgramTestContext, admin: &Keypair, instructions: &[Instruction]) -> Transaction {
    Transaction::new_signed_with_payer(
        instructions,
        Some(&bank.payer.pubkey()),
        &[&bank.payer, admin],
        bank.last_blockhash,
    )
}

#[tokio::test]
async fn retired_cpmm_cannot_enable_in_any_feature_mode_and_keeps_config_disabled() {
    let (mut bank, admin, other) = fixture(false).await;
    assert!(!config_enabled(&mut bank).await);
    let attempt = signed_tx(&bank, &admin, &[set_enabled(admin.pubkey(), true)]);
    let error = bank.banks_client.process_transaction(attempt).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(7002)"), "unexpected gate error: {error:?}");
    assert!(!config_enabled(&mut bank).await);

    let unauthorized = Transaction::new_signed_with_payer(
        &[set_enabled(other.pubkey(), false)],
        Some(&bank.payer.pubkey()),
        &[&bank.payer, &other],
        bank.last_blockhash,
    );
    let error = bank.banks_client.process_transaction(unauthorized).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(2001)"), "unexpected authority error: {error:?}");
    assert!(!config_enabled(&mut bank).await);

    // An authorized disable remains idempotent for a retired adapter.
    let disable = signed_tx(&bank, &admin, &[set_enabled(admin.pubkey(), false)]);
    bank.banks_client.process_transaction(disable).await.unwrap();
    assert!(!config_enabled(&mut bank).await);
}

#[tokio::test]
async fn failed_enable_rolls_back_prior_disable_of_legacy_state() {
    // Synthetic legacy state represents a config enabled before CPMM retirement.
    let (mut bank, admin, _) = fixture(true).await;
    assert!(config_enabled(&mut bank).await);

    // The first instruction disables; the second attempts to re-enable and
    // fails. The bank rolls back the entire transaction, including disable.
    let attempted_pair = signed_tx(&bank, &admin, &[
        set_enabled(admin.pubkey(), false),
        set_enabled(admin.pubkey(), true),
    ]);
    let error = bank.banks_client.process_transaction(attempted_pair).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(7002)"), "unexpected gate error: {error:?}");
    assert!(config_enabled(&mut bank).await);

    let disable = signed_tx(&bank, &admin, &[set_enabled(admin.pubkey(), false)]);
    bank.banks_client.process_transaction(disable).await.unwrap();
    assert!(!config_enabled(&mut bank).await);
}
