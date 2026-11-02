//! The checked-in Beam IDL must match this program's Anchor types.
//!
//! Host serialization only. Nothing here builds a transaction, signs, or
//! enables the superseded CPMM migration.
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_lang::{
    AccountDeserialize, AccountSerialize, Discriminator, Event, InstructionData, ToAccountMetas,
};
use beam_program as beam;
use pump_reference as reference;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(relative)
}

fn load_json(relative: &str) -> Value {
    let path = repo_file(relative);
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn pubkey(value: &str) -> Pubkey {
    value.parse().unwrap_or_else(|err| panic!("pubkey {value}: {err}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let digest = anchor_lang::solana_program::hash::hash(format!("{namespace}:{name}").as_bytes());
    let mut out = [0; 8];
    out.copy_from_slice(&digest.to_bytes()[..8]);
    out
}

fn discriminator(value: &Value) -> Vec<u8> {
    value["discriminator"]
        .as_array()
        .unwrap_or_else(|| panic!("missing discriminator in {value}"))
        .iter()
        .map(|byte| {
            byte.as_u64().and_then(|n| u8::try_from(n).ok()).unwrap_or_else(|| panic!("bad discriminator {value}"))
        })
        .collect()
}

fn instruction<'a>(idl: &'a Value, name: &str) -> &'a Value {
    idl["instructions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|ix| ix["name"] == name)
        .unwrap_or_else(|| panic!("IDL missing instruction {name}"))
}

fn account<'a>(idl: &'a Value, name: &str) -> &'a Value {
    idl["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("IDL missing account {name}"))
}

fn event<'a>(idl: &'a Value, name: &str) -> &'a Value {
    idl["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("IDL missing event {name}"))
}

fn error_code(idl: &Value, name: &str) -> u64 {
    idl["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("IDL missing error {name}"))["code"]
        .as_u64()
        .unwrap()
}

fn anchor_code(err: impl Into<anchor_lang::error::Error>) -> u64 {
    match err.into() {
        anchor_lang::error::Error::AnchorError(err) => u64::from(err.error_code_number),
        other => panic!("expected an Anchor error, got {other:?}"),
    }
}

fn type_def<'a>(idl: &'a Value, name: &str) -> &'a Value {
    idl["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("IDL missing type {name}"))
}

fn field_names(idl: &Value, name: &str) -> Vec<String> {
    type_def(idl, name)["type"]["fields"]
        .as_array()
        .unwrap_or_else(|| panic!("{name} has no named fields"))
        .iter()
        .map(|field| field["name"].as_str().unwrap().to_string())
        .collect()
}

fn flat_accounts(items: &[Value]) -> Vec<&Value> {
    let mut out = Vec::new();
    for item in items {
        if let Some(nested) = item.get("accounts").and_then(Value::as_array) {
            out.extend(flat_accounts(nested));
        } else {
            out.push(item);
        }
    }
    out
}

fn assert_metas(idl_ix: &Value, names: &[&str], metas: &[AccountMeta]) {
    let accounts = flat_accounts(idl_ix["accounts"].as_array().unwrap());
    assert_eq!(accounts.len(), names.len(), "{}", idl_ix["name"]);
    assert_eq!(metas.len(), names.len(), "{}", idl_ix["name"]);
    for ((account, name), meta) in accounts.iter().zip(names).zip(metas) {
        assert_eq!(account["name"], *name, "{}", idl_ix["name"]);
        assert_eq!(account["signer"].as_bool().unwrap_or(false), meta.is_signer, "{name}");
        assert_eq!(account["writable"].as_bool().unwrap_or(false), meta.is_writable, "{name}");
    }
}

fn assert_fixture_metas(fixture_metas: &[Value], metas: &[AccountMeta]) {
    assert_eq!(fixture_metas.len(), metas.len());
    for (expected, meta) in fixture_metas.iter().zip(metas) {
        assert_eq!(expected["pubkey"], meta.pubkey.to_string());
        assert_eq!(expected["isSigner"], meta.is_signer);
        assert_eq!(expected["isWritable"], meta.is_writable);
    }
}

