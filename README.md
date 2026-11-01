# Beam

A Solana launchpad forked from the completed community reconstruction of Pump, with Raydium AMM v4 as the selected secondary venue and Mayhem disabled.

**Status: source fork and host-tested migration candidate. Not deployed. Not audited or complete. Raydium execution is disabled in default builds.**

## Source baseline

- Pump: [`5703df0`](https://github.com/lalalune/pumpfun-source/tree/5703df02131ab944408e0cf679b4bef9c22e1a3d), preserved unmodified under `upstream/pumpfun-source`.
- Earlier Raydium CPMM reference snapshot: [`3d69122`](https://github.com/raydium-io/raydium-cp-swap/tree/3d69122790e97cd6aff24e8ba95940793773f58c). Its gated adapter is superseded; AMM v4 integration is in progress.
- Selected AMM v4 ABI reference: [official `raydium-amm` source at `91739d8`](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6). This is a source pin, not a deployed-bytecode match or an integrated Beam adapter.
- Editable Beam contract: `programs/beam`.

The goal is to retain Pump's instruction/account compatibility and behavior outside documented Beam exceptions. A different program identity, migration destination, disabled Mayhem and revised fees produce a different binary. Full binary or behavioral parity is not established here.

## Provisional Beam curve fee

| Allocation | Per trade | Share of fee |
| --- | ---: | ---: |
| Community rewards / buybacks | 0.42% | 84% |
| Treasury | 0.05% | 10% |
| Founder | 0.03% | 6% |
| Total | **0.50%** | **100%** |

The requested Beam component is 50 nominal basis points with the split shown above. Existing creator fees and creator payout-wallet behavior must be preserved and disclosed separately, so 0.50% is **not** an all-in trading cost. The allocation engine rounds the Beam component once and gives split dust to the community. A provisional test setting divides the community share equally between holder rewards and flagship buybacks; eligibility, the flagship mint, and distribution rules are undecided. Collection and distribution are not yet wired to trading instructions. Raydium's own pool fees are separate. See [economics](docs/ECONOMICS.md) and the [feature matrix](docs/FEATURES.md).

## First contract slice

- New undeployed development identity.
- Mayhem creation/enablement rejected; inert serialized fields and account slots retained.
- Original PumpSwap migration, signed router and AMM-dependent CTO handlers blocked.
- New administrator-pinned Raydium configuration and native-SOL migration candidate: canonical account checks, caller-funded setup, WSOL funding, LP receipt checks and an LP burn in the same instruction. Runtime rollback remains unverified.
- Differential tests compile the pristine upstream crate beside Beam and compare selected retained math, layouts and instruction data.

The current gated migration code targets Raydium CPMM, an earlier candidate superseded by the selected Raydium AMM v4 direction. It is not an AMM v4 adapter. It also burns LP and cannot harvest fees from that burned position. An AMM v4 integration, principal-lock/fee-rights design, and validator testing are required before any migration is enabled.

## Verify

Requires Rust/Cargo and Python 3; no secret or wallet.

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

Default builds retain earlier Raydium CPMM devnet IDs for the superseded adapter. `--no-default-features` selects its mainnet IDs. Both configurations and `experimental-raydium` now reject activation of that adapter while AMM v4 work is pending. Neither option deploys anything.

Beam's declared ID is an undeployed placeholder. See [implementation status](docs/STATUS.md). The original MIT and third-party notices remain in the immutable archive.

## Contributing

Start with [contributor guidance](CONTRIBUTING.md), [agent instructions](AGENTS.md),
the [feature matrix](docs/FEATURES.md), and the [task ledger](docs/TASKS.md),
which includes bounded website/backend lanes. [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; [PR #2](https://github.com/shawkkkkk/beam/pull/2)
added migration-accounting guards to its development branch. Check current PR heads before
starting a task. These documents prepare scoped work for contributors; no
external listing or reward terms are established by this repository. Beam has
no repository-root license yet; the archived sources retain their own licenses.
