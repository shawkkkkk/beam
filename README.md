# Beam

A Solana launchpad forked from the completed community reconstruction of Pump, with Raydium as the intended secondary venue and Mayhem disabled.

**Status: source fork and host-tested migration candidate. Not deployed. Not audited or complete. Raydium execution is disabled in default builds.**

## Source baseline

- Pump: [`5703df0`](https://github.com/lalalune/pumpfun-source/tree/5703df02131ab944408e0cf679b4bef9c22e1a3d), preserved unmodified under `upstream/pumpfun-source`.
- Raydium CPMM ABI: [`3d69122`](https://github.com/raydium-io/raydium-cp-swap/tree/3d69122790e97cd6aff24e8ba95940793773f58c).
- Editable Beam contract: `programs/beam`.

The goal is to retain Pump's instruction/account compatibility and behavior outside documented Beam exceptions. A different program identity, migration destination, disabled Mayhem and revised fees produce a different binary. Full binary or behavioral parity is not established here.

## Proposed Beam fee

| Allocation | Per trade | Share of fee |
| --- | ---: | ---: |
| Community rewards / buybacks | 0.45% | 90% |
| Beam operating treasury | 0.05% | 10% |
| Total | **0.50%** | **100%** |

The allocation engine rounds the combined charge once and gives split dust to the community. A provisional test setting would divide the community share equally between holder rewards and flagship buybacks; eligibility, the flagship mint, and distribution rules are undecided. Collection and distribution are not yet wired to trading instructions. Raydium's own pool fees are separate; this is not a promise of a 0.50% all-in post-graduation fee. See [economics](docs/ECONOMICS.md).

## First contract slice

- New undeployed development identity.
- Mayhem creation/enablement rejected; inert serialized fields and account slots retained.
- Original PumpSwap migration, signed router and AMM-dependent CTO handlers blocked.
- New administrator-pinned Raydium configuration and native-SOL migration candidate: canonical account checks, caller-funded setup, WSOL funding, LP receipt checks and atomic LP burning.
- Differential tests compile the pristine upstream crate beside Beam and compare selected retained math, layouts and instruction data.

The current migration candidate burns LP. It cannot harvest fees from that burned position. A principal-lock/fee-rights integration is required for the recurring-buyback design, and migration remains gated pending that work and validator testing.

## Verify

Requires Rust/Cargo and Python 3; no secret or wallet.

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

Default builds use Raydium devnet IDs with execution gated off. `experimental-raydium` permits an authorized configuration to enable the unqualified candidate. `--no-default-features` selects mainnet IDs and preserves the gate. Neither option deploys anything.

Beam's declared ID is an undeployed placeholder. See [implementation status](docs/STATUS.md). The original MIT and third-party notices remain in the immutable archive.

## Contributing

Start with [contributor guidance](CONTRIBUTING.md), [agent instructions](AGENTS.md),
and the [task ledger](docs/TASKS.md). [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; [PR #2](https://github.com/shawkkkkk/beam/pull/2)
added migration-accounting guards to its development branch. Check current PR heads before
starting a task. These documents prepare scoped work for contributors; no
external listing or reward terms are established by this repository.
