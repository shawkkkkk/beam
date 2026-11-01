//! Local-bank activation and rollback checks for the Beam Raydium gate.
//! This test never invokes Raydium and does not establish migration CPI safety.
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

fn beam_processor<'a>(program_id: &Pubkey, accounts: &[AccountInfo<'a>], data: &[u8]) -> ProgramResult {
    // SAFETY: ProgramTest invokes this processor synchronously. Anchor's
    // generated entrypoint requires the slice borrow to share AccountInfo's
    // lifetime; neither Beam nor ProgramTest retains the slice after this
    // call, and ProgramResult contains no borrowed data.
    let accounts: &'a [AccountInfo<'a>] = unsafe { std::mem::transmute(accounts) };
    beam_program::entry(program_id, accounts, data)
}

async fn fixture() -> (ProgramTestContext, Keypair, Keypair) {
    let admin = Keypair::new();
    let other = Keypair::new();

    // Global has retained historical fields. Set the initialized flag and
    // authority; zeroed remaining fields are valid Borsh values. Extra bytes
    // are harmless account padding.
    let mut global_data = vec![0u8; 2048];
    global_data[..8].copy_from_slice(Global::DISCRIMINATOR);
    global_data[8] = 1;
    global_data[9..41].copy_from_slice(admin.pubkey().as_ref());
    assert_eq!(Global::try_deserialize(&mut &global_data[..]).unwrap().authority, admin.pubkey());
    let mut config_data = Vec::new();
    BeamRaydiumConfig {
        amm_config: Pubkey::new_unique(),
        raydium_program: RAYDIUM_ID,
        enabled: false,
    }
    .try_serialize(&mut config_data)
    .unwrap();

    let mut bank = ProgramTest::new("beam_program", beam_program::ID, processor!(beam_processor));
    bank.add_account(global_pda(), Account {
        lamports: 100_000_000,
        data: global_data,
        owner: beam_program::ID,
        executable: false,
        rent_epoch: 0,
    });
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

async fn config_enabled(bank: &mut ProgramTestContext) -> bool {
    let account = bank.banks_client.get_account(config_pda()).await.unwrap().unwrap();
    BeamRaydiumConfig::try_deserialize(&mut &account.data[..]).unwrap().enabled
}

fn signed_tx(bank: &ProgramTestContext, admin: &Keypair, other: Option<&Keypair>, instructions: &[Instruction]) -> Transaction {
    let mut signers: Vec<&Keypair> = vec![&bank.payer, admin];
    if let Some(other) = other { signers.push(other); }
    Transaction::new_signed_with_payer(
        instructions,
        Some(&bank.payer.pubkey()),
        &signers,
        bank.last_blockhash,
    )
}

#[cfg(not(feature = "experimental-raydium"))]
#[tokio::test]
async fn default_build_cannot_enable_migration_and_keeps_config_disabled() {
    let (mut bank, admin, _) = fixture().await;
    assert!(!config_enabled(&mut bank).await);
    let attempt = signed_tx(&bank, &admin, None, &[set_enabled(admin.pubkey(), true)]);
    let error = bank.banks_client.process_transaction(attempt).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(7002)"), "unexpected gate error: {error:?}");
    assert!(!config_enabled(&mut bank).await);

    // Disabling remains available without the experimental feature.
    let disable = signed_tx(&bank, &admin, None, &[set_enabled(admin.pubkey(), false)]);
    bank.banks_client.process_transaction(disable).await.unwrap();
    assert!(!config_enabled(&mut bank).await);
}

#[cfg(feature = "experimental-raydium")]
#[tokio::test]
async fn failed_second_instruction_rolls_back_experimental_enable() {
    let (mut bank, admin, other) = fixture().await;
    assert!(!config_enabled(&mut bank).await);

    // The first instruction would enable the gate, but the second uses a
    // signer that is not Global.authority. The bank must roll back both.
    let attempted_pair = signed_tx(&bank, &admin, Some(&other), &[
        set_enabled(admin.pubkey(), true),
        set_enabled(other.pubkey(), false),
    ]);
    let error = bank.banks_client.process_transaction(attempted_pair).await.unwrap_err();
    assert!(format!("{error:?}").contains("Custom(2001)"), "unexpected authority error: {error:?}");
    assert!(!config_enabled(&mut bank).await);

    let authorized = signed_tx(&bank, &admin, None, &[set_enabled(admin.pubkey(), true)]);
    bank.banks_client.process_transaction(authorized).await.unwrap();
    assert!(config_enabled(&mut bank).await);
}
