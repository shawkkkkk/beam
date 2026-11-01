# Local BuyV2 and SellV2 ABI fixtures

This SDK slice encodes only `buy_v2` and `sell_v2` instruction data and their
ordered base account metas. It is based on Beam source commit
`4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf` and Anchor 0.31.1. The
`beam-v2-trades.mjs` exports are `encodeBuyV2`, `buildBuyV2`, `encodeSellV2`,
and `buildSellV2`. Arguments are `amount`, `maxSolCost`, `partialFill` for a
buy, and `amount`, `minSolOutput` for a sell. Amounts accept exact `bigint`,
safe integer, or canonical u64 decimal string; `partialFill` is a boolean.
Builders require exactly the named base accounts and return
`{ programId, keys, data }` for local inspection. They do not add remaining
accounts or construct a transaction.

## Source and fixture receipt

`programs/beam/src/lib.rs` declares the instruction argument order;
`programs/beam/src/buy_v2.rs` and `sell_v2.rs` declare the account order and
constraints. The golden instruction bytes and 27 BuyV2 / 26 SellV2 account
metas in `fixtures/v2-trades-anchor.txt` were emitted by compiling the
committed `tools/generate-v2-trade-fixtures.rs` producer against the exact
source commit. A disposable checkout held a temporary
`programs/beam/examples/beam_v2_trades_fixture.rs`; the actual run used
`cargo run --locked --offline --manifest-path programs/beam/Cargo.toml
--example beam_v2_trades_fixture --quiet`. The receipt records the producer,
stdout and warning-stream hashes, exit-success evidence, and Rust/Cargo 1.99.1.
The output exactly matched a separate local harness run. `fixtures/v2-trades.json`
maps its compiled metas to the source field names and carries historical
whole-file source hashes. The Node test compares selected account declarations
with that map; unrelated future source edits need not match the old full hashes.
No handler, Solana runtime, wallet, RPC or captured original binary was run.

All fixture account keys are synthetic. In a real native-SOL V2 trade,
`quoteMint` would need to be WSOL and account constraints, PDAs, token programs,
recipient membership and any remaining accounts would need runtime validation.
This fixture does **not** supply valid values for those requirements. It is
not an executable trade example.

## Outstanding execution boundary

The retained `buy_v2` and `sell_v2` handlers still require inherited Pump
`fee_config` and `fee_program` accounts and call the inherited fee calculation.
The `sharing_config` dependency is also inherited. Beam has not rehosted or
verified these fee-program namespaces and authorities for its own program ID.
The builders do not derive or authenticate those accounts, compute an effective
fee, handle creator payouts, or verify quote-mint eligibility. They do not
connect wallets, sign, submit, trade, or deploy. The development Beam program
ID is undeployed. Host serialization and account-meta agreement do not prove
SBF behavior, CPI success, transaction rollback, or fee conservation.

This lane does not overlap the separate CreateV2 builder, creator-fee client,
or raw event decoder work. It does not edit their modules or fixtures.

Run the local checks from the repository root:

```sh
node --test sdk/beam-native.test.mjs sdk/beam-v2-trades.test.mjs
git diff --check
```

At this base commit, the SDK CI workflow runs only `beam-native.test.mjs`.
The separate raw-event SDK lane is updating that workflow to include all
`sdk/*.test.mjs`; land that gate before relying on CI for this new suite.
