# Validation receipt — 1 November 2026

- Unmodified pinned Pump Main package: `cargo check --locked` passed.
- `python3 scripts/check_fork.py`: 380 original archive Git blobs verified. In the editable Pump Main fork, 57 retained files are byte-identical and nine are explicitly modified: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/create_v2.rs`, `src/pump_quote.rs`, `src/migrate.rs`, `src/migrate_v2.rs`, `src/multi_hop_curve_swap.rs`, and `src/creator/admin_cto.rs`. `src/beam_pumpswap.rs` is an allowed addition, not a retained Pump file.
- Default Beam configuration: 7 unit tests and 4 differential test groups passed.
- `experimental-raydium`: the same 11 host tests passed with the then-current CPMM candidate compiled as enableable. This was an earlier development state; activation is now blocked under every feature choice.
- Mainnet-ID configuration (`--no-default-features`): host `cargo check --locked` passed; migration gate stays off.

The differential groups check all 167 truncation boundaries of a retained curve layout, selected buy/sell/exact-input boundary matrices, fee rounding including u64 limits, and retained initialization/buy/sell instruction data. Semantic errors and retained source line numbers agree after normalizing build-directory prefixes.

The seven unit tests cover generated program identity, disabled Mayhem/PumpSwap policy, Raydium initialize encoding and privilege bits, configuration owner/PDA/discriminator checks, and the new fee allocation with dust and integer boundaries.

Host builds retain upstream warnings (including Anchor cfg and unused reconstructed helpers). These are not SBF or validator receipts. No live funds, wallet signing, deployment, Raydium execution, fee collection or buyback was performed. CI runs the reproducible checks from the committed tree.

## After migration accounting PR #2

At development-branch merge commit `65c96b5`, both default and
`experimental-raydium` host suites passed 10 unit tests and four differential
test groups; the no-default-features host check and 380-blob provenance check
also passed. PR #2's exact-head GitHub Actions run `41331629134` succeeded.
The added tests cover checked post-completion quote, fee/rent liability
backing, donor surplus, zero-liquidity and overflow boundaries, and account
aliases. They do not exercise a Raydium CPI, local-validator fee sweep,
rollback, replay, or real account substitution.

## AMM v4 direction and fee correction

After the user's AMM v4 choice, the superseded CPMM `set_enabled(true)` and
`migrate_raydium` handlers were made fail closed under default and experimental
builds. The proposed Beam component arithmetic was updated to 42/5/3 nominal
basis points for community/treasury/founder. Creator payout funding is
undecided; the inherited source currently calculates creator fees separately.
Host tests cover arithmetic and compilation, not fee collection or an AMM v4
CPI. The [feature ledger](FEATURES.md) tracks the missing integration and
product behavior.

## PumpSwap quote and handler-body rejection

On this branch, `python3 scripts/check_fork.py` still verifies 380 archive
blobs. The retained package is 57 byte-identical files and nine declared
edits. Default and `experimental-raydium` `cargo test --locked` each passed
20 unit tests and four differential groups. `cargo check --locked
--no-default-features` passed. `node --test sdk/beam-native.test.mjs
sdk/no-pumpswap-surface.test.mjs` passed.

The new unit tests reject a decoded historical PumpSwap pool, keep legacy
migrate / multi-hop / CTO instruction bytes equal to the reference, and check
that those handler bodies return before their PumpSwap CPIs. They do not
execute a Solana CPI or prove Anchor skips PDA constraints. The
[removal audit](PUMPSWAP_REMOVAL.md) lists what is still unresolved.
