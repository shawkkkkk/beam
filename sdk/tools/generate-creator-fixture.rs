//! Synthetic host serialization only: no handler invocation, RPC or transaction.
//! Run as an example of the actual beam-program crate at the recorded source.
use anchor_lang::{prelude::Pubkey, ToAccountMetas};

const SOURCE_COMMIT: &str = "b8120a8a58b1375ba3b8f0d2a590c49923e18021";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn event_json(name: &str, timestamp: i64, creator: Pubkey, fee: u64, quote: Pubkey) -> String {
    let event = beam_program::CollectCreatorFeeEvent {
        timestamp,
        creator,
        creator_fee: fee,
        quote_mint: quote,
    };
    let bytes = anchor_lang::Event::data(&event);
    format!(
        "{{\"name\":\"{name}\",\"dataHex\":\"{}\",\"expected\":{{\"timestamp\":\"{timestamp}\",\"creator\":\"{creator}\",\"creatorFee\":\"{fee}\",\"quoteMint\":\"{quote}\"}}}}",
        hex(&bytes)
    )
}

fn main() {
    let observed = std::env::var("BEAM_CREATOR_FIXTURE_SOURCE_COMMIT")
        .expect("record actual source HEAD in BEAM_CREATOR_FIXTURE_SOURCE_COMMIT");
    assert_eq!(observed, SOURCE_COMMIT, "producer supports the reviewed source only");
    let creator = Pubkey::new_from_array([7; 32]);
    let vault = Pubkey::new_from_array([8; 32]);
    let event_authority = Pubkey::new_from_array([9; 32]);
    let system = anchor_lang::system_program::ID;
    let program = beam_program::ID;
    let accounts = beam_program::accounts::CollectCreatorFee {
        creator,
        creator_vault: vault,
        system_program: system,
        event_authority,
        program,
    };
    let metas = accounts.to_account_metas(None).iter().map(|meta| {
        format!("{{\"pubkey\":\"{}\",\"isSigner\":{},\"isWritable\":{}}}",
            meta.pubkey, meta.is_signer, meta.is_writable)
    }).collect::<Vec<_>>().join(",");
    let instruction = beam_program::instruction::CollectCreatorFee {};
    let data = anchor_lang::InstructionData::data(&instruction);
    let events = [
        event_json("ordinary-synthetic", 1_700_000_001, creator, 1_234_567_890, Pubkey::default()),
        event_json("type-minimums-not-a-handler-receipt", i64::MIN, creator, 0, Pubkey::default()),
        event_json("type-maximums-shared-event-layout", i64::MAX, creator, u64::MAX, Pubkey::new_from_array([11; 32])),
    ].join(",");
    println!(
        "{{\"schemaVersion\":1,\"provenance\":{{\"kind\":\"actual Anchor host serialization of synthetic values\",\"sourceCommit\":\"{observed}\",\"anchorVersion\":\"0.31.1\",\"producer\":\"sdk/tools/generate-creator-fixture.rs\",\"runtimeHandlerExecuted\":false}},\"programId\":\"{program}\",\"collectCreatorFee\":{{\"dataHex\":\"{}\",\"accounts\":{{\"creator\":\"{creator}\",\"creatorVault\":\"{vault}\",\"systemProgram\":\"{system}\",\"eventAuthority\":\"{event_authority}\",\"program\":\"{program}\"}},\"metas\":[{metas}]}},\"events\":[{events}]}}",
        hex(&data)
    );
}