fn assert_data(idl_ix: &Value, data: &[u8]) {
    assert_eq!(discriminator(idl_ix), &data[..8], "{}", idl_ix["name"]);
    assert_eq!(discriminator(idl_ix), sighash("global", idl_ix["name"].as_str().unwrap()));
}

fn key_at(bytes: [u8; 32]) -> Pubkey {
    Pubkey::new_from_array(bytes)
}

fn repeat(byte: u8) -> Pubkey {
    key_at([byte; 32])
}

fn meta_json(metas: &[AccountMeta]) -> Vec<Value> {
    metas
        .iter()
        .map(|meta| {
            json!({
                "pubkey": meta.pubkey.to_string(),
                "isSigner": meta.is_signer,
                "isWritable": meta.is_writable,
            })
        })
        .collect()
}

fn accounts_object(names: &[&str], metas: &[AccountMeta]) -> Value {
    let mut object = serde_json::Map::new();
    for (name, meta) in names.iter().zip(metas) {
        let camel = snake_to_camel(name);
        object.insert(camel, Value::String(meta.pubkey.to_string()));
    }
    Value::Object(object)
}

fn snake_to_camel(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for ch in name.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn migration_fixture() -> Value {
    let global = repeat(1);
    let authority = repeat(2);
    let config = repeat(3);
    let amm_config = repeat(4);
    let withdraw_authority = repeat(5);
    let mint = repeat(6);
    let bonding_curve = repeat(7);
    let curve_token_vault = repeat(8);
    let user = repeat(9);
    let migration_authority = repeat(10);
    let authority_base = repeat(11);
    let authority_wsol = repeat(12);
    let raydium_authority = repeat(13);
    let pool = repeat(14);
    let lp_mint = repeat(15);
    let authority_lp = repeat(16);
    let vault_0 = repeat(17);
    let vault_1 = repeat(18);
    let observation = repeat(19);
    let receipt = repeat(20);
    let base_token_program = repeat(21);
    let system_program = anchor_lang::system_program::ID;
    let token_program = anchor_spl::token::ID;
    let associated_token_program = anchor_spl::associated_token::ID;
    let rent = anchor_lang::solana_program::sysvar::rent::ID;
    let wsol_mint = anchor_spl::token::spl_token::native_mint::ID;
    let raydium_program = beam::RAYDIUM_ID;
    let create_pool_fee = beam::CREATE_POOL_FEE;

    let configure_names = ["global", "authority", "config", "amm_config", "system_program"];
    let configure_accounts = beam::accounts::ConfigureRaydium {
        global,
        authority,
        config,
        amm_config,
        system_program,
    };
    let configure_metas = configure_accounts.to_account_metas(None);
    let configure_data = beam::instruction::ConfigureRaydium {}.data();

    let enabled_names = ["global", "authority", "config"];
    let enabled_accounts = beam::accounts::SetRaydiumEnabled { global, authority, config };
    let enabled_metas = enabled_accounts.to_account_metas(None);
    let enabled_false = beam::instruction::SetRaydiumEnabled { enabled: false }.data();
    let enabled_true = beam::instruction::SetRaydiumEnabled { enabled: true }.data();

    let migrate_names = [
        "global",
        "withdraw_authority",
        "mint",
        "bonding_curve",
        "curve_token_vault",
        "user",
        "config",
        "amm_config",
        "raydium_program",
        "migration_authority",
        "authority_base",
        "wsol_mint",
        "authority_wsol",
        "raydium_authority",
        "pool",
        "lp_mint",
        "authority_lp",
        "vault_0",
        "vault_1",
        "create_pool_fee",
        "observation",
        "receipt",
        "base_token_program",
        "token_program",
        "associated_token_program",
        "system_program",
        "rent",
    ];
    let migrate_accounts = beam::accounts::MigrateRaydium {
        global,
        withdraw_authority,
        mint,
        bonding_curve,
        curve_token_vault,
        user,
        config,
        amm_config,
        raydium_program,
        migration_authority,
        authority_base,
        wsol_mint,
        authority_wsol,
        raydium_authority,
        pool,
        lp_mint,
        authority_lp,
        vault_0,
        vault_1,
        create_pool_fee,
        observation,
        receipt,
        base_token_program,
        token_program,
        associated_token_program,
        system_program,
        rent,
    };
    let migrate_metas = migrate_accounts.to_account_metas(None);
    let setup_budget = 1_500_000_u64;
    let migrate_data = beam::instruction::MigrateRaydium { setup_budget }.data();

    let config_account = beam::BeamRaydiumConfig {
        amm_config,
        raydium_program,
        enabled: false,
    };
    let mut config_bytes = Vec::new();
    config_account.try_serialize(&mut config_bytes).unwrap();
    let receipt_account = beam::BeamMigrationReceipt {
        mint,
        pool,
        token_amount: 10,
        quote_amount: 115,
        migration_fee: 10,
        lp_burned: 99,
        setup_spent: 4,
    };
    let mut receipt_bytes = Vec::new();
    receipt_account.try_serialize(&mut receipt_bytes).unwrap();
    let migration_event = beam::BeamRaydiumMigration {
        mint,
        pool,
        token_amount: 10,
        quote_amount: 115,
        migration_fee: 10,
        lp_burned: 99,
    };
    let event_bytes = Event::data(&migration_event);

    json!({
        "fixtureOnly": true,
        "execution": "fail-closed",
        "featureSet": "default raydium-devnet",
        "programId": beam::ID.to_string(),
        "raydiumProgram": raydium_program.to_string(),
        "createPoolFee": create_pool_fee.to_string(),
        "configureRaydium": {
            "dataHex": hex(&configure_data),
            "accounts": accounts_object(&configure_names, &configure_metas),
            "metas": meta_json(&configure_metas),
        },
        "setRaydiumEnabled": {
            "accounts": accounts_object(&enabled_names, &enabled_metas),
            "metas": meta_json(&enabled_metas),
            "enabledFalseDataHex": hex(&enabled_false),
            "enabledTrueDataHex": hex(&enabled_true),
        },
        "migrateRaydium": {
            "setupBudget": setup_budget.to_string(),
            "dataHex": hex(&migrate_data),
            "accounts": accounts_object(&migrate_names, &migrate_metas),
            "metas": meta_json(&migrate_metas),
        },
        "beamRaydiumConfig": {
            "dataHex": hex(&config_bytes),
            "expected": {
                "ammConfig": amm_config.to_string(),
                "raydiumProgram": raydium_program.to_string(),
                "enabled": false,
            }
        },
        "beamMigrationReceipt": {
            "dataHex": hex(&receipt_bytes),
            "expected": {
                "mint": mint.to_string(),
                "pool": pool.to_string(),
                "tokenAmount": "10",
                "quoteAmount": "115",
                "migrationFee": "10",
                "lpBurned": "99",
                "setupSpent": "4",
            }
        },
        "beamRaydiumMigration": {
            "dataHex": hex(&event_bytes),
            "expected": {
                "mint": mint.to_string(),
                "pool": pool.to_string(),
                "tokenAmount": "10",
                "quoteAmount": "115",
                "migrationFee": "10",
                "lpBurned": "99",
            }
        }
    })
}

#[test]
fn idl_identity_and_discriminators_match_this_program() {
    let idl = load_json("sdk/idl/beam.json");
    assert_eq!(idl["address"], beam::ID.to_string());
    assert_ne!(beam::ID, reference::ID);
    assert_eq!(reference::ID.to_string(), "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
    // The Anchor module name is retained. The program id is Beam's placeholder.
    assert_eq!(idl["metadata"]["name"], "pump_recovered");
    assert_eq!(idl["metadata"]["version"], "0.1.0");
    assert_eq!(idl["metadata"]["spec"], "0.1.0");

    for ix in idl["instructions"].as_array().unwrap() {
        assert_eq!(
            discriminator(ix),
            sighash("global", ix["name"].as_str().unwrap()),
            "instruction {}",
            ix["name"]
        );
    }
    for item in idl["accounts"].as_array().unwrap() {
        assert_eq!(
            discriminator(item),
            sighash("account", item["name"].as_str().unwrap()),
            "account {}",
            item["name"]
        );
    }
    for item in idl["events"].as_array().unwrap() {
        assert_eq!(
            discriminator(item),
            sighash("event", item["name"].as_str().unwrap()),
            "event {}",
            item["name"]
        );
    }
    assert_eq!(discriminator(account(&idl, "BondingCurve")), beam::trade_types::BondingCurve::DISCRIMINATOR);
    assert_eq!(discriminator(account(&idl, "Global")), beam::Global::DISCRIMINATOR);
    assert_eq!(discriminator(account(&idl, "BeamRaydiumConfig")), beam::BeamRaydiumConfig::DISCRIMINATOR);
    assert_eq!(discriminator(account(&idl, "BeamMigrationReceipt")), beam::BeamMigrationReceipt::DISCRIMINATOR);
    assert_eq!(discriminator(event(&idl, "CompleteEvent")), beam::CompleteEvent::DISCRIMINATOR);
    assert_eq!(discriminator(event(&idl, "BeamRaydiumMigration")), beam::BeamRaydiumMigration::DISCRIMINATOR);
    assert!(idl["instructions"].as_array().unwrap().len() > 40);
    assert!(idl["accounts"].as_array().unwrap().iter().any(|item| item["name"] == "BondingCurve"));
    let dependency_idl = load_json("programs/beam/idls/pump_fees.json");
    assert_ne!(dependency_idl.get("address"), Some(&idl["address"]));
}

#[test]
fn retained_create_buy_sell_encodings_match_reference_fixture_and_idl() {
    let idl = load_json("sdk/idl/beam.json");
    let fixture = load_json("sdk/fixtures/native-sol.json");
    assert_eq!(fixture["programId"], beam::ID.to_string());
    let creator = pubkey(fixture["create"]["args"]["creator"].as_str().unwrap());
    let name = fixture["create"]["args"]["name"].as_str().unwrap();
    let symbol = fixture["create"]["args"]["symbol"].as_str().unwrap();
    let uri = fixture["create"]["args"]["uri"].as_str().unwrap();
    for (name, symbol, uri, creator) in [
        (name, symbol, uri, creator),
        ("", "", "", creator),
        ("a", "b", "c", Pubkey::default()),
    ] {
        let beam_data = beam::instruction::Create {
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            creator,
        }
        .data();
        let reference_data = reference::instruction::Create {
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            creator,
        }
        .data();
        assert_eq!(beam_data, reference_data);
    }
    let create_data = beam::instruction::Create {
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        creator,
    }
    .data();
    assert_eq!(hex(&create_data), fixture["create"]["dataHex"].as_str().unwrap());
    assert_data(instruction(&idl, "create"), &create_data);
    assert_eq!(
        instruction(&idl, "create")["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|arg| arg["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["name", "symbol", "uri", "creator"]
    );

    let amount = 42_u64;
    let max_sol_cost = 987_654_321_u64;
    let buy = beam::instruction::Buy {
        amount,
        max_sol_cost,
        track_volume: beam::OptionBool(true),
        partial_fill: beam::OptionBool(false),
    }
    .data();
    let reference_buy = reference::instruction::Buy {
        amount,
        max_sol_cost,
        track_volume: reference::OptionBool(true),
        partial_fill: reference::OptionBool(false),
    }
    .data();
    assert_eq!(buy, reference_buy);
    assert_eq!(hex(&buy), fixture["buy"]["dataHex"].as_str().unwrap());
    assert_data(instruction(&idl, "buy"), &buy);
    let sell = beam::instruction::Sell { amount, min_sol_output: 123_456_789 }.data();
    assert_eq!(
        sell,
        reference::instruction::Sell { amount, min_sol_output: 123_456_789 }.data()
    );
    assert_eq!(hex(&sell), fixture["sell"]["dataHex"].as_str().unwrap());
    assert_data(instruction(&idl, "sell"), &sell);

    let create_accounts = accounts_from_fixture(&fixture["create"]["accounts"], CREATE_FIELDS);
    let create_metas = beam::accounts::Create {
        mint: create_accounts[0],
        mint_authority: create_accounts[1],
        bonding_curve: create_accounts[2],
        associated_bonding_curve: create_accounts[3],
        global: create_accounts[4],
        mpl_token_metadata: create_accounts[5],
        metadata: create_accounts[6],
        user: create_accounts[7],
        system_program: create_accounts[8],
        token_program: create_accounts[9],
        associated_token_program: create_accounts[10],
        rent: create_accounts[11],
        event_authority: create_accounts[12],
        program: create_accounts[13],
    }
    .to_account_metas(None);
    assert_metas(
        instruction(&idl, "create"),
        &[
            "mint",
            "mint_authority",
            "bonding_curve",
            "associated_bonding_curve",
            "global",
            "mpl_token_metadata",
            "metadata",
            "user",
            "system_program",
            "token_program",
            "associated_token_program",
            "rent",
            "event_authority",
            "program",
        ],
        &create_metas,
    );
    assert_fixture_metas(fixture["create"]["metas"].as_array().unwrap(), &create_metas);

    let buy_accounts = accounts_from_fixture(&fixture["buy"]["accounts"], BUY_FIELDS);
    let buy_metas = beam::accounts::LegacyBuy {
        global: buy_accounts[0],
        fee_recipient: buy_accounts[1],
        mint: buy_accounts[2],
        bonding_curve: buy_accounts[3],
        associated_bonding_curve: buy_accounts[4],
        associated_user: buy_accounts[5],
        user: buy_accounts[6],
        system_program: buy_accounts[7],
        token_program: buy_accounts[8],
        creator_vault: buy_accounts[9],
        event_authority: buy_accounts[10],
        program: buy_accounts[11],
        global_volume_accumulator: buy_accounts[12],
        user_volume_accumulator: buy_accounts[13],
        fee_config: buy_accounts[14],
        fee_program: buy_accounts[15],
    }
    .to_account_metas(None);
    assert_metas(
        instruction(&idl, "buy"),
        &[
            "global",
            "fee_recipient",
            "mint",
            "bonding_curve",
            "associated_bonding_curve",
            "associated_user",
            "user",
            "system_program",
            "token_program",
            "creator_vault",
            "event_authority",
            "program",
            "global_volume_accumulator",
            "user_volume_accumulator",
            "fee_config",
            "fee_program",
        ],
        &buy_metas,
    );
    assert_fixture_metas(fixture["buy"]["metas"].as_array().unwrap(), &buy_metas);

    let sell_accounts = accounts_from_fixture(&fixture["sell"]["accounts"], SELL_FIELDS);
    let sell_metas = beam::accounts::LegacySell {
        global: sell_accounts[0],
        fee_recipient: sell_accounts[1],
        mint: sell_accounts[2],
        bonding_curve: sell_accounts[3],
        associated_bonding_curve: sell_accounts[4],
        associated_user: sell_accounts[5],
        user: sell_accounts[6],
        system_program: sell_accounts[7],
        creator_vault: sell_accounts[8],
        token_program: sell_accounts[9],
        event_authority: sell_accounts[10],
        program: sell_accounts[11],
        fee_config: sell_accounts[12],
        fee_program: sell_accounts[13],
    }
    .to_account_metas(None);
    assert_metas(
        instruction(&idl, "sell"),
        &[
            "global",
            "fee_recipient",
            "mint",
            "bonding_curve",
            "associated_bonding_curve",
            "associated_user",
            "user",
            "system_program",
            "creator_vault",
            "token_program",
            "event_authority",
            "program",
            "fee_config",
            "fee_program",
        ],
        &sell_metas,
    );
    assert_fixture_metas(fixture["sell"]["metas"].as_array().unwrap(), &sell_metas);

    let complete = beam::CompleteEvent {
        user: pubkey(fixture["completeEvent"]["expected"]["user"].as_str().unwrap()),
        mint: pubkey(fixture["completeEvent"]["expected"]["mint"].as_str().unwrap()),
        bonding_curve: pubkey(fixture["completeEvent"]["expected"]["bondingCurve"].as_str().unwrap()),
        timestamp: fixture["completeEvent"]["expected"]["timestamp"].as_str().unwrap().parse().unwrap(),
        quote_mint: pubkey(fixture["completeEvent"]["expected"]["quoteMint"].as_str().unwrap()),
    };
    assert_eq!(hex(&Event::data(&complete)), fixture["completeEvent"]["dataHex"].as_str().unwrap());
    assert_eq!(discriminator(event(&idl, "CompleteEvent")), &Event::data(&complete)[..8]);

    let curve_bytes = hex_decode(fixture["currentBondingCurve"]["dataHex"].as_str().unwrap());
    let curve = beam::trade_types::BondingCurve::try_deserialize(&mut &curve_bytes[..]).unwrap();
    let mut reserialized = Vec::new();
    curve.try_serialize(&mut reserialized).unwrap();
    assert_eq!(reserialized, curve_bytes);
    assert_eq!(&curve_bytes[..8], beam::trade_types::BondingCurve::DISCRIMINATOR);
    assert_eq!(
        field_names(&idl, "BondingCurve"),
        [
            "virtual_token_reserves",
            "virtual_quote_reserves",
            "real_token_reserves",
            "real_quote_reserves",
            "token_total_supply",
            "complete",
            "creator",
            "is_mayhem_mode",
            "is_cashback_coin",
            "quote_mint",
            "creator_fee_bps",
            "can_edit_creator_fee",
            "is_holder_reward",
            "creator_fee",
            "protocol_fees",
            "depth",
            "initial_virtual_quote_reserves",
            "post_complete_base_out",
            "post_complete_quote_in",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>()
    );
    let expected = &fixture["currentBondingCurve"]["expected"];
    assert_eq!(curve.virtual_token_reserves, expected["virtualTokenReserves"].as_str().unwrap().parse::<u64>().unwrap());
    assert_eq!(curve.complete, true);
    assert_eq!(curve.is_mayhem_mode, false);
    assert_eq!(curve.creator, pubkey(expected["creator"].as_str().unwrap()));
    assert_eq!(curve.post_complete_quote_in, 17);
}

const CREATE_FIELDS: &[&str] = &[
    "mint",
    "mintAuthority",
    "bondingCurve",
    "associatedBondingCurve",
    "global",
    "mplTokenMetadata",
    "metadata",
    "user",
    "systemProgram",
    "tokenProgram",
    "associatedTokenProgram",
    "rent",
    "eventAuthority",
    "program",
];
const BUY_FIELDS: &[&str] = &[
    "global",
    "feeRecipient",
    "mint",
    "bondingCurve",
    "associatedBondingCurve",
    "associatedUser",
    "user",
    "systemProgram",
    "tokenProgram",
    "creatorVault",
    "eventAuthority",
    "program",
    "globalVolumeAccumulator",
    "userVolumeAccumulator",
    "feeConfig",
    "feeProgram",
];
const SELL_FIELDS: &[&str] = &[
    "global",
    "feeRecipient",
    "mint",
    "bondingCurve",
    "associatedBondingCurve",
    "associatedUser",
    "user",
    "systemProgram",
    "creatorVault",
    "tokenProgram",
    "eventAuthority",
    "program",
    "feeConfig",
    "feeProgram",
];

fn accounts_from_fixture(accounts: &Value, fields: &[&str]) -> Vec<Pubkey> {
    fields.iter().map(|name| pubkey(accounts[*name].as_str().unwrap())).collect()
}

fn hex_decode(text: &str) -> Vec<u8> {
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn retained_blocked_interfaces_keep_reference_encodings() {
    let idl = load_json("sdk/idl/beam.json");
    let samples = [
        beam::instruction::Initialize {}.data(),
        beam::instruction::Migrate {}.data(),
        beam::instruction::MigrateV2 {}.data(),
        beam::instruction::ToggleMayhemMode { enabled: true }.data(),
        beam::instruction::SetMayhemVirtualParams {}.data(),
        beam::instruction::AdminCto {
            is_holder_reward: Some(false),
            creator_fee_bps: Some(25),
            new_creator: Some(Pubkey::new_from_array([4; 32])),
        }
        .data(),
        beam::instruction::MultiHopCurveSwap { amount_in: 9, hop_fees: vec![1, 2, 3] }.data(),
    ];
    let reference_samples = [
        reference::instruction::Initialize {}.data(),
        reference::instruction::Migrate {}.data(),
        reference::instruction::MigrateV2 {}.data(),
        reference::instruction::ToggleMayhemMode { enabled: true }.data(),
        reference::instruction::SetMayhemVirtualParams {}.data(),
        reference::instruction::AdminCto {
            is_holder_reward: Some(false),
            creator_fee_bps: Some(25),
            new_creator: Some(Pubkey::new_from_array([4; 32])),
        }
        .data(),
        reference::instruction::MultiHopCurveSwap { amount_in: 9, hop_fees: vec![1, 2, 3] }.data(),
    ];
    let names = [
        "initialize",
        "migrate",
        "migrate_v2",
        "toggle_mayhem_mode",
        "set_mayhem_virtual_params",
        "admin_cto",
        "multi_hop_curve_swap",
    ];
    for ((beam_data, reference_data), name) in samples.iter().zip(&reference_samples).zip(names) {
        assert_eq!(beam_data, reference_data, "{name}");
        assert_data(instruction(&idl, name), beam_data);
    }
    let upstream = fs::read_to_string(repo_file("upstream/pumpfun-source/programs/pump/src/lib.rs")).unwrap();
    assert!(!upstream.contains("fn configure_raydium"));
    assert!(!upstream.contains("fn set_raydium_enabled"));
    assert!(!upstream.contains("fn migrate_raydium"));
    assert_ne!(
        beam::instruction::Migrate {}.data(),
        beam::instruction::MigrateRaydium { setup_budget: 0 }.data()
    );
    assert_eq!(anchor_code(beam::PumpError::NotAuthorized), anchor_code(reference::PumpError::NotAuthorized));
    assert_eq!(error_code(&idl, "NotAuthorized"), anchor_code(beam::PumpError::NotAuthorized));
    assert_eq!(error_code(&idl, "MayhemDisabled"), anchor_code(beam::beam_policy::BeamError::MayhemDisabled));
    assert_eq!(error_code(&idl, "PumpSwapDisabled"), anchor_code(beam::beam_policy::BeamError::PumpSwapDisabled));
    assert_eq!(error_code(&idl, "MigrationNotEnabled"), anchor_code(beam::beam_policy::BeamError::MigrationNotEnabled));
    for name in [
        "InvalidRaydiumAccount",
        "InvalidMigrationBalance",
        "UnsupportedMint",
        "InvalidMigrationResult",
    ] {
        assert!(idl["errors"].as_array().unwrap().iter().any(|err| err["name"] == name), "{name}");
    }
}

// The golden file records the default `raydium-devnet` program and fee ids.
#[cfg(feature = "raydium-devnet")]
#[test]
fn gated_migration_fixture_matches_idl_and_anchor_accounts() {
    let idl = load_json("sdk/idl/beam.json");
    let fresh = migration_fixture();
    let fixture = load_json("sdk/fixtures/gated-migration.json");
    assert_eq!(fixture, fresh);
    assert_eq!(fixture["programId"], idl["address"]);
    assert_eq!(fixture["execution"], "fail-closed");
    assert_ne!(fixture["raydiumProgram"], beam::ID.to_string());

    let configure = instruction(&idl, "configure_raydium");
    let configure_data = hex_decode(fixture["configureRaydium"]["dataHex"].as_str().unwrap());
    assert_data(configure, &configure_data);
    assert!(configure["args"].as_array().unwrap().is_empty());
    assert_eq!(configure_data.len(), 8);

    let enabled = instruction(&idl, "set_raydium_enabled");
    let enabled_false = hex_decode(fixture["setRaydiumEnabled"]["enabledFalseDataHex"].as_str().unwrap());
    let enabled_true = hex_decode(fixture["setRaydiumEnabled"]["enabledTrueDataHex"].as_str().unwrap());
    assert_data(enabled, &enabled_false);
    assert_eq!(enabled["args"][0]["name"], "enabled");
    assert_eq!(enabled["args"][0]["type"], "bool");
    assert_eq!(enabled_false.len(), 9);
    assert_eq!(enabled_true.len(), 9);
    assert_eq!(enabled_false[8], 0);
    assert_eq!(enabled_true[8], 1);
    assert_eq!(&enabled_true[..8], &enabled_false[..8]);

    let migrate = instruction(&idl, "migrate_raydium");
    let migrate_data = hex_decode(fixture["migrateRaydium"]["dataHex"].as_str().unwrap());
    assert_data(migrate, &migrate_data);
    assert_eq!(migrate["args"][0]["name"], "setup_budget");
    assert_eq!(migrate["args"][0]["type"], "u64");
    assert_eq!(&migrate_data[8..], &1_500_000_u64.to_le_bytes());

    let configure_names = ["global", "authority", "config", "amm_config", "system_program"];
    let enabled_names = ["global", "authority", "config"];
    let migrate_names = [
        "global",
        "withdraw_authority",
        "mint",
        "bonding_curve",
        "curve_token_vault",
        "user",
        "config",
        "amm_config",
        "raydium_program",
        "migration_authority",
        "authority_base",
        "wsol_mint",
        "authority_wsol",
        "raydium_authority",
        "pool",
        "lp_mint",
        "authority_lp",
        "vault_0",
        "vault_1",
        "create_pool_fee",
        "observation",
        "receipt",
        "base_token_program",
        "token_program",
        "associated_token_program",
        "system_program",
        "rent",
    ];
    assert_metas(configure, &configure_names, &metas_of(&fixture["configureRaydium"]));
    assert_metas(enabled, &enabled_names, &metas_of(&fixture["setRaydiumEnabled"]));
    assert_metas(migrate, &migrate_names, &metas_of(&fixture["migrateRaydium"]));

    // Anchor's seed resolver emits expressions that do not compile for this
    // reconstructed program, so the checked-in IDL has no PDA or address
    // resolution. Fixed program ids still live on the account metas.
    for account in migrate["accounts"].as_array().unwrap() {
        assert!(account.get("pda").is_none(), "{}", account["name"]);
        assert!(account.get("address").is_none(), "{}", account["name"]);
    }
    let raydium = fixture["migrateRaydium"]["accounts"]["raydiumProgram"].as_str().unwrap();
    let fee = fixture["migrateRaydium"]["accounts"]["createPoolFee"].as_str().unwrap();
    assert_eq!(raydium, beam::RAYDIUM_ID.to_string());
    assert_eq!(fee, beam::CREATE_POOL_FEE.to_string());

    assert_eq!(
        field_names(&idl, "BeamRaydiumConfig"),
        ["amm_config", "raydium_program", "enabled"].into_iter().map(str::to_string).collect::<Vec<_>>()
    );
    assert_eq!(
        field_names(&idl, "BeamMigrationReceipt"),
        ["mint", "pool", "token_amount", "quote_amount", "migration_fee", "lp_burned", "setup_spent"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        field_names(&idl, "BeamRaydiumMigration"),
        ["mint", "pool", "token_amount", "quote_amount", "migration_fee", "lp_burned"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );
    let config_bytes = hex_decode(fixture["beamRaydiumConfig"]["dataHex"].as_str().unwrap());
    let config = beam::BeamRaydiumConfig::try_deserialize(&mut &config_bytes[..]).unwrap();
    assert!(!config.enabled);
    assert_eq!(config.raydium_program, beam::RAYDIUM_ID);
    assert_eq!(&config_bytes[..8], beam::BeamRaydiumConfig::DISCRIMINATOR);
    let event_bytes = hex_decode(fixture["beamRaydiumMigration"]["dataHex"].as_str().unwrap());
    assert_eq!(&event_bytes[..8], beam::BeamRaydiumMigration::DISCRIMINATOR);
    assert_eq!(event_bytes.len(), 8 + 32 + 32 + 8 * 4);
}

fn metas_of(section: &Value) -> Vec<AccountMeta> {
    section["metas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|meta| AccountMeta {
            pubkey: pubkey(meta["pubkey"].as_str().unwrap()),
            is_signer: meta["isSigner"].as_bool().unwrap(),
            is_writable: meta["isWritable"].as_bool().unwrap(),
        })
        .collect()
}
