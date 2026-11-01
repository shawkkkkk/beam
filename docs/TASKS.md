# Beam task ledger

This ledger records scope and evidence as of 1 November 2026. Check the live
branches and pull requests before starting work; status may change. These are
contribution scopes, not paid bounties or a claim that Beam is listed anywhere.

## Current work and ownership

| Work | State and evidence | Boundary |
| --- | --- | --- |
| [Contract fork, PR #1](https://github.com/shawkkkkk/beam/pull/1), merged to `main` at `200257a` | Development Anchor contract in `programs/beam`; pinned Pump/Raydium reference source; Mayhem and legacy PumpSwap paths blocked; gated CPMM migration candidate; host checks described in [validation](VALIDATION.md). | Canonical development baseline. CPMM candidate is superseded by the user's AMM v4 choice. Fee collection, LP fee rights, validator testing, IDL, and UI remain open. |
| [Migration accounting, PR #2](https://github.com/shawkkkkk/beam/pull/2), head `f17f7e2` | Merged into PR #1's development branch as `65c96b5`: includes post-completion quote, preserves fee/rent liabilities and donor surplus, rejects balance aliases; exact-head CI and local default/experimental suites passed 10 unit and 4 compatibility tests each. | Runtime CPI, fee-sweep, replay, donation, and rollback tests remain before activation; migration stays gated. |
| [Alternate prototype and web preview, PR #3](https://github.com/shawkkkkk/beam/pull/3), `codex/beam-core-foundation` at `eb0aef7` | Separate draft PR against `main` with a root Rust model, a second Anchor program under `programs/beam`, local SBF flow tests, a sample-data website, and a proposed root license. | Its `programs/beam` directly conflicts with the canonical program and still has the superseded 40/7/3 economics. Review web work selectively; do not merge the alternate program or license wholesale. |
| [Contributor guidance, PR #4](https://github.com/shawkkkkk/beam/pull/4), merged as `3630889` within PR #1 | Contributor and agent guidance, reviewed before the development foundation landed. | Documentation only; does not activate or deploy contract behavior. |

PR ownership and touched paths at this snapshot:

- PR #1 — `@shawkkkkk`, `feat/pinned-contract-fork`: `programs/beam/`,
  `upstream/`, `scripts/check_fork.py`, `.github/workflows/contracts.yml`,
  and contract status/economics documents.
- PR #2 — `@shawkkkkk`, `codex/beam-migration-invariant`:
  `programs/beam/src/beam_raydium.rs`; merged into PR #1's branch.
- PR #3 — `@shawkkkkk`, `codex/beam-core-foundation`: root `src/lib.rs`,
  competing `programs/beam/`, root Cargo/CI/license files, `web/`, and its own
  architecture documents. Its Anchor prototype is not the merged canonical
  Beam contract; resolve ownership before integrating any program file.
- PR #4 — `@shawkkkkk`, `codex/beam-agent-guide`: `AGENTS.md`,
  `CONTRIBUTING.md`, `README.md`, and `docs/TASKS.md`, `docs/STATUS.md`,
  `docs/ECONOMICS.md`, `docs/VALIDATION.md`.

## Bounded follow-up scopes

1. **Beam fee custody and configuration.** Replace or rehost inherited Pump
   fee-program authority assumptions with Beam-controlled configuration and
   vaults. Prove total-fee conservation and per-recipient authorization in
   host and validator tests. Enforce the user's 42/5/3-bps
   community/treasury/founder split while
   preserving inherited creator fees and creator-selected payout wallets.
   Treat `beam_economics.rs` as proposed arithmetic, not a live 50-bps fee.
   Keep creator, Raydium pool, and inherited migration fees separate in quotes
   and receipts.
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
5. **Community allocation policy.** The user's direction is to split the
   community share between holder rewards and flagship buybacks. A 50/50
   division within that share is a provisional test setting. Holder
   eligibility, flagship mint, recipients, and execution policy remain to be
   specified. Do not add automatic market orders or claim perpetual revenue.
6. **Feature parity outside the contract.** Use the [feature matrix](FEATURES.md)
   to distinguish on-chain interfaces from missing product services. Build
   authorized creator-wallet, charity/donation, callout, and livestream
   experiences without copying unavailable proprietary backends or presenting
   sample data as live. Keep external disbursements and streams disabled until
   the service, permissions, and tests exist.

## Website and backend contribution lanes

All lanes below are **open development scopes**, not claims of a working Beam
service. Mock wallets, APIs and providers locally; no real financial or media
actions are part of acceptance. Contribute observable behavior and interface
tests, not recovered proprietary web/backend bytes.

| Lane | Interface and dependency | Local acceptance evidence |
| --- | --- | --- |
| Website and UI | Typed product state from the Beam client and read-only API; separate preview fixtures from verified chain data. PR #3's `web/` is a sample preview only. | Responsive create/discovery/detail flows; keyboard/modal/storage tests; visible freshness and unavailable states; no preview RPC/signature attempt. |
| Wallet SDK and IDL | Regenerated `programs/beam` IDL, current account/event layouts, configured Beam ID, explicit capability gates. | Golden create/buy/sell and creator-fee builders against Anchor; exact integer amounts; reject missing/stale accounts; construction separate from signing/sending. |
| Read-only indexer and API | Chain events/accounts and a versioned schema with provenance, slot/finality and cursor. No trading authority. | Replay/rollback, duplicate-event and malformed-account tests; bounded queries; fixtures unmistakably labeled; no fabricated market metrics. |
| Creator wallet and fee sharing | Curve creator, fee vault and sharing-config interfaces; retained Pump fee program is not yet rehosted for Beam. | Selection at creation, recipient/authority validation, claim preview and failure cases; local payout tests without real transfers; no implied arbitrary later wallet change. |
| Charity registry and donation adapter | Verified charity identifiers, authorized Donate.gg relay/settlement and Beam fee configuration; archived relay source alone is insufficient. | Registry provenance, consent and attribution tests; mocked donation/settlement/retry receipts; feature disabled without verified provider and recipient. |
| Callouts | Product contract for identity, moderation, permission and display timing; no on-chain callout ABI is evidenced. | Authenticated mock request, abuse/error handling and accessible UI tests; no paid or live callout action. |
| Livestreaming and moderation | Media-provider contract, creator identity, permissions, moderation and retention policy; no on-chain stream ABI is evidenced. | Local mock player/broadcast states, permission denial, moderation and accessibility tests; no live stream or provider account creation. |

Before advertising these tasks on Slop, a maintainer should confirm the actual
listing, repository-root license/contribution terms, task scope and any reward
funding. This repo currently has no root `LICENSE`; archived MIT/third-party
notices do not decide Beam's new-code license. Draft PR #3 proposes a root
license but it is not merged or adopted. Do not fabricate Slop registration
or generated skill/provenance files, and do not install a GitHub app on behalf
of contributors.

For external contributor listings, copy one bounded scope with its acceptance
tests and current base commit. A maintainer handles any listing, reward terms,
and registration separately. Do not open duplicate public issues to imply
work is available or funded.
