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

Website and backend contributors can choose a bounded [product lane](docs/TASKS.md)
without changing the Anchor contract. State the interface and data source,
mock the external provider locally, test unavailable/error states, and keep
wallet signing, real donations, live streams, and live trades disabled until a
separate reviewed release. Use observable product behavior and public
documentation for compatibility research; the pinned on-chain reconstruction
does not include Pump's proprietary website or service backends.

There is currently no repository-root license file. The licenses inside the
upstream snapshots apply to those snapshots; they do not establish a license
for every new Beam file. Maintainers need to clarify the project's own license
and contribution terms before inviting substantial external submissions.

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
before activation. Check CI on the exact pull-request head and obtain any
required formal approvals before merging. Do not self-approve, bypass branch
protections, or merge unresolved work.

For the merged native-SOL SDK fixtures, use Node.js 22 from the repository
root; `sdk/` has no package manifest:

```sh
node --test sdk/beam-native.test.mjs
```

For the merged website preview, use Node.js 24:

```sh
cd web
npm ci
npm run build
npm run test:unit
npm run format:check
```

The website's browser suite needs built output and Chromium in an isolated
test environment because its runner uses `--no-sandbox`. Website CI currently
checks build, unit tests and format; it does not run that browser suite.
Draft backend services have local fixture tests only and are not part of main.
An isolated local-bank harness, if used, exercises Beam with synthetic accounts
and valueless balances; it does not execute captured original Pump binaries or
prove real Raydium CPI behavior.

Host tests, mock accounts, and local simulations are welcome. Do not deploy to
a live network, use real wallets, sign live transactions, transfer funds, or
handle credentials as part of a normal contribution. If Beam is registered on
Slop, use its current private security-report workflow for suspected
exploitable findings. Otherwise use GitHub private vulnerability reporting if
available; if neither exists, wait for a private maintainer route. Never post
findings in public issues or pull requests.
