# Working on Beam

These instructions apply to the Beam repository. They do not apply to the
archived copy at `upstream/pumpfun-source`, which has its own `AGENTS.md`.

## Start with the current work

1. Read `README.md`, `docs/STATUS.md`, `docs/ECONOMICS.md`, `docs/TASKS.md`,
   and `CONTRIBUTING.md`. Check the working tree, remote branches, and open
   pull requests before choosing a task. Inspect the intended target branch.
2. Use `programs/beam` as the editable Anchor program. Its retained Pump Main
   modules, Beam policy/economics/Raydium modules, and compatibility tests are
   the contract baseline. Do not start a second launchpad implementation.
3. Choose one bounded task and record its scope, base commit, intended
   behavior, tests, and remaining risks in the pull request description.
   Check `docs/TASKS.md` and active pull requests to avoid duplicate work.
4. Work in a separate branch. Do not rewrite another contributor's branch or
   merge competing architectures or fee policies without a reviewed decision.

## Source and behavior boundaries

- `upstream/pumpfun-source` pins the community reconstruction at
  `5703df02131ab944408e0cf679b4bef9c22e1a3d`. Its MIT and third-party
  notices stay with the source. `upstream/raydium-cp-swap` pins reference
  Raydium CPMM files at `3d69122790e97cd6aff24e8ba95940793773f58c`.
  Treat both as reference snapshots; edit the Beam program instead.
- `scripts/check_fork.py` verifies archive Git blobs and the declared Beam
  source delta. Document every intentional change to retained Pump source.
  Source equality, host tests, and historical reconstruction receipts do not
  prove Beam's binary identity or full runtime parity with Pump.
- Mayhem creation and enablement are blocked. Legacy PumpSwap migration,
  signed router, and AMM-dependent CTO handlers fail closed. Do not reconnect
  them to PumpSwap or claim that the retained quote-chain code routes through
  Raydium.
- Raydium CPMM migration is a gated candidate. Do not enable it based on host
  tests alone. Current LP burning does not leave Beam an LP fee claim for
  recurring buybacks. Principal custody and fee rights remain unresolved.
- `beam_economics.rs` contains provisional fee-allocation arithmetic; the
  inherited Pump fee CPI still governs real trade collection. The community
  share's holder-reward/buyback split is a provisional test setting. Holder
  eligibility, the flagship mint, vaults, and distribution rules are open.
  Distinguish curve fees from Raydium pool fees and never promise returns.

## Validation and review

Run these from the repository root for contract changes:

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

Use pinned lockfiles and inspect setup hooks before installing dependencies.
Review source changes, test output, and the account/fee effects in the pull
request. Host checks do not prove Solana CPI behavior, transaction rollback,
replay resistance, or production safety. Migration and fee-rights changes need
local-validator/SBF evidence and independent review before activation.

Host tests, mock accounts, and local simulations are normal contributor work.
Do not deploy to a live network, use real wallets, sign live transactions,
trade, move funds, collect secrets, or launch live services in a pull request.
A release requires a separate maintainer-controlled process. Respect branch
protections and leave unresolved or untested work in draft state.

If Beam is registered on Slop, report a suspected exploitable issue through
Slop's current private security-report workflow. Otherwise use GitHub private
vulnerability reporting if available. If neither private route exists, wait
for a private maintainer route. Never publish findings or sensitive data in a
public issue or pull request.
