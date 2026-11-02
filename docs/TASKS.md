# Beam task ledger

This ledger records scope and evidence checked through 2 November 2026. Check the live
branches and pull requests before starting work; status may change. These are
contribution scopes, not paid bounties. Beam's Slop listing was paused with a
security scan queued on 1 November; its state could not be verified on
2 November. Do not claim clearance, reward funding or authorization to resume
feature work from this ledger. See [contribution status](STATUS.md#contribution-and-security-status).
Unset owner values live only in [open owner decisions](OPEN_DECISIONS.md).

## Current work and ownership

| Work | State and evidence | Boundary |
| --- | --- | --- |
| [Contract fork, PR #1](https://github.com/shawkkkkk/beam/pull/1), merged to `main` at `200257a` | Development Anchor contract in `programs/beam`; pinned Pump/Raydium reference source; Mayhem and legacy PumpSwap paths blocked; gated CPMM migration candidate; host checks described in [validation](VALIDATION.md). | Canonical development baseline. CPMM candidate is superseded by the user's AMM v4 choice. Fee collection, LP fee rights, validator testing, IDL, and UI remain open. |
| [Migration accounting, PR #2](https://github.com/shawkkkkk/beam/pull/2), head `f17f7e2` | Merged into PR #1's development branch as `65c96b5`: includes post-completion quote, preserves fee/rent liabilities and donor surplus, rejects balance aliases; exact-head CI and local default/experimental suites passed 10 unit and 4 compatibility tests each. | Runtime CPI, fee-sweep, replay, donation, and rollback tests remain before activation; migration stays gated. |
| [Alternate prototype, PR #3](https://github.com/shawkkkkk/beam/pull/3), closed unmerged at `eb0aef7` | Separate root Rust model, competing Anchor program under `programs/beam`, local SBF tests, sample website, and proposed root license. | Historical draft; its program and 40/7/3 fee policy are not canonical. The reviewed web subtree was carried into PR #6. |
| [Contributor guidance, PR #4](https://github.com/shawkkkkk/beam/pull/4), merged as `3630889` within PR #1 | Contributor and agent guidance, reviewed before the development foundation landed. | Documentation only; does not activate or deploy contract behavior. |
| [Fee/venue correction, PR #5](https://github.com/shawkkkkk/beam/pull/5), merged at `a265c3a` | 42/5/3 test arithmetic, superseded CPMM path made fail closed, feature matrix and contributor scopes. | No AMM v4 CPI or enforced Beam fee collection. |
| [Website preview, PR #6](https://github.com/shawkkkkk/beam/pull/6), merged at `ecd1caf` | Beam-branded sample-data web preview; [PR #8](https://github.com/shawkkkkk/beam/pull/8) merged state, copy and local interaction fixes into its branch. | No real wallet, trading, donations, streams, indexer or backend. |
| [Native-SOL ABI fixtures, PR #7](https://github.com/shawkkkkk/beam/pull/7), merged at `b8120a8` | Local-only legacy create/buy/sell encoder and current account/event fixtures. | No executable transaction builder or creator-fee account resolution. |
| [AMM v4 ABI, PR #9](https://github.com/shawkkkkk/beam/pull/9), merged at `c8dc348` | Source-only `Initialize2` data/account/PDA guards for the official pinned AMM v4 source. | No migration CPI, LP disposition or Token-2022 graduation. |
| [Creator-funding clarity, PR #11](https://github.com/shawkkkkk/beam/pull/11), merged at `9a6ede9` | Recorded the then-open question of whether creator payouts use the community share. [Open owner decisions](OPEN_DECISIONS.md) is now the decision record: creators are paid from the 42 bps community share, and the basis-point count is unset. | No live fee collection. The curve fee is 0.50% all-in. |
| [Nested quote guard, PR #12](https://github.com/shawkkkkk/beam/pull/12), merged at `4e32d30` | Graduated `create_v2` quote fails closed before reading PumpSwap pool accounts. | No AMM v4 quote, venue migration or fee routing; other PumpSwap assumptions still need an audit. |
| [Legacy creator SDK, PR #13](https://github.com/shawkkkkk/beam/pull/13), open draft | Local collection builder, creator event decoder and rent/balance preview under `sdk/beam-creator*`; review is pending. | No authenticated state, payout-wallet change, RPC, signing or live claim. Avoid duplicate creator SDK work. |
| [Charity backend, PR #14](https://github.com/shawkkkkk/beam/pull/14), open draft | Disabled local registry and donation mock under `backend/charity/`. | No authorized provider, settlement or real donation. |
| [Callouts backend, PR #15](https://github.com/shawkkkkk/beam/pull/15), open draft | Local moderation service with mock adapters under `backend/callouts/`. | No live callout provider or payment. |
| [Fixture indexer, PR #16](https://github.com/shawkkkkk/beam/pull/16), open draft | Synthetic block replay and bounded queries under `backend/indexer/`. | No live chain feed; reconcile its schema with PR #17 before integration. |
| [Read-only web adapter, PR #17](https://github.com/shawkkkkk/beam/pull/17), open draft | Unverified data boundary under `web/src/indexerAdapter.ts` and local tests. | Separate from PR #16's backend; agree on schema/freshness semantics before integration. |
| [CreateV2 SDK, PR #18](https://github.com/shawkkkkk/beam/pull/18), open draft | Local CreateV2 encoder and account builder in `sdk/beam-native.mjs`. | No RPC, signing or live CreateV2; PR #20 owns raw Create/Trade event decoding. |
| [Progress docs, PR #19](https://github.com/shawkkkkk/beam/pull/19), merged as `7dab121` | README, CONTRIBUTING, STATUS and TASKS contributor progress sync, integrated with PR #40's upstream attribution. | Documentation only; it does not activate draft features. |
| [Raw event SDK, PR #20](https://github.com/shawkkkkk/beam/pull/20), open draft | Strict CreateEvent and TradeEvent decoders under `sdk/beam-events.mjs`. | No live indexer or client state; coordinate event schema with PR #16. |
| [Livestream backend, PR #21](https://github.com/shawkkkkk/beam/pull/21), open draft | Local authorization, moderation and retention lifecycle under `backend/livestream/` with mock provider. | No media provider, playable stream, live account, durable moderation or website player. |
| [Maintainer guide, PR #35](https://github.com/shawkkkkk/beam/pull/35), merged at `654b3cb` | Contributor and maintainer review gates. | Documentation only. It is merged; do not review it as an open draft. |
| [Upstream attribution, PR #40](https://github.com/shawkkkkk/beam/pull/40), merged at `5757c5d` | [Upstream attribution](UPSTREAM.md). pump.fun ($PUMP) and [lalalune/pumpfun-source](https://github.com/lalalune/pumpfun-source/commit/5703df02131ab944408e0cf679b4bef9c22e1a3d) at `5703df02131ab944408e0cf679b4bef9c22e1a3d`, with its contributors, stay credited. | Documentation only. Retained Pump bytes stay unchanged. |
| [AMM v4 local-validator CPI, PR #38](https://github.com/shawkkkkk/beam/pull/38), merged at `ae3b3b6` | `tests/amm-v4-validator` runs the pinned 19-account `Initialize2` processor in `solana-program-test`. | No Beam migration instruction and no enabled graduation. LP stays locked permanently. Token-2022 migration stays rejected. |
| [PumpSwap removal, PR #36](https://github.com/shawkkkkk/beam/pull/36), open draft at `5c412dbc` | Program and docs draft on `cursor/pumpswap-removal-audit-ab4f`. | Not merged. Do not start a second removal audit. |
| [Community allocation model, PR #37](https://github.com/shawkkkkk/beam/pull/37), open draft at `2fb5787c` | Disabled program model and `docs/COMMUNITY_ALLOCATION.md` on `cursor/community-allocation-policy-ed30`. | Not merged and not closed by the owner-decisions document. That document does not replace the program module. Unset values belong only in [open owner decisions](OPEN_DECISIONS.md). |
| [IDL and gated-migration fixtures, PR #39](https://github.com/shawkkkkk/beam/pull/39), open draft at `e0245755` | Regenerated IDL and gated-migration fixtures on `cursor/beam-idl-gated-migration-dee1`. | Not merged. Scope 4 remains open. |
| Owner decisions, [open owner decisions](OPEN_DECISIONS.md) | The only list of unset owner values. Curve fee, 42/5/3 split, creator source, AMM v4 pin, CPMM fail-closed, PumpSwap removal, permanent LP lock, Token-2022 curve trading, and `$BEAM` as flagship stay as written. | Documentation only. No parameter is set and no fee number changes. |

PR ownership and touched paths at this snapshot:

- PR #1 — `@shawkkkkk`, `feat/pinned-contract-fork`: `programs/beam/`,
  `upstream/`, `scripts/check_fork.py`, `.github/workflows/contracts.yml`,
  and contract status/economics documents.
- PR #2 — `@shawkkkkk`, `codex/beam-migration-invariant`:
  `programs/beam/src/beam_raydium.rs`; merged into PR #1's branch.
- PR #3 — `@shawkkkkk`, `codex/beam-core-foundation`: closed unmerged. Its root
  model, competing `programs/beam/`, Cargo/CI/license files and architecture
  notes are not the canonical Beam contract.
- Merged PR #6 / PR #8 — `@shawkkkkk`, `codex/beam-web-preview`: `web/`,
  website CI and preview status notes. The sample-data preview is now on main.
- PR #4 — `@shawkkkkk`, `codex/beam-agent-guide`: `AGENTS.md`,
  `CONTRIBUTING.md`, `README.md`, and `docs/TASKS.md`, `docs/STATUS.md`,
  `docs/ECONOMICS.md`, `docs/VALIDATION.md`.
- Merged PR #12 — `@shawkkkkk`, `codex/beam-no-pumpswap-quote`:
  `programs/beam/src/create_v2.rs` only. Its fail-closed guard does not create
  an AMM v4 quote path.
- Open PR #13 — `@shawkkkkk`, `codex/beam-creator-client-20261101`:
  `sdk/beam-creator.mjs`, fixtures and creator SDK tests only; coordinate
  against that interface before adding creator-wallet UI.
- Open PR #14 — `@shawkkkkk`, `codex/beam-charity-20261101`:
  `backend/charity/` local mock only.
- Open PR #15 — `@shawkkkkk`, `codex/beam-callouts-20261101`:
  `backend/callouts/` local service only.
- Open PR #16 — `@shawkkkkk`, `codex/beam-indexer-20261101`:
  `backend/indexer/` fixture replay and API only.
- Open PR #17 — `@shawkkkkk`, `codex/beam-indexer-readonly`:
  `web/src/indexerAdapter.ts` and its web test. Coordinate schema with PR #16.
- Open PR #18 — `@shawkkkkk`, `codex/beam-sdk-create-v2`:
  `sdk/beam-native.mjs`, CreateV2 fixture and tests; no raw event decoder.
- Merged PR #19 — `@shawkkkkk`, `codex/beam-progress-docs`:
  README, CONTRIBUTING and contributor progress documentation only.
- Open PR #20 — `@shawkkkkk`, `codex/beam-sdk-events-20261101`:
  `sdk/beam-events.mjs`, raw Create/Trade fixtures, tests and SDK CI.
- Open PR #21 — `@shawkkkkk`, `codex/beam-livestream-20261101`:
  `backend/livestream/` local lifecycle and tests only; no playable stream.

## Draft review queue: PRs #22–#34

Snapshot of PRs #22–#34: 2 November 2026. Their head hashes below were
rechecked later the same day and still match the open draft heads. The
snapshot base `4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf` has since moved.
[PR #35](https://github.com/shawkkkkk/beam/pull/35) merged at `654b3cb`,
[PR #40](https://github.com/shawkkkkk/beam/pull/40) merged upstream
attribution, and [PR #38](https://github.com/shawkkkkk/beam/pull/38) merged
at `ae3b3b6` (`main` at this docs change). Do not review #35 as an open
draft. PRs #36, #37, and #39 are later open drafts recorded in the ownership
table above. Where a draft note below still says LP custody or creator
funding is open, current policy is: LP is locked permanently, creators are
paid from the 42 bps community share, and every unset amount is only in
[open owner decisions](OPEN_DECISIONS.md).
The head hashes below are exact Git PR-head refs at this snapshot. All listed
PRs are drafts; a passing local test or hosted check does not imply merge,
activation, complete integration, or production safety. Recheck heads, target
branches, diffs and exact-head CI before review or merge. File lists describe
the change against the *declared PR target*, not the cumulative diff against
`main`. Every change to `upstream/manifest.json` is a source allowlist update,
not an edit to the retained Pump source archive.

| Draft and exact head | Target / changed files | Evidence and remaining boundary |
| --- | --- | --- |
| [#22 AMM v4 config fee guard](https://github.com/shawkkkkk/beam/pull/22) `884f375343dc1731e21856a4f1bb0e4322c86175` | `main`; `programs/beam/src/beam_amm_v4.rs`. | Local locked default/experimental suites: 17 unit + 4 compatibility each; no-default, provenance and diff checks reported passed. Source guard only. Stored Beam policy must supply the cap and an eventual caller must recheck before CPI. |
| [#23 strict ordinary fee caps](https://github.com/shawkkkkk/beam/pull/23) `c2735f13809e9fc63c9aaa5f31b1760d5fa8a447` | `main`; `programs/beam/src/beam_economics.rs`, `docs/ECONOMICS.md`. | Local default/experimental: 15 unit + 4 compatibility each, no-default/provenance, 200,004-case arithmetic sweep reported passed. Provisional helper only; creator funding and live collection unresolved. |
| [#24 disabled fee policy](https://github.com/shawkkkkk/beam/pull/24) `d578111b75b705948cbcc4a34bef2183f2440603` | `main`; `programs/beam/src/beam_fee_policy_v1.rs`, `programs/beam/src/lib.rs`, `upstream/manifest.json`. | Local default/experimental: 19 unit + 4 compatibility each, no-default/provenance reported passed. Initializer stores Disabled/Undecided and zero credited fees; no quote, collect, update, withdrawal or trade wiring. #23 is a prerequisite before fee activation. |
| [#25 synthetic retired-gate harness](https://github.com/shawkkkkk/beam/pull/25) `4706d59ddb479d3b32c8c369fc342712ff89b023` | `main`; `.github/workflows/runtime-harness.yml`, `tests/runtime-harness/{Cargo.toml,Cargo.lock,src/lib.rs,tests/migration_runtime.rs}`. | Local ProgramTest 3/3 in default and experimental modes reported passed. Tests retired CPMM gates with synthetic programs, not real Raydium CPI or AMM v4 migration. |
| [#26 fee policy local-bank test](https://github.com/shawkkkkk/beam/pull/26) `43ca5500d8046c6b7fc4125b300e16b28699a8ce` | **Stacked on #24**, also incorporates #25; `.github/workflows/runtime-harness.yml`, `tests/runtime-harness/{Cargo.toml,Cargo.lock,src/lib.rs,tests/fee_policy_runtime.rs,tests/migration_runtime.rs}` relative to #24. | New synthetic initializer test reported passed in default/experimental modes: authority, atomic failure, duplicate init, rent and uncredited unsolicited SOL. Review integrated #24+#25 diff. No live fee routing; #23 remains necessary before activation. |
| [#27 V2 trade ABI fixtures](https://github.com/shawkkkkk/beam/pull/27) `5832a12516c0b91751925da5c033a4de678dc531` | `main`; `sdk/V2_TRADES.md`, `sdk/beam-v2-trades.{mjs,test.mjs}`, `sdk/fixtures/v2-trades*`, `sdk/tools/generate-v2-trade-fixtures.rs`. | Local Node tests 10/10 reported passed for BuyV2/SellV2 fixture encodings. Draft #20's SDK workflow is needed to discover these tests in hosted CI. No account discovery, RPC, signing or trade execution. |
| [#28 local feature status](https://github.com/shawkkkkk/beam/pull/28) `290f9a6566ff44d8d4ed7699dddb14dce1826d0b` | **Stacked on #17**; `web/package.json`, `web/src/{App.tsx,localFeaturePresentation.ts}`, `web/tests/{browser.mjs,localFeaturePresentation.test.ts}`. | Local website tests (16), build, Chromium interaction, formatting and diff checks reported passed. Labels creator, charity, callout and livestream plans as local/unavailable; no provider, wallet, trade, donation or stream integration. |
| [#29 AMM v4 source-account preflight](https://github.com/shawkkkkk/beam/pull/29) `7677a7431136a65de93828dc3344b9884d91bc94` | **Stacked on #22**; `programs/beam/src/beam_amm_v4.rs`. | Exact-head local locked default/experimental 23 unit + 4 compatibility each, no-default/provenance/diff passed. Exact-head hosted Website/Native/Contract checks completed successfully. No instruction calls the helper; no CPI, transfer, LP custody or Token-2022 graduation. |
| [#30 disabled AMM v4 funding helper](https://github.com/shawkkkkk/beam/pull/30) `3de7f5cfc1555a340508301121eca4e383c16751` | **Targets #29**; `programs/beam/src/beam_amm_v4_funding.rs`, `programs/beam/tests/amm_v4_funding.rs`, `upstream/manifest.json` against current #29 tree. Latest commit carries the same frozen-source guard as #29, though #29's commit is not literally an ancestor. | Exact-head local default/experimental 23 unit + 7 funding + 4 compatibility each, no-default/provenance passed. Exact-head hosted Website/Native/Contract checks completed successfully. Pure accounting only; module is not exported or invoked and does not establish LP custody, creator funding or migration-fee recipient. |
| [#31 AMM v4 read-only state model](https://github.com/shawkkkkk/beam/pull/31) `6ef639e19162361ced3ddd4d814680a56e751aa1` | **Stacked on #28**; `web/package.json`, `web/src/ammV4Presentation.ts`, `web/tests/ammV4Presentation.test.ts`. | Local website 21/21, build, Chromium, formatting/diff reported passed. Pure unverified-state presentation model, not yet used by App; no chain status, pool metric or transaction action. |
| [#32 disabled migration status UI](https://github.com/shawkkkkk/beam/pull/32) `5c47c03303a46293698bc864e2ab0c4f326fac2e` | **Stacked on #31**; `web/src/{App.tsx,styles.css}`, `web/tests/browser.mjs`. | Local website 21 unit, build, Chromium, formatting/diff reported passed. Inert preview wording only; no wallet, RPC, verified liquidity or live migration. |
| [#33 AMM v4 synthetic settlement](https://github.com/shawkkkkk/beam/pull/33) `6a695e7107576058431f87bc2c52a4a09c5a64ee` | **Targets #25** and integrates #22→#29→#30 guard/funding lineage plus staged settlement; `docs/AMM_V4_INTEGRATION.md`, `programs/beam/src/{beam_amm_v4.rs,beam_amm_v4_funding.rs,beam_amm_v4_settlement.rs,lib.rs}`, `programs/beam/tests/amm_v4_funding.rs`, `tests/runtime-harness/tests/{amm_v4_mock.rs,amm_v4_settlement_mock.rs}`, `upstream/manifest.json`. Its frozen-source guard file matches current #29's tree, though histories differ. | On this head, local default/experimental host suites each passed 33 unit + 7 funding + 4 compatibility; synthetic ProgramTest 9/9 per mode and no-default/provenance/diff checks also passed. Exact-head hosted Website/Native/Contract/Local-bank checks completed successfully. Review combined diff before merge. Mock processors are not a real Raydium CPI or Beam migration entrypoint. LP custody, fee rights, exact setup-rent attribution, recipient policy and Token-2022 remain open. |
| [#34 AMM v4 post-CPI account snapshot](https://github.com/shawkkkkk/beam/pull/34) `8eaadc50a174d94ba10fb1e79696c3d65925932e` | **Stacked on #33**; `docs/AMM_V4_INTEGRATION.md`, `programs/beam/src/{beam_amm_v4_snapshot.rs,lib.rs}`, `upstream/manifest.json`. | Exact-head local default/experimental 37 unit + 7 funding + 4 compatibility each, no-default/provenance/diff passed. Exact-head hosted Website/Native/Contract/Local-bank checks completed successfully. Disabled source-pinned account decoder only; no real CPI/entrypoint, authenticated market seed/clock, LP custody or deployed-binary match. |

Review order follows actual branch targets, not PR number alone:

1. Contract guard chain: #22 → #29 (frozen guard) → #30 (equivalent frozen
   guard carried forward), plus #25 → #33 → #34. Review #33 only after its
   guard/funding and synthetic harness dependencies, including its staged
   settlement code as a separate trust boundary. Review #34's source-pinned
   snapshot independently; its exact-head hosted checks passed on 2 November.
2. Fee chain: #23 and #24 are independent drafts on `main`; review both,
   then #25 and stacked #26 against their resulting combined tree. Policy
   remains disabled. Creators are paid from the 42 bps community share. The
   creator basis-point count stays unset in
   [open owner decisions](OPEN_DECISIONS.md).
3. Client/UI chain: #20 is needed for #27 hosted test discovery. For web,
   review #17 → #28 → #31 → #32. Check #6's merged preview baseline and
   avoid treating local-only cards as production service integration.

At this snapshot, the maintainer verified exact-head hosted checks through the
GitHub connector for #22–#33; #34's exact-head hosted checks completed successfully on 2 November 2026. Treat this as
time-bound evidence. Recheck checks on each exact head and combined tree,
especially after a base branch advances. Do not auto-merge drafts on the
strength of this table.

## Nonoverlapping next slices

| Owner and path | Dependency | Evidence before integration |
| --- | --- | --- |
| PR #13 owner: `sdk/beam-creator*` | Confirm final contract account/event ABI and selected creator-payout semantics; do not duplicate the active draft. | Node fixtures and exact Anchor producer receipt; reject stale or missing account observations. |
| PR #14 / #15 owners: `backend/charity/` / `backend/callouts/` | External authorization and provider policy remain separate; no live integration from local mocks. | Mock registry/reconciliation and moderation/permission tests, including failures. |
| PR #21 owner: `backend/livestream/` | Local authorization and moderation lifecycle only; real media integration and player UI are separate work. | Mock permission, start/stop/ban race, cleanup and retention tests; no live provider. |
| PR #16 / #17 owners: `backend/indexer/` / `web/src/indexerAdapter.ts` | Agree on a versioned response schema, slot/finality and unavailable states before connecting them. | Replay/rollback and bounded backend queries; web stale/malformed data tests. |
| PR #18 / #20 owners: CreateV2 builder / raw Create/Trade decoder in separate `sdk/` files | Keep fixture ownership distinct; PR #20 owns CreateEvent decoding. | Exact Anchor fixture bytes, rejection paths and Node test receipts. |
| Unassigned: AMM v4 execution adapter in a new `programs/beam/src/` module | Source-only ABI plus merged PR #38 local-validator evidence. LP is locked permanently. CPMM stays fail-closed. Token-2022 migration stays rejected until a safe plan lands. Any post-graduation Beam fee is unset in [open owner decisions](OPEN_DECISIONS.md). | Host ABI tests plus isolated local-program CPI, rollback, account substitution, donation and replay cases before activation. |
| Unassigned: Beam fee custody and routing in dedicated contract modules | Creators are paid from the 42 bps community share. The basis-point count and vault accounts are unset in [open owner decisions](OPEN_DECISIONS.md) and stay disabled or unset, failing closed. Build on PR #12's merged `create_v2` guard without reintroducing PumpSwap reads. | Conserved total fee, creator/community allocation, rounding and unauthorized-recipient tests; local runtime evidence before live use. |
| Unassigned: wallet integration and remaining `web/` flows | Build on merged sample preview, finalized Beam IDL/SDK and a verified API. PR #17 already owns the read-only adapter; PR #13 owns creator SDK. | Local mock flows, stale-data/error states, accessibility and transaction-construction tests; no preview fixture presented as chain data. |

Each future owner should record its exact base commit and touched paths in its
PR and check the open PR queue before claiming a lane. Do not combine these
slices or treat a host-only result as a Raydium CPI
receipt. The optional three-second buy-side quote prototype remains disabled
and test-only. It models an initial 99% of gross buyer debit, whereas 99% of
net base is about 49.75% of gross (0.99 / 1.99). Its launch surcharge has no
recipient or withdrawal path; do not wire it to a trading handler without a
precise specification and isolated adversarial tests.

## Bounded follow-up scopes

1. **Beam fee custody and configuration.** Replace or rehost inherited Pump
   fee-program authority assumptions with Beam-controlled configuration and
   vaults. Prove total-fee conservation and per-recipient authorization in
   host and validator tests. Enforce the user's 42/5/3-bps
   community/treasury/founder split while
   preserving creator payouts and creator-selected payout wallets. Creator
   payouts are taken out of the 42 bps community share. The curve fee is
   0.50% all-in. The creator basis-point count is unset. Any code path that
   needs an unset value in [open owner decisions](OPEN_DECISIONS.md) uses an
   admin-configurable parameter that defaults to disabled or unset and fails
   closed. Treat `beam_economics.rs` as proposed arithmetic, not live
   collection. Disclose each applicable creator, Raydium pool, and inherited
   migration fee in quotes and receipts.
2. **Raydium AMM v4 graduation, principal and fee rights.** Replace the gated
   CPMM candidate with a verified AMM v4 adapter. The venue is Raydium AMM v4
   (`raydium-io/raydium-amm` `91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6`,
   program `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`, 19-account
   `Initialize2`). The CPMM candidate stays fail-closed. LP is locked
   permanently, so a locked position is not a recurring fee claim. Any
   post-graduation Beam fee is unset; see
   [open owner decisions](OPEN_DECISIONS.md). Keep migration disabled until
   the adapter and tests exist. Token-2022 still trades on the curve and
   stays rejected for migration until a safe plan lands.
3. **Raydium AMM v4 migration validation.** Exercise actual CPIs, pool identity,
   config/mint/vault/reserve deltas, fee liabilities, donated balances,
   precreated accounts, replay rejection, and rollback on a local validator.
   Record the pinned AMM v4 program/config used. Use its current 19-account
   `Initialize2` path rather than inventing an OpenBook dependency. Reject
   Token-2022 migration while preserving those curve launches until a reviewed
   compatibility plan exists. Host arithmetic tests do not satisfy this task.
   [Merged PR #38](https://github.com/shawkkkkk/beam/pull/38) added
   `tests/amm-v4-validator`, a local banks run of the pinned `Initialize2`
   processor. That evidence does not add a Beam migration instruction, enable
   graduation, or finish this scope.
4. **Client and account compatibility.** Regenerate a Beam IDL and build
   create/buy/sell and gated-migration fixtures against the real Anchor
   accounts/events. Test retained instruction encodings and each intentional
   difference. Retained dependency IDLs are references, not a usable Beam SDK.
   Open draft [PR #39](https://github.com/shawkkkkk/beam/pull/39) owns one
   fixture attempt. This scope still needs program-facing review and is not
   a docs-only item.
5. **Community allocation policy.** Holder rewards and `$BEAM` flagship
   buybacks are funded from the community allocation after the creator take.
   `$BEAM` is the flagship. The separate 10,000,000 whole `$BEAM` Slop Cash
   contributor allocation stays disabled by default. Every unset owner value
   is only in [open owner decisions](OPEN_DECISIONS.md). Documentation of
   that list is done. Wiring each parameter still needs program code and an
   owner decision. Do not add automatic market orders or claim perpetual
   revenue.
6. **Feature parity outside the contract.** Use the [feature matrix](FEATURES.md)
   to distinguish on-chain interfaces from missing product services. Build
   authorized creator-wallet, charity/donation, callout, and livestream
   experiences without copying unavailable proprietary backends or presenting
   sample data as live. Keep external disbursements and streams disabled until
   the service, permissions, and tests exist.
7. **Complete PumpSwap removal without losing features.** PumpSwap stays
   removed as venue policy, with no runtime fallback. Open draft
   [PR #36](https://github.com/shawkkkkk/beam/pull/36) owns the in-progress
   audit. Audit active Beam
   handlers, quote-chain paths, creator-fee/sharing/CTO instructions,
   donation creation/collection, SDK and UI for PumpSwap program IDs, CPIs,
   PDAs and ownership assumptions. Replace each supported behavior with an
   AMM v4-compatible path where the venue permits it; make unresolved cases
   explicitly unavailable. The immutable upstream archive remains only a
   provenance reference. No PumpSwap runtime fallback or claim that blocked
   entrypoints are finished feature parity.

## Website and backend contribution lanes

The lanes below describe **development scope and ownership**, not working Beam
services. PRs #14–#18 and #20–#21 already own the noted draft slices; check their
heads before starting adjacent work. Mock wallets, APIs and providers locally;
no real financial or media actions are part of acceptance. Contribute observable
behavior and interface tests, not recovered proprietary web/backend bytes.

| Lane | Interface and dependency | Local acceptance evidence |
| --- | --- | --- |
| Website and UI | PR #17 owns a read-only web adapter; PR #6's `web/` remains a sample preview. Additional UI requires typed state from a verified chain data source. | Responsive create/discovery/detail flows; keyboard/modal/storage tests; visible freshness and unavailable states; no preview RPC/signature attempt. |
| Wallet SDK and IDL | PR #13 creator, PR #18 CreateV2 and PR #20 raw events are draft owners. PR #39 is the open regenerated-IDL draft. Configured ID and complete builders remain open. | Golden fixtures against Anchor; exact integer amounts; reject missing/stale accounts; construction separate from signing/sending. |
| Read-only indexer and API | PR #16 owns local fixture backend; PR #17 owns web adapter. Agree on versioned schema, slot/finality and cursor. No trading authority. | Replay/rollback, duplicate-event and malformed-account tests; bounded queries; fixtures labeled; no fabricated market metrics. |
| Creator wallet and fee sharing | Curve creator, fee vault and sharing-config interfaces; retained Pump fee program is not yet rehosted for Beam. | Selection at creation, recipient/authority validation, claim preview and failure cases; local payout tests without real transfers; no implied arbitrary later wallet change. |
| Charity registry and donation adapter | PR #14 owns local `backend/charity/` mocks; authorized Donate.gg relay/settlement and Beam fee configuration are still missing. | Registry provenance, consent and attribution tests; mocked donation/settlement/retry receipts; feature disabled without verified provider and recipient. |
| Callouts | PR #15 owns local `backend/callouts/` service; no on-chain callout ABI or live provider is evidenced. | Authenticated mock request, abuse/error handling and accessible UI tests; no paid or live callout action. |
| Livestreaming and moderation | PR #21 owns the local `backend/livestream/` lifecycle; media provider, durable identity/moderation and player UI remain separate. No on-chain stream ABI is evidenced. | Local permission, lifecycle and retention tests; future mock player/accessibility tests; no live stream or provider account creation. |

The [Slop listing](https://slop.cash/projects/beam) was paused with its
security scan queued at the 1 November 2026 maintainer check; its state was
not verifiable during the 2 November documentation review. A maintainer must
verify current status, repository-root license and contribution terms, task
scope, and any reward funding before inviting Slop submissions. This repo
currently has no root
`LICENSE`; archived MIT/third-party notices do not decide Beam's new-code
license. Closed PR #3 proposed a root license but it was not merged or adopted.
Do not forge generated Slop skill/provenance files or change app permissions
on behalf of contributors.

For external contributor listings, copy one bounded scope with its acceptance
tests and current base commit. A maintainer handles any listing, reward terms,
and registration separately. Do not open duplicate public issues to imply
work is available or funded. [CONTRIBUTING.md](../CONTRIBUTING.md#maintainer-review-and-merge)
defines the GitHub review and merge sequence; Slop automatic closure is not a
GitHub approval or protection bypass.
