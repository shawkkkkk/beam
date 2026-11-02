# Contributing to Beam

Beam's current contract lives in `programs/beam`. Start with the
[implementation status](docs/STATUS.md), [task ledger](docs/TASKS.md), and
[agent guidance](AGENTS.md). The Pump and Raydium source snapshots under
`upstream/` are reference inputs; preserve their licenses and notices.

## Before you start

Use Python 3 and a Rust/Cargo toolchain for contract checks, Node.js 22 for
the SDK, and Node.js 24 plus npm for `web/`. Install dependencies from the
project lockfiles and inspect install scripts first. No wallet, RPC key,
private credential, or live network is needed for the checks below. The
[Slop status note](docs/STATUS.md#contribution-and-security-status) is a dated
observation, not permission to begin a listed task or merge a PR.

## Choose a scoped change

Check open pull requests and remote branches before beginning. [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; inspect the current target branch
and [task ledger](docs/TASKS.md) for its latest state. Claim one lane in your
PR description, list intended paths and dependencies, and ask the maintainer
to resolve any overlap before editing another active lane. Avoid importing a
separate model's assumptions into the Anchor program. The
[draft review queue](docs/TASKS.md#draft-review-queue-prs-2234) records the
stacked #22–#34 work; check live PR heads because that snapshot can age.

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

Report exact commands, tested commit, and results. If a check is unavailable,
say which one and why. The differential host tests cover selected layouts, math, and
instruction encodings; they do not establish complete runtime parity. Changes
to CPI accounts, funds, migrations, or fee rights need SBF/local-validator
tests for success, failure rollback, authority checks, replay, and accounting
before activation. Do not treat a host pass as a real program or deployed
Raydium test.

For the merged native-SOL SDK fixtures, use Node.js 22 from the repository
root; `sdk/` has no package manifest:

```sh
node --test sdk/beam-native.test.mjs
```

For SDK drafts, also run each changed `sdk/*.test.mjs` file explicitly.
The SDK workflow on `main` runs only `beam-native.test.mjs`; a green job
does not prove other draft fixtures ran unless the workflow is updated.

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
checks build, unit tests and format; it does not run that browser suite. In
such an isolated environment, run `npm run test:browser` after `npm run build`.

Draft backend services are not on `main`. With Node.js 22 from the repository
root, use the exact local fixture test for the lane you changed:

```sh
node --test backend/charity/charity.test.mjs
node --test backend/callouts/service.test.mjs
node --test backend/indexer/index.test.mjs sdk/beam-native.test.mjs
node --test backend/livestream/livestream.test.mjs
```

Run only commands for paths present in your checked-out branch. These
services do not start an authenticated production API, media provider or
donation provider. Their tests are not included in `main` CI until a reviewed
workflow adds them.

If `tests/runtime-harness/Cargo.toml` exists in your checked-out branch, run
its separate locked suite from the repository root in both modes:

```sh
cargo test --locked --manifest-path tests/runtime-harness/Cargo.toml
cargo test --locked --manifest-path tests/runtime-harness/Cargo.toml --features experimental-raydium
```

The harness is in draft PR #25 and its descendants, not on `main`. It uses
synthetic accounts and valueless balances; it does not execute captured
original Pump binaries or prove real Raydium CPI behavior. The normal
`programs/beam` tests above do not run this separate manifest.

The merged AMM v4 validator is another separate crate. With Rust 1.99.1,
run its pinned-source local CPI tests from the repository root:

```sh
cargo test --locked --manifest-path tests/amm-v4-validator/Cargo.toml -- --test-threads=1
```

These tests invoke the pinned Raydium Rust processor through a synthetic
local probe. They do not execute Beam's migration handler or verify deployed
Raydium bytecode. See the [validator scope](tests/amm-v4-validator/README.md).

Host tests, mock accounts, and local simulations are welcome. Do not deploy to
a live network, use real wallets, sign live transactions, transfer funds, or
handle credentials as part of a normal contribution. If Beam is registered on
Slop, use its current private security-report workflow for suspected
exploitable findings. Otherwise use GitHub private vulnerability reporting if
available; if neither exists, wait for a private maintainer route. Never post
findings in public issues or pull requests.

## Maintainer review and merge

1. Confirm the live project status, requested scope, repository permissions,
   root-license/contribution terms, PR target and dependency branches. Slop's
   security scan or automatic task closure does not approve a GitHub merge.
   While the listing is paused, do not infer that feature work or a release is
   authorized from a draft PR or a passing check.
2. Review the **incremental diff against the declared target** and the
   **combined tree against `main`**. For stacks, follow the dependency order
   in [the task ledger](docs/TASKS.md#draft-review-queue-prs-2234). Resolve
   conflicts and repeated cherry-picks by comparing resulting source and test
   behavior. Never merge an unreviewed cumulative diff because the PR itself
   appears small.
3. Require the relevant local commands above, archive provenance, source
   review, exact-head hosted checks, and any repository-required formal
   approvals. Inspect failures, skipped jobs, and whether CI actually
   discovers the new tests. A new commit, retargeted base, dependency merge,
   conflict resolution, or changed workflow invalidates prior exact-head
   evidence; rerun affected checks and refresh review on the final tree.
4. Keep unresolved security, fee custody, creator funding, LP rights,
   Token-2022 graduation, authenticated indexer and live-service work in
   draft. A disabled helper may be merged as development source only when its
   scope and limitations are explicit and the current maintainer decision
   allows it. Host or mock tests never authorize activation or deployment.
5. Merge only when the PR is ready, its exact reviewed head and base still
   match, required protections and formal approvals are satisfied, and the
   maintainer has authority for that scope. Do not self-approve, bypass rules,
   enable auto-merge to evade an unresolved blocker, or delete another
   contributor's branch. After a dependency lands, rebase or retarget its
   children and repeat the combined-tree checks.
