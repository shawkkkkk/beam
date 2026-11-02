# Beam

Beam is an independent development fork of a community reconstruction of
Pump's on-chain launchpad. The intended venue after the bonding curve is
Raydium AMM v4; Mayhem is disabled, and finished Beam must have no operational
PumpSwap fallback. The goal is to preserve other observable launchpad behavior
where the venue and independently built services permit it.

**Current result:** an undeployed Anchor source fork, local ABI fixtures, and a
sample-data website preview. Beam is not audited or ready for live use. The
superseded Raydium CPMM migration is blocked in every build. AMM v4 has
source-level guards and draft synthetic tests, but no Beam migration entrypoint
or verified real Raydium CPI. The modified Beam binary cannot be byte-identical
to Pump's binary.

## Contribution focus

Choose one bounded gap from the [task ledger](docs/TASKS.md), check open PRs for
overlap, and record the base commit, affected paths, tests, and remaining risks.
The [contributor guide](CONTRIBUTING.md) gives commands and the maintainer
review sequence. A passing mock or host test is evidence for that narrow case,
not proof of a deployable launchpad.

At the 1 November 2026 maintainer check, Beam's
[Slop listing](https://slop.cash/projects/beam) was paused with a security
scan queued. Its current status could not be verified on 2 November. Confirm
current maintainer direction before treating a listed task as available or
resuming feature work. A Slop task's automatic closure does not approve a
GitHub PR.

## Progress at a glance

This is a **development** snapshot from 2 November 2026. **Merged** means
source is on `main`, **Draft** means it is in an unmerged PR, and **Blocked**
means a required implementation or decision is missing. These labels are not
completion percentages or production approval.

### Contract and source compatibility

| Goal | State | Evidence and remaining work |
| --- | --- | --- |
| Preserve the recovered Pump source | **Merged source** | The pinned archive passes [380 Git-blob checks](docs/VALIDATION.md). Of the retained Pump package, 62 files are byte-identical and four changes are declared. This is source evidence, not Beam binary identity. |
| Launch tokens and trade on the curve | **Merged development code** | [PR #1](https://github.com/shawkkkkk/beam/pull/1) contains Anchor handlers and selected host tests. Complete instruction execution and SBF behavior remain unverified. |
| Remove Mayhem and PumpSwap | **In progress** | Mayhem creation and legacy PumpSwap entry paths fail closed; [PR #12](https://github.com/shawkkkkk/beam/pull/12) blocked a nested graduated quote. Router, creator/sharing, CTO and donation dependencies still need compatible replacements. |
| Graduate to Raydium AMM v4 | **Blocked; guards in draft** | [Merged PR #9](https://github.com/shawkkkkk/beam/pull/9) adds source-only ABI guards. Draft [#22 → #29 → #30](docs/TASKS.md#draft-review-queue-prs-2234) and [#25 → #33 → #34](docs/TASKS.md#draft-review-queue-prs-2234) add account, funding, synthetic settlement and snapshot checks. No real migration CPI, LP-rights design or Token-2022 graduation exists. |
| Enforce Beam fees and creator payouts | **Blocked; math and disabled policy in draft** | [PR #5](https://github.com/shawkkkkk/beam/pull/5) merged provisional 42/5/3 arithmetic. Draft [#23/#24/#26](docs/TASKS.md#draft-review-queue-prs-2234) test stricter caps and disabled state. Live collection, selected creator wallet payout and post-graduation rights are unresolved. |

### Website, frontend, and SDK

| Goal | State | Evidence and remaining work |
| --- | --- | --- |
| Beam-branded discovery and token pages | **Merged preview** | [PR #6](https://github.com/shawkkkkk/beam/pull/6) added a responsive UI with sample market data and browser-local drafts. It has no live chain feed. |
| Create, wallet, and trade flows | **Blocked; local fixtures in draft** | [PR #7](https://github.com/shawkkkkk/beam/pull/7) merged native-SOL ABI fixtures; [#13/#18/#20/#27](docs/TASKS.md) add distinct creator, CreateV2, event and V2 trade drafts. There is no deployed Beam ID, finalized IDL, wallet signing or trade submission. |
| Creator wallet, charity, callouts, livestream screens | **Local plans in draft** | [#17 → #28 → #31 → #32](docs/TASKS.md#draft-review-queue-prs-2234) label unverified and unavailable states. They do not provide authorized donations, calls, streams or migration. |

### Backend and integrations

| Goal | State | Evidence and remaining work |
| --- | --- | --- |
| Read-only market/indexer API | **Fixture drafts; live feed blocked** | [#16](https://github.com/shawkkkkk/beam/pull/16) has local replay/query fixtures; [#17](https://github.com/shawkkkkk/beam/pull/17) has a web adapter. Their schema needs reconciliation. Neither authenticates a chain feed. |
| Creator sharing and payout service | **Blocked** | Retained on-chain source has creator paths, but Beam still depends on inherited Pump fee configuration. No Beam-controlled payout integration is verified. |
| Charity donations | **Mock draft; live use blocked** | [#14](https://github.com/shawkkkkk/beam/pull/14) provides a disabled local registry/provider mock. There is no authorized Donate.gg integration or real settlement. |
| Callouts and livestreaming | **Mock drafts; live use blocked** | [#15](https://github.com/shawkkkkk/beam/pull/15) and [#21](https://github.com/shawkkkkk/beam/pull/21) test local moderation and lifecycle behavior; no production provider or playable live stream exists. |

See [implementation status](docs/STATUS.md), [feature matrix](docs/FEATURES.md)
and the [dated PR dependency queue](docs/TASKS.md) for more detail.

## Required before live use

| Requirement | Current boundary |
| --- | --- |
| Verified AMM v4 migration and liquidity | Draft guards and synthetic local-bank tests do not execute the real Raydium program. Authenticate account state and receipts, test rollback and replay, and decide LP custody/fee rights before enabling migration. |
| Token compatibility | Raydium AMM v4 accepts classic SPL Token mints; Beam's curve also retains restricted Token-2022 support. A safe graduation plan for those launches is unresolved. |
| Enforced and disclosed fees | Beam's proposed allocation is not wired into trades. Decide creator funding, vault authority, community distribution, migration charges and all-in quote behavior. |
| Complete client and services | Regenerate the Beam IDL; add authenticated indexer, wallet/client and authorized creator, charity, callout and media integrations. Sample data and mocks cannot be presented as live observations. |
| Release evidence and authorization | Complete SBF/local-validator tests with real permitted program dependencies, independent security review and maintainer-controlled deployment decisions. This repository deploys nothing. |

## Evidence and components

| Component | Location | What its current evidence establishes |
| --- | --- | --- |
| Community Pump reconstruction | [`upstream/pumpfun-source`](upstream/pumpfun-source/README.md), pinned at [`5703df0`](https://github.com/lalalune/pumpfun-source/tree/5703df02131ab944408e0cf679b4bef9c22e1a3d) | Pristine reference archive with retained notices; upstream's historical reconstruction claim is not a Beam binary match. |
| Editable Beam contract | [`programs/beam`](programs/beam/Cargo.toml) | Anchor development code, declared source differences and selected host tests; no complete runtime parity or deployment. |
| Raydium references | Historical [CPMM `3d69122`](https://github.com/raydium-io/raydium-cp-swap/tree/3d69122790e97cd6aff24e8ba95940793773f58c); selected AMM v4 [source `91739d8`](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6) | Source pins for ABI research. CPMM is superseded and disabled; AMM v4 source is not a verified deployed-bytecode or CPI receipt. |
| Local client and UI | [`sdk/`](sdk/README.md), [`web/`](web/README.md) | ABI fixtures and sample-data preview; no wallet, RPC-authenticated market data or transaction service. |
| Backend drafts | [Task ledger](docs/TASKS.md#website-and-backend-contribution-lanes) | Local fixture indexer, charity, callout and livestream work in unmerged PRs, not services on `main`. |

Source equality, host compilation, synthetic bank tests and real CPI execution
are different evidence levels. None implies a whole-program binary match,
production safety or a promised market outcome. The historical ATH/fee
comparisons motivated Beam; they do not establish that a venue caused token
performance.

## Planned ordinary fee

| Allocation | Proposed rate | Share of 0.50% |
| --- | ---: | ---: |
| Community rewards and buybacks | 0.42% | 84% |
| Treasury | 0.05% | 10% |
| Founder | 0.03% | 6% |
| **Total** | **0.50%** | **100%** |

Treasury plus founder are 8 basis points under platform control. Creator
payout behavior and creator-selected wallet support remain requirements.
Whether creator payouts come **from** the 0.42% community share or add to the
trade charge is undecided, so 0.50% is **not** an all-in quote. A 50/50
holder-reward/buyback split is only a test assumption; beneficiaries, custody
and spending rules are not approved. Draft [#23](https://github.com/shawkkkkk/beam/pull/23)
corrects dust caps, but no live Beam collection is wired. Raydium pool and
retained migration fees are separate. See [economics](docs/ECONOMICS.md).

An optional three-second, buy-side 99%-to-base launch-fee quote is
**disabled and test-only**. Its initial 99% is modeled on gross buyer debit,
not net base; the extra charge has no approved recipient or withdrawal path.
It does not detect bots or promise a price outcome.

## Get started

Prerequisites: Git, Python 3, a Rust/Cargo toolchain, Node.js 22 for the SDK
and Node.js 24 with npm for the website. Inspect installation hooks and use
the repository lockfiles. No wallet, private key or live RPC is needed.

```sh
git clone https://github.com/shawkkkkk/beam.git
cd beam
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
node --test sdk/beam-native.test.mjs
(cd web && npm ci && npm run build && npm run test:unit && npm run format:check)
```

To view the sample-data website, run `cd web && npm run dev` after `npm ci`.
The Chromium interaction suite is separate from hosted website CI and requires
an isolated test environment; see [contributor checks](CONTRIBUTING.md#check-the-change).
The draft `tests/runtime-harness` has its own locked commands in that guide.
Build flags do not activate the disabled CPMM adapter or provide an AMM v4
migration entrypoint.

## Repository layout

- `programs/beam/`: editable Anchor contract and host compatibility tests.
- `upstream/`: pinned third-party source snapshots and notices; keep them immutable.
- `scripts/check_fork.py`: source provenance and declared-difference check.
- `sdk/`: local native-SOL ABI fixtures and decoder/builder drafts.
- `web/`: Beam-branded sample-data preview.
- `docs/`: economics, feature matrix, status, validation and contributor tasks.
- `.github/workflows/`: current hosted checks; verify which changed tests each workflow actually runs.

## Open priorities and contribution

The immediate engineering gates are authenticated AMM v4 migration, LP and
creator fee rights, Token-2022 graduation, Beam-controlled fee routing,
PumpSwap-dependent feature replacements, a final Beam client and authorized
off-chain services. They remain **blocked or in draft** while feature work is
paused; this list does not reopen them. See [task ownership](docs/TASKS.md)
before taking a lane.

Start with [CONTRIBUTING.md](CONTRIBUTING.md),
[AGENTS.md](AGENTS.md), [status](docs/STATUS.md),
[validation](docs/VALIDATION.md), and [economics](docs/ECONOMICS.md).
Use the [maintainer review checklist](CONTRIBUTING.md#maintainer-review-and-merge)
for stacked PRs and exact-head CI. Suspected exploitable issues belong in the
current private security-report channel, never a public issue or PR.

Beam has no repository-root license yet. Archived sources retain their own
licenses; maintainers must clarify terms for new Beam code before inviting
substantial external submissions.
