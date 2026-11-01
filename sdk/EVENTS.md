# Strict local event decoding

Lane baseline: Beam main `b8120a8a58b1375ba3b8f0d2a590c49923e18021`.
This lane owns the raw `CreateEvent` and `TradeEvent` decoders and their local
fixtures/tests. It does not change the Anchor program, immutable archives,
economics, program identity, website or Raydium adapter.

## Interface and data

`beam-events.mjs` exports `decodeCreateEvent(bytes)`,
`decodeTradeEvent(bytes)` and `decodeBeamEvent(bytes)`. The first two return
camelCase fields; the dispatcher returns `{ eventName, data }`, where
`eventName` is `CreateEvent` or `TradeEvent`. Input is one complete raw Anchor
event payload, including its eight discriminator bytes. `Uint8Array`, Buffer
and nonzero-offset Uint8Array views are supported. Base64 strings, log text,
accounts, instruction data and event-CPI envelopes are not this interface.

All u64/i64 fields return `bigint`, including values representable as Number.
Public keys return base58 strings through the existing read-only
`beam-native.mjs` helper. u8/u16 values return exact Numbers; booleans require
wire bytes zero or one. Strings use their u32 byte lengths and fatal UTF-8,
preserving embedded NUL and a leading U+FEFF. Shareholder rows use the actual
`Pubkey + u16 share_bps` IDL definition. The final
`unparsed_trailing_field` is a required u64, returned as
`unparsedTrailingField`; its business meaning remains unknown.

The SDK accepts at most 1 MiB per payload as its explicit local resource
policy, rather than a recovered Anchor or chain limit. It checks declared
string/vector lengths against remaining bytes before decoding/allocating, and
requires the entire current layout with no trailing bytes. Historical or future
layouts need an explicit separate decoder. Input is copied before parsing so
later caller mutations cannot change the returned record; callers must supply
stable bytes during that copy.

## Source and authorization boundary

The layouts come from `programs/beam/src/lib.rs` at this baseline. TradeEvent's
Shareholder type is reexported through `trade_types.rs` from the retained
`programs/beam/idls/pump_fees.json`. `fixtures/events.json` binds selected
event-definition hashes, ordered fields, the Shareholder IDL shape/reexport,
discriminator derivation and dynamic field offsets. Its whole-file baseline
hashes are historical provenance; drift assertions concern the selected
definitions so unrelated contract edits do not invalidate these fixtures.

The five synthetic payloads in `fixtures/events-anchor.json` were actually
serialized with the locked Beam package's `CreateEvent::data()` and
`TradeEvent::data()`, by independently running
`tools/generate-event-fixtures.rs`. The receipt records baseline commit,
producer/output/stderr hashes, Cargo 1.89, Rust 1.94 and the exact command. The
host compiler emitted existing Beam/reference warnings; no warning-free or
handler/runtime claim is made. A separate source-layout writer independently
matches every payload byte, and `--check` reproduces the selected witnesses.
Existing native-SOL fixtures add an Anchor-produced CompleteEvent control for
the discriminator convention. Anchor lang is 0.31.1; locked event/serde macros
are 0.31.2 and Borsh is 0.10.4. Inspection of their checksum-matching registry
sources confirms default SHA-256 event tags, declaration-order serialization,
canonical bools and strict String UTF-8. No dependency source is copied.

For another Rust serialization run, stage the retained producer as the
temporary `programs/beam/examples/beam_events_fixture.rs` only in a disposable
baseline checkout, run `cargo run --locked --manifest-path
programs/beam/Cargo.toml --example beam_events_fixture`, then remove that
temporary example. Do not modify the immutable archive or manifests. The
Node-only byte/witness check is reproducible with
`node sdk/tools/generate-events-fixtures.mjs --check`.

Decoding says only that caller-supplied bytes match this layout. It does not
authenticate a chain event, program ID, transaction, slot, commitment, finality,
account ownership or economic truth. No result gains trusted provenance or
authorization from this parser. An indexer must separately verify transaction
execution and program context. The module performs no RPC, wallet, signing,
trade, service, money or deployment action. Wire-valid flags/share values are
decoded without asserting they satisfy Beam's business policy.

## Local acceptance

Before publication, tests must cover both event types and every field;
maximum u64 and signed i64 boundaries; exact shareholder nesting; empty and
multibyte/NUL/BOM strings; strict booleans and fatal malformed UTF-8; complete
truncation sweeps; forged lengths/counts; unknown/wrong discriminators;
trailing bytes; nonzero-offset views; input mutation isolation; and source
witness drift. Run these tests and the existing SDK tests under Node 22 and
Node 24, plus `python3 scripts/check_fork.py` and `git diff --check`.
Another lane must review the source, fixtures and rejection boundaries before
the separate draft PR. Contract/SBF/runtime/chain verification is outside this
SDK-only acceptance, and unavailable checks must be reported.

Observed local validation: the 10 new tests and 5 unchanged native-SOL tests
pass under Node 22.23.4 and 24.19.0. The tests include 1,937 truncation prefixes
and all 256 values of each of the six bool fields, plus the bounds/encoding
cases above. `scripts/check_fork.py` verifies 380 immutable Pump blobs and the
retained 62-identical/4-declared-change source delta. The shared SDK workflow
uses Node 22 and only `sdk/*.test.mjs backend/*/*.test.mjs`, with Bash nullglob;
website TypeScript/browser tests keep their separate workflow. These results
do not establish chain authenticity, event emission by a deployed Beam binary,
SBF behavior or the full launch/trade path.
