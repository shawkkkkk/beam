# Beam

A Solana launchpad forked from the completed community reconstruction of Pump, with Raydium AMM v4 as the selected secondary venue and Mayhem disabled.

**Status: undeployed source fork, local ABI fixtures, and sample-data website preview. Not audited or complete. The selected AMM v4 migration has no execution adapter; the older CPMM candidate is disabled in every build.**

## Source baseline

- Pump: [`5703df0`](https://github.com/lalalune/pumpfun-source/tree/5703df02131ab944408e0cf679b4bef9c22e1a3d), preserved unmodified under `upstream/pumpfun-source`.
- Earlier Raydium CPMM reference snapshot: [`3d69122`](https://github.com/raydium-io/raydium-cp-swap/tree/3d69122790e97cd6aff24e8ba95940793773f58c). Its gated adapter is superseded; AMM v4 integration is in progress.
- Selected AMM v4 ABI reference: [official `raydium-amm` source at `91739d8`](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6). This is a source pin, not a deployed-bytecode match or an integrated Beam adapter.
- Editable Beam contract: `programs/beam`.

The goal is to retain Pump's instruction/account compatibility and behavior outside documented Beam exceptions. A different program identity, migration destination, disabled Mayhem and revised fees produce a different binary. Full binary or behavioral parity is not established here.

## Progress at a glance

These are **development** milestones as recorded on 2 November 2026, not a live launch. “Done” means the
named source/check is merged; “In review” means a separate change is pending;
“Blocked” means the feature needs an implementation or a verified external
dependency. No percentage can honestly describe whole-product parity yet.

### Contract and byte evidence

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Preserve the recovered Pump source | **Done** | The pinned community archive passes [380 Git-blob checks](docs/VALIDATION.md). In Beam's retained Pump package, 62 files remain byte-identical and four are declared changes. This is source evidence, not a Beam binary match. |
| Keep Pump-style launch and curve trades | **In progress** | [Merged PR #1](https://github.com/shawkkkkk/beam/pull/1) contains the Anchor handlers and selected host tests. Full instruction execution, SBF and live behavior remain unverified. |
| Remove Mayhem and PumpSwap | **In progress** | Mayhem and legacy PumpSwap entry paths fail closed, with no PumpSwap fallback. Retained router, creator and fee/donation dependencies still need compatible replacements; blocked features are not considered complete. |
| Graduate to Raydium AMM v4 | **Blocked; guards in draft review** | [PR #5](https://github.com/shawkkkkk/beam/pull/5) retired CPMM; merged [PR #9](https://github.com/shawkkkkk/beam/pull/9) adds source-only AMM v4 ABI guards. Draft [#22 → #29 → #30](docs/TASKS.md#draft-review-queue-prs-2234) and [#25 → #33 → #34](docs/TASKS.md#draft-review-queue-prs-2234) add preflight, funding, synthetic settlement and snapshot checks. No AMM v4 migration entrypoint, real Raydium CPI, LP-rights policy or Token-2022 graduation exists. |
| Apply Beam 42/5/3 fees and preserve creator payouts | **Math only on main; routing blocked** | [PR #5](https://github.com/shawkkkkk/beam/pull/5) merged provisional allocation math; [PR #11](https://github.com/shawkkkkk/beam/pull/11) recorded the unresolved creator-funding choice. Draft [#23/#24/#26](docs/TASKS.md#draft-review-queue-prs-2234) add strict caps, disabled policy state and a synthetic initializer test. Actual collection, creator-fee compatibility, selected payout wallet and post-graduation rights still need implementation and runtime tests. |
| Match Pump byte for byte | **Not attainable for modified Beam binary** | The requested venue, Mayhem and fee changes alter code and program identity. The target is tested behavior/interface compatibility for unaffected features, tracked in the [feature matrix](docs/FEATURES.md). |

### Website and frontend

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Pump-style experience with Beam branding | **Preview merged** | [PR #6](https://github.com/shawkkkkk/beam/pull/6) merged a responsive **sample-data preview** selected from the closed, unmerged alternate [PR #3](https://github.com/shawkkkkk/beam/pull/3). It is not a live Pump clone or canonical client. |
| Create, wallet and trade flows | **Blocked; SDK fixtures in draft review** | The preview saves local drafts; it does not upload metadata, connect wallets, sign or trade. Draft [#27](https://github.com/shawkkkkk/beam/pull/27) adds V2 trade ABI fixtures; a Beam IDL, full client and verified chain data remain open. |
| Creator wallet, charity, callouts and livestream screens | **Local plans merged; services in review** | PR #6 has draft fields only. Separate draft [charity PR #14](https://github.com/shawkkkkk/beam/pull/14), [callouts PR #15](https://github.com/shawkkkkk/beam/pull/15) and [livestream PR #21](https://github.com/shawkkkkk/beam/pull/21) provide local backend mocks, not authorized providers, live actions or playable streams. |
| Client and frontend data | **Native-SOL fixtures merged; drafts in review** | [PR #7](https://github.com/shawkkkkk/beam/pull/7) merged create/buy/sell ABI fixtures. Draft [#13](https://github.com/shawkkkkk/beam/pull/13), [#18](https://github.com/shawkkkkk/beam/pull/18), [#20](https://github.com/shawkkkkk/beam/pull/20) and [#27](https://github.com/shawkkkkk/beam/pull/27) own distinct SDK slices; [#17 → #28 → #31 → #32](docs/TASKS.md#draft-review-queue-prs-2234) own read-only web work. No deployed Beam ID, complete SDK or authenticated feed exists. |

### Backend and integrations

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Read-only market/indexer API | **Local drafts in review** | [PR #16](https://github.com/shawkkkkk/beam/pull/16) owns a fixture backend under `backend/indexer/`; [PR #17](https://github.com/shawkkkkk/beam/pull/17) owns a read-only `web/` adapter. Their schema must be reconciled. Neither is a live chain feed; preview charts are fixtures. |
| Creator fee, wallet and sharing service | **Blocked** | On-chain source retains creator vault/collection paths, but Beam still depends on inherited Pump fee configuration. No Beam-controlled payout integration is verified. |
| Donate.gg charity donations | **Local draft in review; live use blocked** | [PR #14](https://github.com/shawkkkkk/beam/pull/14) owns a disabled local charity mock. There is no authorized provider or settlement service; no real donations occur. |
| Callouts and livestreaming | **Local drafts in review; live use blocked** | [PR #15](https://github.com/shawkkkkk/beam/pull/15) owns a local callout service; [PR #21](https://github.com/shawkkkkk/beam/pull/21) owns a mock livestream lifecycle. Neither is on main or connected to a live provider. |

See [implementation status](docs/STATUS.md) and [contributor tasks](docs/TASKS.md)
for evidence and bounded ways to help with the website, frontend and backend.

## Provisional Beam curve fee

| Allocation | Per trade | Share of fee |
| --- | ---: | ---: |
| Community allocation (uses pending) | 0.42% | 84% |
| Treasury | 0.05% | 10% |
| Founder | 0.03% | 6% |
| Total | **0.50%** | **100%** |

The requested ordinary Beam allocation is 50 nominal basis points: 42 community, 5 treasury and 3 founder. Treasury plus founder are 8 basis points under platform control. Existing creator payout behavior must be preserved, but whether creator payouts come from the community share or are charged in addition is undecided. Therefore 0.50% is **not yet an all-in trading quote**. The current helper rounds this component once; draft [#23](https://github.com/shawkkkkk/beam/pull/23) corrects dust arithmetic to cap founder and treasury against the trade base. Holder rewards and flagship buybacks are proposed uses of the community amount remaining after any creator allocation, if that funding route is chosen. A 50/50 split of that remainder is only a provisional test setting; eligibility, the flagship mint, and distribution rules are undecided. Collection and distribution are not yet wired to trading instructions. Raydium's own pool fees are separate. See [economics](docs/ECONOMICS.md) and the [feature matrix](docs/FEATURES.md).

## First contract slice

- New undeployed development identity.
- Mayhem creation/enablement rejected; inert serialized fields and account slots retained.
- Original PumpSwap migration, signed router and AMM-dependent CTO handlers blocked.
- Earlier administrator-pinned CPMM migration candidate is disabled in every build; its account checks, caller-funded setup, WSOL funding and LP burn remain development source only. Runtime migration rollback remains unverified.
- Differential tests compile the pristine upstream crate beside Beam and compare selected retained math, layouts and instruction data.

The current gated migration code targets Raydium CPMM, an earlier candidate superseded by the selected Raydium AMM v4 direction. It is not an AMM v4 adapter. It also burns LP and cannot harvest fees from that burned position. An AMM v4 integration, principal-lock/fee-rights design, and validator testing are required before any migration is enabled.
[Merged PR #12](https://github.com/shawkkkkk/beam/pull/12) separately guards a nested graduated PumpSwap quote in `create_v2`. It fails closed; it does not supply an AMM v4 quote.

An optional three-second, buy-side launch-fee quote prototype is **disabled and test-only**. Its initial 99% models the fee as a share of the **gross buyer debit**: for a 100-unit debit, 99 units would be fee and 1 unit base quote. A fee of 99% of **net base** would instead be about 49.75% of the gross debit (0.99 / 1.99); these are different models. The prototype keeps its launch surcharge separate from the ordinary 42/5/3 allocation and has no recipient or withdrawal path for that surcharge. It has no enabled contract path, sell-handler change, live trades or promised outcome. Creator funding, account custody and adversarial runtime behavior remain unresolved.

## Website preview

`web/` contains a responsive Beam interface with original blue, purple, and
black ray-gun branding. It previews token discovery, search, a watchlist,
token details, and local token drafts using sample market data. It is not a
wallet client and does not create tokens or submit trades. The displayed fee
allocation follows the proposal above; fee collection is not live.

Use Node.js 24 to run the preview:

```sh
cd web
npm ci
npm run build
npm run format:check
npm run dev
```

## Verify

Requires Rust/Cargo and Python 3; no secret or wallet.

```sh
python3 scripts/check_fork.py
cargo test --locked --manifest-path programs/beam/Cargo.toml
cargo test --locked --manifest-path programs/beam/Cargo.toml --features experimental-raydium
cargo check --locked --manifest-path programs/beam/Cargo.toml --no-default-features
```

Default builds retain earlier Raydium CPMM devnet IDs for the superseded adapter; `--no-default-features` selects its mainnet IDs. Every current build rejects activation and execution of that superseded adapter, including `experimental-raydium`. AMM v4 has no callable migration entrypoint. These flags do not deploy anything.

Beam's declared ID is an undeployed placeholder. See [implementation status](docs/STATUS.md). The original MIT and third-party notices remain in the immutable archive.

## Contributing

Start with [contributor guidance](CONTRIBUTING.md), [agent instructions](AGENTS.md),
the [feature matrix](docs/FEATURES.md), and the [task ledger](docs/TASKS.md),
which includes bounded website/backend lanes. [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; [PR #2](https://github.com/shawkkkkk/beam/pull/2)
added migration-accounting guards to its development branch. Check current PR
heads before starting a task. [PR #12](https://github.com/shawkkkkk/beam/pull/12)
merged the nested quote guard. Draft PRs #13–#34 own separate SDK, backend,
web-adapter, contract guard and documentation slices listed in the [task ledger](docs/TASKS.md).
Beam is
[registered on Slop](https://slop.cash/projects/beam), but its listing was
paused with a security scan queued at the 1 November maintainer check. The
listing could not be verified on 2 November; this repository does not claim
scan clearance, an installed project skill, or
funded bounties. Beam has no repository-root license yet; the archived sources
retain their own licenses.
