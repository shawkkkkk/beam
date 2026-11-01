//! Synthetic local-bank checks for the disabled FeePolicyV1 initializer.
//! These transactions use ephemeral keypairs and valueless test lamports.
//! They do not execute captured Pump or Raydium binaries or submit live RPC.
use anchor_lang::solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use anchor_lang::{AccountDeserialize, Discriminator, InstructionData};
use beam_program::{
    beam_fee_policy_v1::{
        BeamFeePolicyV1, BeamFeeVaultV1, CreatorFunding, PolicyStatus, SurchargeStatus,
        ORDINARY_BEAM_TOTAL_BPS, ORDINARY_COMMUNITY_BPS, ORDINARY_FOUNDER_BPS,
        ORDINARY_PLATFORM_BPS, ORDINARY_TREASURY_BPS, POLICY_SPACE, POLICY_VERSION, VAULT_SPACE,
    },
    Global,
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};

fn global_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"global"], &beam_program::ID).0
}

fn policy_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"beam-fee-policy", b"v1"], &beam_program::ID)
}

fn vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"beam-fee-vault", b"v1"], &beam_program::ID)
}

fn initialize(authority: Pubkey) -> Instruction {
    Instruction {
        program_id: beam_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(global_pda(), false),
            AccountMeta::new(authority, true),
            AccountMeta::new(policy_pda().0, false),
            AccountMeta::new(vault_pda().0, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: beam_program::instruction::InitializeBeamFeePolicyV1 {}.data(),
    }
}

fn synthetic_global(authority: Pubkey) -> Account {
    // The retained Global layout begins with the initialized flag and
    // authority. The remaining zeroed fixed fields deserialize in this test.
    let mut data = vec![0u8; 2048];
    data[..8].copy_from_slice(Global::DISCRIMINATOR);
    data[8] = 1;
    data[9..41].copy_from_slice(authority.as_ref());
    assert_eq!(
        Global::try_deserialize(&mut &data[..]).unwrap().authority,
        authority
    );
    Account {
        lamports: 100_000_000,
        data,
        owner: beam_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

fn beam_processor<'a>(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'a>],
    data: &[u8],
) -> ProgramResult {
    // SAFETY: ProgramTest calls this synchronously. Neither the Anchor entry
    // nor ProgramTest retains the slice; ProgramResult borrows no accounts.
    let accounts: &'a [AccountInfo<'a>] = unsafe { std::mem::transmute(accounts) };
    beam_program::entry(program_id, accounts, data)
}

