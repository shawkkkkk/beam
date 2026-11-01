# Contributing to Beam

Beam's current contract lives in `programs/beam`. Start with the
[implementation status](docs/STATUS.md), [task ledger](docs/TASKS.md), and
[agent guidance](AGENTS.md). The Pump and Raydium source snapshots under
`upstream/` are reference inputs; preserve their licenses and notices.

## Choose a scoped change

Check open pull requests and remote branches before beginning. [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; inspect the current target branch
and [task ledger](docs/TASKS.md) for its latest state. Avoid duplicating an
active lane or importing a separate model's assumptions into the Anchor program.

Make the smallest change that can be reviewed and tested. A pull request should
state its base commit, affected instructions/accounts, intentional differences
from retained Pump behavior, fee and custody effects, tests run, and remaining
runtime gaps. Keep unfinished migration, fee-rights, and client work in draft
state. Task descriptions in this repository are engineering scopes, not reward
offers or claims that an external listing has accepted a submission.

## Check the change

From the repository root, run:

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

Report exact commands and results. If a check is unavailable, say which one
and why. The differential host tests cover selected layouts, math, and
instruction encodings; they do not establish complete runtime parity. Changes
to CPI accounts, funds, migrations, or fee rights need SBF/local-validator
tests for success, failure rollback, authority checks, replay, and accounting
before activation. CI and independent review must pass; do not bypass branch
protections or merge unresolved work.

Host tests, mock accounts, and local simulations are welcome. Do not deploy to
a live network, use real wallets, sign live transactions, transfer funds, or
handle credentials as part of a normal contribution. If Beam is registered on
Slop, use its current private security-report workflow for suspected
exploitable findings. Otherwise use GitHub private vulnerability reporting if
available; if neither exists, wait for a private maintainer route. Never post
findings in public issues or pull requests.
