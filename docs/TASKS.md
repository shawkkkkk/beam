# Beam task ledger

This ledger records scope and evidence as of 1 November 2026. Check the live
branches and pull requests before starting work; status may change. These are
contribution scopes, not paid bounties or a claim that Beam is listed anywhere.

## Current work and ownership

| Work | State and evidence | Boundary |
| --- | --- | --- |
| [Contract fork, PR #1](https://github.com/shawkkkkk/beam/pull/1), `feat/pinned-contract-fork` at `65c96b5` | Draft Anchor contract in `programs/beam`; pinned Pump/Raydium reference source; Mayhem and legacy PumpSwap paths blocked; gated CPMM migration candidate; host checks described in [validation](VALIDATION.md). | Canonical contract baseline for review. Fee collection, LP fee rights, validator testing, IDL, and UI remain open. |
| [Migration accounting, PR #2](https://github.com/shawkkkkk/beam/pull/2), head `f17f7e2` | Merged into PR #1's development branch as `65c96b5`: includes post-completion quote, preserves fee/rent liabilities and donor surplus, rejects balance aliases; exact-head CI and local default/experimental suites passed 10 unit and 4 compatibility tests each. | Runtime CPI, fee-sweep, replay, donation, and rollback tests remain before activation; migration stays gated. |
| [Core model and web preview, PR #3](https://github.com/shawkkkkk/beam/pull/3), `codex/beam-core-foundation` at `5214074` | Separate draft PR against `main` with a root Rust model, sample-data website, and different venue/economic assumptions. | The model is not `programs/beam`; reconcile contracts and fee policy before integration. The sample UI is not an on-chain client. |
| [Contributor guidance, PR #4](https://github.com/shawkkkkk/beam/pull/4), `codex/beam-agent-guide` | Draft contributor and agent guidance stacked on PR #1's updated development head. | Documentation only; does not activate or deploy contract behavior. |

PR ownership and touched paths at this snapshot:

- PR #1 — `@shawkkkkk`, `feat/pinned-contract-fork`: `programs/beam/`,
  `upstream/`, `scripts/check_fork.py`, `.github/workflows/contracts.yml`,
  and contract status/economics documents.
- PR #2 — `@shawkkkkk`, `codex/beam-migration-invariant`:
  `programs/beam/src/beam_raydium.rs`; merged into PR #1's branch.
- PR #3 — `@shawkkkkk`, `codex/beam-core-foundation`: root `src/lib.rs`,
  root Cargo/CI files, `web/`, and its own architecture documents. It is a
  separate draft model and preview, not the Anchor contract.
- PR #4 — `@shawkkkkk`, `codex/beam-agent-guide`: `AGENTS.md`,
  `CONTRIBUTING.md`, `README.md`, and `docs/TASKS.md`, `docs/STATUS.md`,
  `docs/ECONOMICS.md`, `docs/VALIDATION.md`.

## Bounded follow-up scopes

1. **Beam fee custody and configuration.** Replace or rehost inherited Pump
   fee-program authority assumptions with Beam-controlled configuration and
   vaults. Prove total-fee conservation and per-recipient authorization in
   host and validator tests. Treat `beam_economics.rs` as proposed arithmetic,
   not a live 50-bps fee. Keep Raydium pool fees and the inherited migration
   fee separate in quotes and receipts.
2. **Graduation principal and fee rights.** Specify who can withdraw principal
   and who can collect ongoing fees, then test those permissions. Current LP
   burning supplies no recurring LP-fee income. Keep migration disabled until
   the design and tests resolve this.
3. **Raydium CPMM migration validation.** Exercise actual CPIs, pool identity,
   config/mint/vault/reserve deltas, fee liabilities, donated balances,
   precreated accounts, replay rejection, and rollback on a local validator.
   Record the pinned Raydium program/config used. Host arithmetic tests do not
   satisfy this task.
4. **Client and account compatibility.** Regenerate a Beam IDL and build
   create/buy/sell and gated-migration fixtures against the real Anchor
   accounts/events. Test retained instruction encodings and each intentional
   difference. Retained dependency IDLs are references, not a usable Beam SDK.
5. **Community allocation policy.** The user's direction is to split the
   community share between holder rewards and flagship buybacks. A 50/50
   division within that share is a provisional test setting. Holder
   eligibility, flagship mint, recipients, and execution policy remain to be
   specified. Do not add automatic market orders or claim perpetual revenue.

For external contributor listings, copy one bounded scope with its acceptance
tests and current base commit. A maintainer handles any listing, reward terms,
and registration separately. Do not open duplicate public issues to imply
work is available or funded.
