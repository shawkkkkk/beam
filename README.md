# Beam

A Solana launchpad forked from the completed community reconstruction of Pump, with Raydium AMM v4 as the selected secondary venue and Mayhem disabled.

**Status: source fork and host-tested migration candidate. Not deployed. Not audited or complete. Raydium execution is disabled in default builds.**

## Source baseline

- Pump: [`5703df0`](https://github.com/lalalune/pumpfun-source/tree/5703df02131ab944408e0cf679b4bef9c22e1a3d), preserved unmodified under `upstream/pumpfun-source`.
- Earlier Raydium CPMM reference snapshot: [`3d69122`](https://github.com/raydium-io/raydium-cp-swap/tree/3d69122790e97cd6aff24e8ba95940793773f58c). Its gated adapter is superseded; AMM v4 integration is in progress.
- Selected AMM v4 ABI reference: [official `raydium-amm` source at `91739d8`](https://github.com/raydium-io/raydium-amm/tree/91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6). This is a source pin, not a deployed-bytecode match or an integrated Beam adapter.
- Editable Beam contract: `programs/beam`.

The goal is to retain Pump's instruction/account compatibility and behavior outside documented Beam exceptions. A different program identity, migration destination, disabled Mayhem and revised fees produce a different binary. Full binary or behavioral parity is not established here.

## Progress at a glance

These are **development** milestones, not a live launch. “Done” means the
named source/check is merged; “In review” means a separate change is pending;
“Blocked” means the feature needs an implementation or a verified external
dependency. No percentage can honestly describe whole-product parity yet.

### Contract and byte evidence

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Preserve the recovered Pump source | **Done** | The pinned community archive passes [380 Git-blob checks](docs/VALIDATION.md). In Beam's retained Pump package, 62 files remain byte-identical and four are declared changes. This is source evidence, not a Beam binary match. |
| Keep Pump-style launch and curve trades | **In progress** | [Merged PR #1](https://github.com/shawkkkkk/beam/pull/1) contains the Anchor handlers and selected host tests. Full instruction execution, SBF and live behavior remain unverified. |
| Remove Mayhem and PumpSwap | **In progress** | Mayhem and legacy PumpSwap entry paths fail closed, with no PumpSwap fallback. Retained router, creator and fee/donation dependencies still need compatible replacements; blocked features are not considered complete. |
| Graduate to Raydium AMM v4 | **Blocked** | [PR #5](https://github.com/shawkkkkk/beam/pull/5) retired the wrong-venue CPMM path. [Merged PR #9](https://github.com/shawkkkkk/beam/pull/9) adds source-only AMM v4 ABI guards, not a tested migration adapter or Token-2022 graduation path. |
| Apply Beam 42/5/3 fees and preserve creator payouts | **Math done; routing blocked** | PR #5 merged allocation **math only**. Actual Beam collection, creator-fee compatibility, selected payout wallet and post-graduation rights need implementation and runtime tests. Whether creator payouts come from the 0.42% community share or an additional charge is undecided. |
| Match Pump byte for byte | **Not attainable for modified Beam binary** | The requested venue, Mayhem and fee changes alter code and program identity. The target is tested behavior/interface compatibility for unaffected features, tracked in the [feature matrix](docs/FEATURES.md). |

### Website and frontend

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Pump-style experience with Beam branding | **In review** | [PR #6](https://github.com/shawkkkkk/beam/pull/6) carries a responsive **sample-data preview** selected from the closed alternate PR #3. It is not a live Pump clone or canonical client. |
| Create, wallet and trade flows | **Blocked** | The preview saves local drafts; it does not upload metadata, connect wallets, sign or trade. A Beam IDL, client and verified chain data are required. |
| Creator wallet, charity, callouts and livestream screens | **In review as local plans** | PR #6 has draft fields only; no provider or action exists. Preserve observable behavior with Beam branding while implementing services independently. |
| Client and frontend data | **In progress** | [PR #7](https://github.com/shawkkkkk/beam/pull/7) merged golden native-SOL ABI fixtures. There is still no deployed Beam ID, complete SDK, indexer or live market feed. |

### Backend and integrations

| Goal | Status | What has been shown / what remains |
| --- | --- | --- |
| Read-only market/indexer API | **Not started** | Needs chain event replay, finality/freshness metadata and honest error states. Preview charts are fixtures. |
| Creator fee, wallet and sharing service | **Blocked** | On-chain source retains creator vault/collection paths, but Beam still depends on inherited Pump fee configuration. No Beam-controlled payout integration is verified. |
| Donate.gg charity donations | **Blocked** | The archive contains a donation relay and Pump Fees instructions; Beam has no authorized charity registry, configured adapter or settlement service. No real donations occur. |
| Callouts and livestreaming | **Not started** | No backend or provider integration is in the on-chain archive. Identity, permissions, moderation and media services must be defined and tested. |

See [implementation status](docs/STATUS.md) and [contributor tasks](docs/TASKS.md)
for evidence and bounded ways to help with the website, frontend and backend.

## Provisional Beam curve fee

| Allocation | Per trade | Share of fee |
| --- | ---: | ---: |
| Community allocation (creator slice, holder airdrop, flagship burn) | 0.42% | 84% |
| Treasury | 0.05% | 10% |
| Founder | 0.03% | 6% |
| Total | **0.50%** | **100%** |

The decided curve fee is 50 nominal basis points all-in, with the split shown above. Creator payouts come out of the 0.42% community share; there is no additional creator charge. How many of those 42 basis points go to creators is unset. The 0.03% founder share is the current proposal and is tunable. The design goal is a minimal fee, with a very small founder cut, kept below pump.fun's published bonding-curve total. The allocation engine rounds this component once and gives split dust to the community. After the creator take, the remainder funds `$BEAM` holder airdrops and a `$BEAM` buyback-and-burn. `$BEAM` will launch on Beam and has no mint address yet. A 50/50 remainder split is only a provisional test setting. Holder eligibility and buyback execution are still open. Collection is not wired. A post-graduation Beam fee stacks with Raydium's own pool fee. See [economics](docs/ECONOMICS.md), [community allocation](docs/COMMUNITY_ALLOCATION.md), and the [feature matrix](docs/FEATURES.md).

## First contract slice

- New undeployed development identity.
- Mayhem creation/enablement rejected; inert serialized fields and account slots retained.
- Original PumpSwap migration, signed router and AMM-dependent CTO handlers blocked.
- New administrator-pinned Raydium configuration and native-SOL migration candidate: canonical account checks, caller-funded setup, WSOL funding, LP receipt checks and an LP burn in the same instruction. Runtime rollback remains unverified.
- Differential tests compile the pristine upstream crate beside Beam and compare selected retained math, layouts and instruction data.

The current gated migration code targets Raydium CPMM, an earlier candidate superseded by the selected Raydium AMM v4 direction. It is not an AMM v4 adapter. It also burns LP and cannot harvest fees from that burned position. An AMM v4 integration, principal-lock/fee-rights design, and validator testing are required before any migration is enabled.

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

Default builds retain earlier Raydium CPMM devnet IDs for the superseded adapter. `--no-default-features` selects its mainnet IDs. Both configurations and `experimental-raydium` now reject activation of that adapter while AMM v4 work is pending. Neither option deploys anything.

Beam's declared ID is an undeployed placeholder. See [implementation status](docs/STATUS.md). The original MIT and third-party notices remain in the immutable archive.

## Contributing

Start with [contributor guidance](CONTRIBUTING.md), [agent instructions](AGENTS.md),
the [feature matrix](docs/FEATURES.md), and the [task ledger](docs/TASKS.md),
which includes bounded website/backend lanes. [PR #1](https://github.com/shawkkkkk/beam/pull/1)
introduced the pinned contract baseline; [PR #2](https://github.com/shawkkkkk/beam/pull/2)
added migration-accounting guards to its development branch. Check current PR heads before
starting a task. These documents prepare scoped work for contributors; no
Beam is [registered on Slop](https://slop.cash/projects/beam), but the project
was paused with its security scan queued at the last maintainer check. This
repository does not claim clearance, an installed project skill, or funded
bounties. Beam has no repository-root license yet; the archived sources retain
their own licenses.
