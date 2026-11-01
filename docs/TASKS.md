# Beam task ledger

This ledger records scope and evidence as of 1 November 2026. Check the live
branches and pull requests before starting work; status may change. These are
contribution scopes, not paid bounties. Beam's Slop listing was paused with a
security scan queued at the last maintainer check; do not solicit submissions
or claim clearance or reward funding from this ledger.

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
| [Creator-funding clarity, PR #11](https://github.com/shawkkkkk/beam/pull/11), merged at `9a6ede9` | Records that the 42/5/3-bps allocation does not decide whether creator payouts use its community share or add a separate charge. | No live fee collection or all-in fee quote. |
| [Nested quote guard, PR #12](https://github.com/shawkkkkk/beam/pull/12), merged at `4e32d30` | Graduated `create_v2` quote fails closed before reading PumpSwap pool accounts. | No AMM v4 quote, venue migration or fee routing; other PumpSwap assumptions still need an audit. |
| [Legacy creator SDK, PR #13](https://github.com/shawkkkkk/beam/pull/13), open draft | Local collection builder, creator event decoder and rent/balance preview under `sdk/beam-creator*`; review is pending. | No authenticated state, payout-wallet change, RPC, signing or live claim. Avoid duplicate creator SDK work. |
| [Charity backend, PR #14](https://github.com/shawkkkkk/beam/pull/14), open draft | Disabled local registry and donation mock under `backend/charity/`. | No authorized provider, settlement or real donation. |
| [Callouts backend, PR #15](https://github.com/shawkkkkk/beam/pull/15), open draft | Local moderation service with mock adapters under `backend/callouts/`. | No live callout provider or payment. |
| [Fixture indexer, PR #16](https://github.com/shawkkkkk/beam/pull/16), open draft | Synthetic block replay and bounded queries under `backend/indexer/`. | No live chain feed; reconcile its schema with PR #17 before integration. |
| [Read-only web adapter, PR #17](https://github.com/shawkkkkk/beam/pull/17), open draft | Unverified data boundary under `web/src/indexerAdapter.ts` and local tests. | Separate from PR #16's backend; agree on schema/freshness semantics before integration. |
| [CreateV2 SDK, PR #18](https://github.com/shawkkkkk/beam/pull/18), open draft | Local CreateV2 encoder and account builder in `sdk/beam-native.mjs`. | No RPC, signing or live CreateV2; PR #20 owns raw Create/Trade event decoding. |
| [Progress docs, PR #19](https://github.com/shawkkkkk/beam/pull/19), open draft | README, CONTRIBUTING, STATUS and TASKS contributor progress sync. | Documentation only; reflects current main `4e32d30` plus separate draft ownership. |
| [Raw event SDK, PR #20](https://github.com/shawkkkkk/beam/pull/20), open draft | Strict CreateEvent and TradeEvent decoders under `sdk/beam-events.mjs`. | No live indexer or client state; coordinate event schema with PR #16. |

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
- Open PR #19 — `@shawkkkkk`, `codex/beam-progress-docs`:
  README, CONTRIBUTING and contributor progress documentation only.
- Open PR #20 — `@shawkkkkk`, `codex/beam-sdk-events-20261101`:
  `sdk/beam-events.mjs`, raw Create/Trade fixtures, tests and SDK CI.

## Nonoverlapping next slices

| Owner and path | Dependency | Evidence before integration |
| --- | --- | --- |
| PR #13 owner: `sdk/beam-creator*` | Confirm final contract account/event ABI and selected creator-payout semantics; do not duplicate the active draft. | Node fixtures and exact Anchor producer receipt; reject stale or missing account observations. |
| PR #14 / #15 owners: `backend/charity/` / `backend/callouts/` | External authorization and provider policy remain separate; no live integration from local mocks. | Mock registry/reconciliation and moderation/permission tests, including failures. |
| PR #16 / #17 owners: `backend/indexer/` / `web/src/indexerAdapter.ts` | Agree on a versioned response schema, slot/finality and unavailable states before connecting them. | Replay/rollback and bounded backend queries; web stale/malformed data tests. |
| PR #18 / #20 owners: CreateV2 builder / raw Create/Trade decoder in separate `sdk/` files | Keep fixture ownership distinct; PR #20 owns CreateEvent decoding. | Exact Anchor fixture bytes, rejection paths and Node test receipts. |
| Unassigned: AMM v4 execution adapter in a new `programs/beam/src/` module | Source-only ABI on main; decide SPL-only graduation, LP custody/fee rights and setup fee bounds first. Keep the retired CPMM path disabled. | Host ABI tests plus isolated local-program CPI, rollback, account substitution, donation and replay cases before activation. |
| Unassigned: Beam fee custody and routing in dedicated contract modules | Decide creator payout funding and vault authority before editing shared trade handlers. Build on PR #12's merged `create_v2` guard without reintroducing PumpSwap reads. | Conserved total fee, creator/community allocation, rounding and unauthorized-recipient tests; local runtime evidence before live use. |
| Unassigned: wallet integration and remaining `web/` flows | Build on merged sample preview, finalized Beam IDL/SDK and a verified API. PR #17 already owns the read-only adapter; PR #13 owns creator SDK. | Local mock flows, stale-data/error states, accessibility and transaction-construction tests; no preview fixture presented as chain data. |

Each future owner should record its exact base commit and touched paths in its
PR. Do not combine these slices or treat a host-only result as a Raydium CPI
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
   funding from the community share versus an additional charge is undecided;
   do not hard-code either interpretation or present 50 bps as all-in.
   Treat `beam_economics.rs` as proposed arithmetic, not a live 50-bps fee.
   Disclose each applicable creator, Raydium pool, and inherited migration fee
   in quotes and receipts.
2. **Raydium AMM v4 graduation, principal and fee rights.** Replace the gated
   CPMM candidate with a verified AMM v4 adapter. Specify who can withdraw
   principal and collect ongoing fees, then test those permissions. Current
   CPMM LP burning supplies no recurring LP-fee income. Keep migration disabled
   until the design and tests resolve this.
3. **Raydium AMM v4 migration validation.** Exercise actual CPIs, pool identity,
   config/mint/vault/reserve deltas, fee liabilities, donated balances,
   precreated accounts, replay rejection, and rollback on a local validator.
   Record the pinned AMM v4 program/config used. Use its current 19-account
   `Initialize2` path rather than inventing an OpenBook dependency. Reject
   Token-2022 migration while preserving those curve launches until a reviewed
   compatibility plan exists. Host arithmetic tests do not satisfy this task.
4. **Client and account compatibility.** Regenerate a Beam IDL and build
   create/buy/sell and gated-migration fixtures against the real Anchor
   accounts/events. Test retained instruction encodings and each intentional
   difference. Retained dependency IDLs are references, not a usable Beam SDK.
5. **Community allocation policy.** The user's direction is to fund holder
   rewards and flagship buybacks from the available community allocation.
   If creator payouts use that allocation, divide only its remainder. A 50/50
   division of the available amount is a provisional test setting. Holder
   eligibility, flagship mint, recipients, and execution policy remain to be
   specified. Do not add automatic market orders or claim perpetual revenue.
6. **Feature parity outside the contract.** Use the [feature matrix](FEATURES.md)
   to distinguish on-chain interfaces from missing product services. Build
   authorized creator-wallet, charity/donation, callout, and livestream
   experiences without copying unavailable proprietary backends or presenting
   sample data as live. Keep external disbursements and streams disabled until
   the service, permissions, and tests exist.
7. **Complete PumpSwap removal without losing features.** Audit active Beam
   handlers, quote-chain paths, creator-fee/sharing/CTO instructions,
   donation creation/collection, SDK and UI for PumpSwap program IDs, CPIs,
   PDAs and ownership assumptions. Replace each supported behavior with an
   AMM v4-compatible path where the venue permits it; make unresolved cases
   explicitly unavailable. The immutable upstream archive remains only a
   provenance reference. No PumpSwap runtime fallback or claim that blocked
   entrypoints are finished feature parity.

## Website and backend contribution lanes

The lanes below describe **development scope and ownership**, not working Beam
services. PRs #14–#18 and #20 already own the noted draft slices; check their
heads before starting adjacent work. Mock wallets, APIs and providers locally;
no real financial or media actions are part of acceptance. Contribute observable
behavior and interface tests, not recovered proprietary web/backend bytes.

| Lane | Interface and dependency | Local acceptance evidence |
| --- | --- | --- |
| Website and UI | PR #17 owns a read-only web adapter; PR #6's `web/` remains a sample preview. Additional UI requires typed state from a verified chain data source. | Responsive create/discovery/detail flows; keyboard/modal/storage tests; visible freshness and unavailable states; no preview RPC/signature attempt. |
| Wallet SDK and IDL | PR #13 creator, PR #18 CreateV2 and PR #20 raw events are draft owners; regenerated Beam IDL, configured ID and complete builders remain open. | Golden fixtures against Anchor; exact integer amounts; reject missing/stale accounts; construction separate from signing/sending. |
| Read-only indexer and API | PR #16 owns local fixture backend; PR #17 owns web adapter. Agree on versioned schema, slot/finality and cursor. No trading authority. | Replay/rollback, duplicate-event and malformed-account tests; bounded queries; fixtures labeled; no fabricated market metrics. |
| Creator wallet and fee sharing | Curve creator, fee vault and sharing-config interfaces; retained Pump fee program is not yet rehosted for Beam. | Selection at creation, recipient/authority validation, claim preview and failure cases; local payout tests without real transfers; no implied arbitrary later wallet change. |
| Charity registry and donation adapter | PR #14 owns local `backend/charity/` mocks; authorized Donate.gg relay/settlement and Beam fee configuration are still missing. | Registry provenance, consent and attribution tests; mocked donation/settlement/retry receipts; feature disabled without verified provider and recipient. |
| Callouts | PR #15 owns local `backend/callouts/` service; no on-chain callout ABI or live provider is evidenced. | Authenticated mock request, abuse/error handling and accessible UI tests; no paid or live callout action. |
| Livestreaming and moderation | Media-provider contract, creator identity, permissions, moderation and retention policy; no on-chain stream ABI is evidenced. | Local mock player/broadcast states, permission denial, moderation and accessibility tests; no live stream or provider account creation. |

The [Slop listing](https://slop.cash/projects/beam) was paused with its
security scan queued at the last maintainer check. A maintainer must verify
scan clearance, repository-root license and contribution terms, task scope,
and any reward funding
before inviting external submissions. This repo currently has no root
`LICENSE`; archived MIT/third-party notices do not decide Beam's new-code
license. Closed PR #3 proposed a root license but it was not merged or adopted.
Do not forge generated Slop skill/provenance files or change app permissions
on behalf of contributors.

For external contributor listings, copy one bounded scope with its acceptance
tests and current base commit. A maintainer handles any listing, reward terms,
and registration separately. Do not open duplicate public issues to imply
work is available or funded.