async fn fixture() -> (ProgramTestContext, Keypair, Keypair) {
    let admin = Keypair::new();
    let other = Keypair::new();
    let mut bank = ProgramTest::new("beam_program", beam_program::ID, processor!(beam_processor));
    bank.add_account(global_pda(), synthetic_global(admin.pubkey()));
    for signer in [&admin, &other] {
        bank.add_account(
            signer.pubkey(),
            Account {
                lamports: 1_000_000_000,
                data: Vec::new(),
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }
    (bank.start_with_context().await, admin, other)
}

fn signed_tx(
    bank: &ProgramTestContext,
    authority: &Keypair,
    instructions: &[Instruction],
) -> Transaction {
    Transaction::new_signed_with_payer(
        instructions,
        Some(&bank.payer.pubkey()),
        &[&bank.payer, authority],
        bank.last_blockhash,
    )
}

#[tokio::test]
async fn authority_init_replay_rent_and_donation_are_bounded() {
    let (bank, admin, other) = fixture().await;
    let policy_key = policy_pda().0;
    let vault_key = vault_pda().0;
    let admin_before = bank
        .banks_client
        .get_account(admin.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let other_before = bank
        .banks_client
        .get_account(other.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let unauthorized = signed_tx(&bank, &other, &[initialize(other.pubkey())]);
    let error = bank
        .banks_client
        .process_transaction(unauthorized)
        .await
        .unwrap_err();
    assert!(
        format!("{error:?}").contains("Custom(6000)"),
        "unexpected authority error: {error:?}"
    );
    assert!(bank
        .banks_client
        .get_account(policy_key)
        .await
        .unwrap()
        .is_none());
    assert!(bank
        .banks_client
        .get_account(vault_key)
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        bank.banks_client
            .get_account(admin.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports,
        admin_before
    );
    assert_eq!(
        bank.banks_client
            .get_account(other.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports,
        other_before
    );

    let authorized = signed_tx(&bank, &admin, &[initialize(admin.pubkey())]);
    bank.banks_client
        .process_transaction(authorized)
        .await
        .unwrap();
    let policy_account = bank
        .banks_client
        .get_account(policy_key)
        .await
        .unwrap()
        .unwrap();
    let vault_account = bank
        .banks_client
        .get_account(vault_key)
        .await
        .unwrap()
        .unwrap();
    let policy = BeamFeePolicyV1::try_deserialize(&mut &policy_account.data[..]).unwrap();
    let vault = BeamFeeVaultV1::try_deserialize(&mut &vault_account.data[..]).unwrap();
    assert_eq!(policy_account.owner, beam_program::ID);
    assert_eq!(vault_account.owner, beam_program::ID);
    assert_eq!(
        (policy_account.data.len(), vault_account.data.len()),
        (POLICY_SPACE, VAULT_SPACE)
    );
    assert_eq!(
        (policy.version, vault.version),
        (POLICY_VERSION, POLICY_VERSION)
    );
    assert_eq!(
        (
            policy.status,
            policy.creator_funding,
            policy.launch_surcharge
        ),
        (
            PolicyStatus::Disabled,
            CreatorFunding::Undecided,
            SurchargeStatus::Disabled
        )
    );
    assert_eq!(policy.vault, vault_key);
    assert_eq!(vault.policy, policy_key);
    assert_eq!(
        (policy.policy_bump, policy.vault_bump, vault.bump),
        (policy_pda().1, vault_pda().1, vault_pda().1)
    );
    assert_eq!(
        (
            policy.ordinary_total_bps,
            policy.ordinary_community_bps,
            policy.ordinary_treasury_bps,
            policy.ordinary_founder_bps
        ),
        (
            ORDINARY_BEAM_TOTAL_BPS,
            ORDINARY_COMMUNITY_BPS,
            ORDINARY_TREASURY_BPS,
            ORDINARY_FOUNDER_BPS
        )
    );
    assert_eq!(ORDINARY_PLATFORM_BPS, 8);
    assert_eq!(vault.credited_fee_lamports, 0);

    let rent = bank.banks_client.get_rent().await.unwrap();
    assert_eq!(policy_account.lamports, rent.minimum_balance(POLICY_SPACE));
    assert_eq!(vault_account.lamports, rent.minimum_balance(VAULT_SPACE));
    let admin_after = bank
        .banks_client
        .get_account(admin.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(
        admin_before - admin_after,
        policy_account.lamports + vault_account.lamports
    );

    // A new signature reaches the initializer rather than the duplicate
    // transaction cache. Anchor rejects the already initialized canonical PDA.
    let duplicate = signed_tx(
        &bank,
        &admin,
        &[
            ComputeBudgetInstruction::set_compute_unit_limit(400_000),
            initialize(admin.pubkey()),
        ],
    );
    let duplicate_error = bank
        .banks_client
        .process_transaction(duplicate)
        .await
        .unwrap_err();
    assert!(
        format!("{duplicate_error:?}").contains("Custom(0)"),
        "unexpected duplicate initializer error: {duplicate_error:?}"
    );
    assert_eq!(
        bank.banks_client
            .get_account(policy_key)
            .await
            .unwrap()
            .unwrap(),
        policy_account
    );
    assert_eq!(
        bank.banks_client
            .get_account(vault_key)
            .await
            .unwrap()
            .unwrap(),
        vault_account
    );
    assert_eq!(
        bank.banks_client
            .get_account(admin.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports,
        admin_after
    );

    // System transfer can donate real local-bank lamports to the program-owned
    // vault, but does not write a Beam credit or enable quoting/withdrawal.
    const DONATION: u64 = 10_000;
    let donation = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &bank.payer.pubkey(),
            &vault_key,
            DONATION,
        )],
        Some(&bank.payer.pubkey()),
        &[&bank.payer],
        bank.last_blockhash,
    );
    bank.banks_client
        .process_transaction(donation)
        .await
        .unwrap();
    let donated = bank
        .banks_client
        .get_account(vault_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(donated.lamports, vault_account.lamports + DONATION);
    assert_eq!(donated.data, vault_account.data);
    let donated_vault = BeamFeeVaultV1::try_deserialize(&mut &donated.data[..]).unwrap();
    assert_eq!(donated_vault.credited_fee_lamports, 0);
    assert_eq!(
        bank.banks_client
            .get_account(policy_key)
            .await
            .unwrap()
            .unwrap(),
        policy_account
    );
}
