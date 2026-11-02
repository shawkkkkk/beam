//! Local banks execution of the pinned Raydium AMM v4 `Initialize2` processor.
//!
//! These tests are the migration-validation evidence for task scope 3. They do
//! not activate a Beam migration instruction.

use beam_amm_v4_validator::{
    derive_pool, expected_lp, id, pinned_config_bytes, process_instruction, process_raydium,
    MigrateInput, CURVE_LAUNCH_PREFIX, ERR_PRECREATED, ERR_ROLLBACK, SOURCE_COMMIT,
    SOURCE_REPOSITORY, SYSTEM_PROGRAM_ID, TAG_FORWARD, TAG_MIGRATE, TAG_MIGRATE_THEN_ABORT,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program::program_pack::Pack;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_pubkey::Pubkey;
use solana_rent::Rent;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token::state::{Account as TokenAccount, AccountState, Mint};

const INIT_COIN: u64 = 1_000_000_000_000;
const INIT_PC: u64 = 1_000_000_000_000;
const COIN_RETAINED: u64 = 123_456;
const PC_LIABILITY: u64 = 42_000_000;
const PC_DONATION: u64 = 777;
const USER_COIN_DONATION: u64 = 50;
const USER_PC_DONATION: u64 = 80;
const CREATE_FEE: u64 = 25_000_000;
const FEE_DEST_DONATION: u64 = 999;
const COIN_DECIMALS: u8 = 6;
const TOKEN_2022: Pubkey = solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

struct Fixture {
    market: Pubkey,
    coin_mint: Pubkey,
    pc_mint: Pubkey,
    curve_coin: Pubkey,
    curve_pc: Pubkey,
    curve_launch: Pubkey,
    user_coin: Pubkey,
    user_pc: Pubkey,
    launch: Vec<u8>,
}

fn rent() -> Rent {
    Rent::default()
}

fn mint_account(owner: &Pubkey, decimals: u8, supply: u64) -> Account {
    let mut mint = Mint::default();
    mint.decimals = decimals;
    mint.is_initialized = true;
    mint.supply = supply;
    let mut data = vec![0; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    Account {
        lamports: rent().minimum_balance(data.len()),
        data,
        owner: *owner,
        executable: false,
        rent_epoch: 0,
    }
}

fn token_account(
    mint: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    native_reserve: Option<u64>,
) -> Account {
    let mut token = TokenAccount::default();
    token.mint = *mint;
    token.owner = *owner;
    token.amount = amount;
    token.state = AccountState::Initialized;
    token.is_native = native_reserve.into();
    let mut data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(token, &mut data).unwrap();
    let lamports = match native_reserve {
        Some(reserve) => reserve + amount,
        None => rent().minimum_balance(data.len()),
    };
    Account {
        lamports,
        data,
        owner: spl_token::id(),
        executable: false,
        rent_epoch: 0,
    }
}

fn launch_bytes(mint: &Pubkey, kind: u8) -> Vec<u8> {
    let mut data = CURVE_LAUNCH_PREFIX.to_vec();
    data.extend_from_slice(mint.as_ref());
    data.push(kind);
    data.extend_from_slice(b"preserved-curve-launch");
    data
}

fn fixture(token_2022_coin: bool) -> Fixture {
    let market = Pubkey::new_unique();
    let coin_mint = Pubkey::new_unique();
    let pc_mint = spl_token::native_mint::id();
    let curve_coin = Pubkey::new_unique();
    let curve_pc = Pubkey::new_unique();
    let curve_launch = Pubkey::new_unique();
    let user_coin = Pubkey::new_unique();
    let user_pc = Pubkey::new_unique();
    Fixture {
        market,
        coin_mint,
        pc_mint,
        curve_coin,
        curve_pc,
        curve_launch,
        user_coin,
        user_pc,
        launch: launch_bytes(&coin_mint, if token_2022_coin { 2 } else { 1 }),
    }
}

fn add_market(pt: &mut ProgramTest, fixture: &Fixture, token_2022_coin: bool) {
    let pool = derive_pool(&fixture.market);
    pt.add_account(
        pool.authority,
        Account {
            lamports: 5_000_000_000,
            data: vec![],
            owner: SYSTEM_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    pt.add_account(
        fixture.market,
        Account {
            lamports: rent().minimum_balance(0),
            data: vec![],
            owner: SYSTEM_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let coin_owner = if token_2022_coin {
        TOKEN_2022
    } else {
        spl_token::id()
    };
    pt.add_account(
        fixture.coin_mint,
        mint_account(
            &coin_owner,
            COIN_DECIMALS,
            INIT_COIN + COIN_RETAINED + USER_COIN_DONATION,
        ),
    );
    pt.add_account(
        fixture.curve_coin,
        token_account(
            &fixture.coin_mint,
            &pool.authority,
            INIT_COIN + COIN_RETAINED,
            None,
        ),
    );
    pt.add_account(
        fixture.curve_pc,
        token_account(
            &fixture.pc_mint,
            &pool.authority,
            INIT_PC + PC_LIABILITY + PC_DONATION,
            None,
        ),
    );
    pt.add_account(
        fixture.user_coin,
        token_account(
            &fixture.coin_mint,
            &pool.authority,
            USER_COIN_DONATION,
            None,
        ),
    );
    pt.add_account(
        fixture.user_pc,
        token_account(&fixture.pc_mint, &pool.authority, USER_PC_DONATION, None),
    );
    pt.add_account(
        fixture.curve_launch,
        Account {
            lamports: rent().minimum_balance(fixture.launch.len()),
            data: fixture.launch.clone(),
            owner: id(),
            executable: false,
            rent_epoch: 0,
        },
    );
}

fn base_test() -> ProgramTest {
    let mut pt = ProgramTest::new("beam_amm_v4_probe", id(), processor!(process_instruction));
    pt.prefer_bpf(false);
    pt.set_compute_max_units(1_400_000);
    pt.add_program(
        "raydium_amm",
        raydium_amm::id(),
        processor!(process_raydium),
    );
    let sample = derive_pool(&Pubkey::new_unique());
    let config = pinned_config_bytes(
        &raydium_amm::processor::config_feature::amm_owner::id(),
        &raydium_amm::processor::config_feature::amm_owner::id(),
        CREATE_FEE,
    );
    pt.add_account(
        sample.config,
        Account {
            lamports: rent().minimum_balance(config.len()),
            data: config,
            owner: raydium_amm::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    let reserve = rent().minimum_balance(TokenAccount::LEN);
    pt.add_account(
        sample.fee_destination,
        token_account(
            &spl_token::native_mint::id(),
            &Pubkey::new_unique(),
            FEE_DEST_DONATION,
            Some(reserve),
        ),
    );
    pt.add_account(
        spl_token::native_mint::id(),
        mint_account(&spl_token::id(), 9, 0),
    );
    pt
}

fn instruction(fixture: &Fixture, input: MigrateInput) -> Instruction {
    let pool = derive_pool(&fixture.market);
    let keys = [
        (pool.authority, true),
        (fixture.curve_coin, true),
        (fixture.curve_pc, true),
        (fixture.curve_launch, false),
        (fixture.user_coin, true),
        (fixture.user_pc, true),
        (pool.user_lp, true),
        (pool.amm, true),
        (pool.amm_authority, false),
        (pool.lp_mint, true),
        (fixture.coin_mint, false),
        (fixture.pc_mint, false),
        (pool.coin_vault, true),
        (pool.pc_vault, true),
        (pool.target_orders, true),
        (pool.config, false),
        (pool.fee_destination, true),
        (fixture.market, false),
        (spl_token::id(), false),
        (spl_associated_token_account::id(), false),
        (SYSTEM_PROGRAM_ID, false),
        (solana_program::sysvar::rent::ID, false),
        (raydium_amm::id(), false),
    ];
    assert_eq!(keys.len(), 23);
    Instruction {
        program_id: id(),
        accounts: keys
            .into_iter()
            .map(|(pubkey, writable)| {
                if writable {
                    AccountMeta::new(pubkey, false)
                } else {
                    AccountMeta::new_readonly(pubkey, false)
                }
            })
            .collect(),
        data: input.encode().to_vec(),
    }
}

fn input(tag: u8) -> MigrateInput {
    MigrateInput {
        tag,
        init_coin: INIT_COIN,
        init_pc: INIT_PC,
        coin_retained: COIN_RETAINED,
        pc_liability: PC_LIABILITY,
        pc_donation: PC_DONATION,
        open_time: 0,
    }
}

async fn send(ctx: &mut ProgramTestContext, ix: Instruction) -> Result<(), String> {
    let blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        blockhash,
    );
    ctx.banks_client
        .process_transaction(tx)
        .await
        .map_err(|err| format!("{err:?}"))
}

async fn account(ctx: &mut ProgramTestContext, key: &Pubkey) -> Option<Account> {
    ctx.banks_client.get_account(*key).await.unwrap()
}

async fn amount(ctx: &mut ProgramTestContext, key: &Pubkey) -> u64 {
    TokenAccount::unpack(&account(ctx, key).await.unwrap().data)
        .unwrap()
        .amount
}

#[tokio::test]
async fn initialize2_cpi_checks_pool_reserves_fees_liabilities_and_donations() {
    println!("pinned AMM v4 source: {SOURCE_REPOSITORY}@{SOURCE_COMMIT}");
    let fixture = fixture(false);
    let pool = derive_pool(&fixture.market);
    println!(
        "program {} config {} fee destination {} create_pool_fee {CREATE_FEE}",
        raydium_amm::id(),
        pool.config,
        pool.fee_destination
    );
    let mut pt = base_test();
    add_market(&mut pt, &fixture, false);
    let mut ctx = pt.start_with_context().await;
    let fee_before = account(&mut ctx, &pool.fee_destination).await.unwrap();
    send(&mut ctx, instruction(&fixture, input(TAG_MIGRATE)))
        .await
        .expect("pinned Initialize2 CPI");

    let amm = account(&mut ctx, &pool.amm).await.expect("pool account");
    assert_eq!(amm.owner, raydium_amm::id());
    assert_eq!(
        amm.data.len(),
        std::mem::size_of::<raydium_amm::state::AmmInfo>()
    );
    assert_eq!(&amm.data[..8], &6u64.to_le_bytes());
    assert_eq!(amount(&mut ctx, &pool.coin_vault).await, INIT_COIN);
    assert_eq!(amount(&mut ctx, &pool.pc_vault).await, INIT_PC);
    let (liquidity, user_lp) = expected_lp(INIT_COIN, INIT_PC, COIN_DECIMALS).unwrap();
    assert_eq!(liquidity, 1_000_000_000_000);
    assert_eq!(user_lp, 999_999_000_000);
    assert_eq!(amount(&mut ctx, &pool.user_lp).await, user_lp);
    let lp = Mint::unpack(&account(&mut ctx, &pool.lp_mint).await.unwrap().data).unwrap();
    assert_eq!(lp.supply, user_lp);
    assert_eq!(lp.decimals, COIN_DECIMALS);
    assert_eq!(amount(&mut ctx, &fixture.curve_coin).await, COIN_RETAINED);
    assert_eq!(
        amount(&mut ctx, &fixture.curve_pc).await,
        PC_LIABILITY + PC_DONATION
    );
    assert_eq!(
        amount(&mut ctx, &fixture.user_coin).await,
        USER_COIN_DONATION
    );
    assert_eq!(amount(&mut ctx, &fixture.user_pc).await, USER_PC_DONATION);
    let fee_after = account(&mut ctx, &pool.fee_destination).await.unwrap();
    assert_eq!(fee_after.lamports, fee_before.lamports + CREATE_FEE);
    assert_eq!(
        TokenAccount::unpack(&fee_after.data).unwrap().amount,
        FEE_DEST_DONATION + CREATE_FEE
    );
    assert_eq!(
        account(&mut ctx, &fixture.curve_launch).await.unwrap().data,
        fixture.launch
    );
    let target = account(&mut ctx, &pool.target_orders).await.unwrap();
    assert_eq!(target.owner, raydium_amm::id());
    assert_eq!(target.data.len(), 2208);
    assert!(account(&mut ctx, &pool.openbook_program).await.is_none());

    let mut curve_coin = account(&mut ctx, &fixture.curve_coin).await.unwrap();
    let mut coin = TokenAccount::unpack(&curve_coin.data).unwrap();
    coin.amount = COIN_RETAINED + INIT_COIN;
    TokenAccount::pack(coin, &mut curve_coin.data).unwrap();
    ctx.set_account(&fixture.curve_coin, &curve_coin.into());
    let mut curve_pc = account(&mut ctx, &fixture.curve_pc).await.unwrap();
    let mut pc = TokenAccount::unpack(&curve_pc.data).unwrap();
    pc.amount = PC_LIABILITY + PC_DONATION + INIT_PC;
    TokenAccount::pack(pc, &mut curve_pc.data).unwrap();
    ctx.set_account(&fixture.curve_pc, &curve_pc.into());
    let err = send(&mut ctx, instruction(&fixture, input(TAG_MIGRATE)))
        .await
        .expect_err("replay must fail before a second CPI");
    assert!(err.contains(&ERR_PRECREATED.to_string()), "{err}");
    let forward = send(&mut ctx, instruction(&fixture, input(TAG_FORWARD)))
        .await
        .expect_err("pinned program must reject an existing pool");
    assert!(forward.contains("Custom(49)"), "{forward}");
    assert_eq!(
        amount(&mut ctx, &fixture.curve_coin).await,
        COIN_RETAINED + INIT_COIN
    );
    assert_eq!(amount(&mut ctx, &pool.coin_vault).await, INIT_COIN);
    assert_eq!(amount(&mut ctx, &pool.user_lp).await, user_lp);
}

#[tokio::test]
async fn precreated_pool_is_rejected_and_curve_balances_stay() {
    let fixture = fixture(false);
    let pool = derive_pool(&fixture.market);
    let mut pt = base_test();
    add_market(&mut pt, &fixture, false);
    pt.add_account(
        pool.amm,
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: SYSTEM_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    let mut ctx = pt.start_with_context().await;
    let err = send(&mut ctx, instruction(&fixture, input(TAG_MIGRATE)))
        .await
        .expect_err("precreated pool");
    assert!(err.contains(&ERR_PRECREATED.to_string()), "{err}");
    assert_eq!(
        amount(&mut ctx, &fixture.curve_coin).await,
        INIT_COIN + COIN_RETAINED
    );
    assert_eq!(
        amount(&mut ctx, &fixture.curve_pc).await,
        INIT_PC + PC_LIABILITY + PC_DONATION
    );
    assert_eq!(
        account(&mut ctx, &pool.amm).await.unwrap().lamports,
        1_000_000
    );
    assert!(account(&mut ctx, &pool.user_lp).await.is_none());
}

#[tokio::test]
async fn post_cpi_failure_rolls_the_pool_fee_and_curve_back() {
    let fixture = fixture(false);
    let pool = derive_pool(&fixture.market);
    let mut pt = base_test();
    add_market(&mut pt, &fixture, false);
    let mut ctx = pt.start_with_context().await;
    let fee_before = account(&mut ctx, &pool.fee_destination)
        .await
        .unwrap()
        .lamports;
    let err = send(
        &mut ctx,
        instruction(&fixture, input(TAG_MIGRATE_THEN_ABORT)),
    )
    .await
    .expect_err("forced rollback");
    assert!(err.contains(&ERR_ROLLBACK.to_string()), "{err}");
    assert!(account(&mut ctx, &pool.amm).await.is_none());
    assert!(account(&mut ctx, &pool.lp_mint).await.is_none());
    assert!(account(&mut ctx, &pool.coin_vault).await.is_none());
    assert!(account(&mut ctx, &pool.pc_vault).await.is_none());
    assert_eq!(
        amount(&mut ctx, &fixture.curve_coin).await,
        INIT_COIN + COIN_RETAINED
    );
    assert_eq!(
        amount(&mut ctx, &fixture.curve_pc).await,
        INIT_PC + PC_LIABILITY + PC_DONATION
    );
    assert_eq!(
        amount(&mut ctx, &fixture.user_coin).await,
        USER_COIN_DONATION
    );
    assert_eq!(amount(&mut ctx, &fixture.user_pc).await, USER_PC_DONATION);
    assert_eq!(
        account(&mut ctx, &pool.fee_destination)
            .await
            .unwrap()
            .lamports,
        fee_before
    );
    assert_eq!(
        account(&mut ctx, &fixture.curve_launch).await.unwrap().data,
        fixture.launch
    );
}

#[tokio::test]
async fn token2022_migration_is_rejected_and_the_curve_launch_remains() {
    let fixture = fixture(true);
    let pool = derive_pool(&fixture.market);
    let mut pt = base_test();
    add_market(&mut pt, &fixture, true);
    let mut ctx = pt.start_with_context().await;
    let fee_before = account(&mut ctx, &pool.fee_destination)
        .await
        .unwrap()
        .lamports;
    let err = send(&mut ctx, instruction(&fixture, input(TAG_FORWARD)))
        .await
        .expect_err("token-2022 mint");
    assert!(err.contains("Custom(38)"), "{err}");
    assert!(account(&mut ctx, &pool.amm).await.is_none());
    assert_eq!(
        account(&mut ctx, &fixture.curve_launch).await.unwrap().data,
        fixture.launch
    );
    assert_eq!(
        account(&mut ctx, &fixture.coin_mint).await.unwrap().owner,
        TOKEN_2022
    );
    assert_eq!(
        account(&mut ctx, &pool.fee_destination)
            .await
            .unwrap()
            .lamports,
        fee_before
    );
    assert_eq!(
        amount(&mut ctx, &fixture.curve_coin).await,
        INIT_COIN + COIN_RETAINED
    );
}
