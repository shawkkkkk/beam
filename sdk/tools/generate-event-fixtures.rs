//! Host-only synthetic event serializer. No handler, wallet, RPC or transaction.
//! Run as an example of the existing, locked beam-program package; no new manifest.
use anchor_lang::{prelude::Pubkey, Event};
use beam_program::{trade_types::Shareholder, CreateEvent, TradeEvent};

fn key(value: u8) -> Pubkey { Pubkey::new_from_array([value; 32]) }
fn hex(data: &[u8]) -> String { data.iter().map(|byte| format!("{byte:02x}")).collect() }
fn create(unicode: bool) -> CreateEvent {
    let mut value = CreateEvent {
        name: String::new(), symbol: String::new(), uri: String::new(),
        mint: key(0), bonding_curve: key(0), user: key(0), creator: key(0),
        timestamp: i64::MAX, virtual_token_reserves: 0, virtual_sol_reserves: 0,
        real_token_reserves: 0, token_total_supply: 0, token_program: key(255),
        is_mayhem_mode: false, is_cashback_enabled: true, quote_mint: key(0),
        virtual_quote_reserves: 0, creator_fee_bps: 0, is_holder_reward: false, depth: 0,
    };
    if unicode {
        value.name = "Beam 🚀".into();
        value.symbol = "\u{feff}B\0".into();
        value.uri = "https://example.invalid/🛸.json".into();
        value.mint = key(1); value.bonding_curve = key(2); value.user = key(3);
        value.creator = key(4); value.timestamp = i64::MIN;
        value.virtual_token_reserves = u64::MAX;
        value.virtual_sol_reserves = 9_007_199_254_740_993;
        value.real_token_reserves = 1; value.token_program = key(5);
        value.is_mayhem_mode = true; value.is_cashback_enabled = false;
        value.quote_mint = key(6); value.virtual_quote_reserves = 42;
        value.creator_fee_bps = 65_535; value.is_holder_reward = true; value.depth = 255;
    }
    value
}
fn trade(variant: u8) -> TradeEvent {
    let mut value = TradeEvent {
        mint: key(0), sol_amount: 0, token_amount: 0, is_buy: false,
        user: key(0), timestamp: 0, virtual_sol_reserves: 0,
        virtual_token_reserves: 0, real_sol_reserves: 0, real_token_reserves: 0,
        fee_recipient: key(0), fee_basis_points: 0, fee: 0, creator: key(0),
        creator_fee_basis_points: 0, creator_fee: 0, track_volume: true,
        total_unclaimed_tokens: 0, total_claimed_tokens: 0, current_sol_volume: 0,
        last_update_timestamp: 0, ix_name: String::new(), mayhem_mode: false,
        cashback_fee_basis_points: 0, cashback: 0, buyback_fee_basis_points: 0,
        buyback_fee: 0, shareholders: Vec::new(), quote_mint: key(0), quote_amount: 0,
        virtual_quote_reserves: 0, real_quote_reserves: 0, holder_rewards_bps: 0,
        holder_rewards: 0, creator_fee_unclaimed: 0, unparsed_trailing_field: 0,
    };
    if variant == 1 {
        value.mint = key(7); value.sol_amount = u64::MAX;
        value.token_amount = 9_007_199_254_740_995; value.is_buy = true;
        value.user = key(8); value.timestamp = i64::MIN;
        value.virtual_token_reserves = 1; value.real_sol_reserves = 2;
        value.real_token_reserves = 3; value.fee_recipient = key(9);
        value.fee_basis_points = 50; value.fee = 17; value.creator = key(10);
        value.creator_fee_basis_points = 30; value.creator_fee = 19;
        value.track_volume = false; value.total_unclaimed_tokens = u64::MAX;
        value.total_claimed_tokens = 9; value.current_sol_volume = 9_007_199_254_740_993;
        value.last_update_timestamp = i64::MAX; value.ix_name = "\u{feff}buy\0🚀".into();
        value.mayhem_mode = true; value.cashback_fee_basis_points = 11;
        value.cashback = 12; value.buyback_fee_basis_points = 13; value.buyback_fee = 14;
        value.shareholders = vec![Shareholder { address: key(11), share_bps: 0 },
                                  Shareholder { address: key(12), share_bps: 65_535 }];
        value.quote_mint = key(13); value.quote_amount = u64::MAX;
        value.virtual_quote_reserves = 15; value.real_quote_reserves = 16;
        value.holder_rewards_bps = 17; value.holder_rewards = 18;
        value.creator_fee_unclaimed = 19; value.unparsed_trailing_field = u64::MAX;
    } else if variant == 2 {
        value.mint = key(21); value.user = key(22); value.fee_recipient = key(23);
        value.creator = key(24); value.quote_mint = key(25);
        value.sol_amount = 1; value.token_amount = 2; value.timestamp = -3;
        value.virtual_sol_reserves = 4; value.virtual_token_reserves = 5;
        value.real_sol_reserves = 6; value.real_token_reserves = 7;
        value.fee_basis_points = 8; value.fee = 9;
        value.creator_fee_basis_points = 10; value.creator_fee = 11;
        value.total_unclaimed_tokens = 12; value.total_claimed_tokens = 13;
        value.current_sol_volume = 14; value.last_update_timestamp = -15;
        value.ix_name = "sell".into(); value.cashback_fee_basis_points = 16;
        value.cashback = 17; value.buyback_fee_basis_points = 18; value.buyback_fee = 19;
        value.quote_amount = 20; value.virtual_quote_reserves = 21;
        value.real_quote_reserves = 22; value.holder_rewards_bps = 23;
        value.holder_rewards = 24; value.creator_fee_unclaimed = 25;
        value.unparsed_trailing_field = 26;
        value.shareholders = vec![Shareholder { address: key(200), share_bps: 0 },
            Shareholder { address: key(201), share_bps: 1 },
            Shareholder { address: key(202), share_bps: 65_535 }];
    }
    value
}
fn main() {
    let cases = [
        ("create-unicode", create(true).data()), ("create-empty", create(false).data()),
        ("trade-mixed", trade(1).data()), ("trade-empty", trade(0).data()),
        ("trade-sequential", trade(2).data()),
    ];
    println!("{{\"provenance\":\"actual Anchor Event::data; synthetic values only\",\"events\":[");
    for (index, (id, data)) in cases.iter().enumerate() {
        println!("{}{{\"id\":\"{id}\",\"dataHex\":\"{}\"}}", if index == 0 { "" } else { "," }, hex(data));
    }
    println!("]}}");
}
