# Validation receipt — 1 November 2026

- Unmodified pinned Pump Main package: `cargo check --locked` passed.
- `python3 scripts/check_fork.py`: 380 original archive Git blobs verified. In the editable Pump Main fork, 62 retained files are byte-identical and four are explicitly modified: manifest, lockfile, `lib.rs`, and `create_v2.rs`.
- Default Beam configuration: 7 unit tests and 4 differential test groups passed.
- `experimental-raydium`: the same 11 host tests passed with the candidate compiled as enableable.
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
