# PumpSwap removal audit

Audit base: `4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf` on `main`
(merge of PR #12, which already rejects a graduated nested quote before
reading a pool account). This change is not stacked on draft PRs #13–#35.
Those drafts own fee policy, the AMM v4 adapter and its local-bank checks,
SDK fixtures, the preview, and the product-service mocks. This pass does not
edit those lanes.

The upstream snapshots under `upstream/pumpfun-source` and
`upstream/raydium-cp-swap` are unchanged. Reference IDLs under
`programs/beam/idls/` stay byte-identical with the retained package.

PumpSwap here means program `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`
and the pool, creator-vault, and event PDAs derived from it. Raydium AMM v4
is a different program and a different account layout. This pass does not
add an AMM v4 swap, migration CPI, or pool-reserve reader. Those belong to
the graduation and local-validator tasks already in review.

## Slice in this change

Graduated quote accounts can no longer be accepted as PumpSwap pools.
`validate_pool`, `pool_record`, and the graduated branch of create-v2 quote
walking all return `PumpSwapDisabled` before any owner check. The historical
211-byte pool decoder remains, and a host test shows that a well-formed
decoder result is still rejected.

`migrate`, `migrate_v2`, `multi_hop_curve_swap`, and `admin_cto` already
returned that error at the instruction entry. Their bodies, and the public
boost helpers, now return the same error before transfers or PumpSwap CPIs.
Instruction discriminators still match the pinned reference. Curve-phase
creator collection and sharing distribution do not call PumpSwap and are
unchanged.

SDK and preview sources on this branch do not contain the PumpSwap program
id. A Node test locks that.

## Touchpoints

| Surface | PumpSwap assumption | Disposition in this change | Remaining |
| --- | --- | --- | --- |
| `declare_program!(pump_amm)` in `programs/beam/src/lib.rs` | Generated client and id from `idls/pump_amm.json`. | Left in place so retained call syntax still typechecks. No ungated instruction calls it. | Deleting the generated client is a separate retained-source edit. It is not required once every body returns first. |
| `migrate` accounts and `migrate::execute` / `init_pump_amm_boost` | `Program<PumpAmm>`, pool / global-config / LP-mint / event PDAs, `create_pool` and `init_boost` CPIs. | Body and boost helper return `PumpSwapDisabled` before transfers or CPIs. Discriminator preserved. | Anchor still checks those PDAs and the program id before the handler. AMM v4 `Initialize2` is a different 19-account path and is not substituted here. |
| `migrate_v2` accounts and `migrate_v2::execute` / `init_pump_amm_boost_v2` | Same PDAs, plus an event-authority address derived from `pump_amm::ID` inside the body. | Body and boost helper return before that derivation and before `create_pool` / `init_boost`. | Same constraint-layer PDAs. Non-SOL and Token-2022 graduation stay unavailable; AMM v4 does not accept Token-2022 mints. |
| `multi_hop_curve_swap` | Account `pump_amm_global_config` must be the PumpSwap `global_config` PDA and a signer. The body does not CPI to PumpSwap; it estimates a post-complete fill locally. | Instruction entry and `execute` return `PumpSwapDisabled` before that estimate. | The account constraint still names the PumpSwap PDA. An AMM v4 hop router is not built here. |
| `admin_cto` | PumpSwap program, global-config, pool, event, and creator-vault PDAs. CPIs: `collect_coin_creator_fee`, `transfer_creator_fees_to_pump_v2`, `admin_cto_pool`, and pump-fees `admin_cto_sharing_config`. | `execute` returns before those CPIs. | AMM v4 has no coin-creator vault, Pump vault transfer, or CTO-pool instruction. Post-graduation creator changes stay unavailable until a reviewed fee-rights design exists. Constraint-level PDAs remain. |
| `pump_quote` `AMM`, `parse_pool`, `validate_pool`, `pool_record` | Owner must be the PumpSwap program and the data must match the PumpSwap pool discriminator and vault offsets. | `validate_pool` and `pool_record` return `PumpSwapDisabled` for every owner, including the historical id. `parse_pool` still decodes the old layout and has no quote caller. | An AMM v4 reserve reader is a different layout and depends on a real migrated pool. It is not invented here. |
| create-v2 nested quote walk | Graduated (zero-reserve) ancestors used to require PumpSwap pool accounts. PR #12 already returned `PumpSwapDisabled`. Active curves use `Record::Curve`. | The graduated branch now calls the shared `graduated_quote_unavailable` helper. Active-curve pricing is unchanged. In-memory `Record::Pool` math remains and is not fed by an account. | Pricing a graduated ancestor from AMM v4 vaults. |
| `buy`, `sell`, `buy_v2`, `sell_v2`, `buy_v3` / `sell_v3` | No PumpSwap program, CPI, or pool PDA. Completed curves are rejected or marked complete on the curve. | Unchanged. Curve trading stays available. | Post-graduation swaps are not routed to PumpSwap and are not routed to AMM v4 in this change. |
| `collect_creator_fee`, `collect_creator_fee_v2`, `distribute_creator_fees`, `distribute_creator_fees_v2`, holder distribution, `migrate_bonding_curve_creator`, `set_creator` | Curve creator vault and pump-fees sharing-config accounts (`pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ`). No PumpSwap CPI. | Unchanged. A host test checks the collect and native distribute sources do not name PumpSwap. | Pump-fees authority still assumes Pump's deployed program id. Rehosting that is the fee-custody task, not a PumpSwap fallback. Post-graduation creator claims stay unavailable with `admin_cto`. |
| `get_fees_with_quote_mint` in `util/fee.rs` and `fee_cpi.rs` | CPI to the pump-fees program, not PumpSwap. The fees IDL error `UnauthorizedProgram` says only Pump and PumpSwap may call some fees instructions. | No PumpSwap caller was added to satisfy that check. | Fee-program rehost. Do not route Beam through PumpSwap to pass that check. |
| Fees IDL `create_fee_sharing_config`, `update_fee_shares`, `update_fee_shares_v2` | Optional or required `pump_amm_program` address `pAMMBay…`, plus Pump program `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`. | Beam does not invoke these instructions. | Unavailable. Sharing updates that require those accounts are not given a PumpSwap path. |
| Fees IDL `create_donation_fee_pda` | `pool` is present and unconstrained. No PumpSwap address constant on that instruction. | No Beam donation instruction calls it. | Unavailable. AMM v4 has no donation-fee PDA. The local charity mock in draft PR #14 is a separate lane and was not changed. |
| Fees IDL `crank_donation_fee_pda` | Donation relay `RLAYHr9TRFcKB2ubYQhspcnXiaGpaVzNQvHytt47RZu`. No PumpSwap program id. | No Beam caller. | Unavailable until an authorized relay and recipient exist. Not a PumpSwap feature. |
| `idls/pump_amm.json`, `idls/pump_fees.json`, Mayhem IDLs | Embedded PumpSwap address and CPIs, including Mayhem `buy_pump_swap` / `sell_pump_swap` in the archive only. | Retained as reference bytes. Mayhem creation and enablement stay rejected. Beam does not deploy Mayhem. | Keep the IDLs with the retained package. They are not a client. |
| `QuotePoolAccountsRequired`, `QuotePoolNotFound`, `QuoteCurveAwaitingMigration` messages in `lib.rs` | Old strings told callers to supply a pump-amm pool or to retry `migrate_v2`. | Same error codes. Strings now say PumpSwap pools and PumpSwap migration are disabled. `InvalidQuotePool` still describes the historical decoder. | None for these strings. Codes were not renumbered. |
| `programs/beam/src/beam_raydium.rs` | Superseded CPMM candidate. Not PumpSwap. | Untouched. Activation already fails closed. | AMM v4 adapter work in the other drafts. |
| `programs/beam/src/beam_amm_v4.rs` | Source-only AMM v4 `Initialize2` guards. Not a PumpSwap path and not called by a handler. | Untouched. | Wiring, LP rights, and local-validator CPIs stay on the graduation tasks. |
| `sdk/beam-native.mjs` and `sdk/fixtures/native-sol.json` | Legacy create/buy/sell for the Beam placeholder id. No PumpSwap id, PDA, or CPI. | Unchanged. New `sdk/no-pumpswap-surface.test.mjs` fails if `sdk/` or `web/src` embeds the PumpSwap id. | Draft SDK PRs #13, #18, #20, and #27 add other fixtures. They were not edited. They need the same absence check when they land. |
| `web/src` | Sample preview. No PumpSwap id. Copy already says the AMM v4 adapter is not active. | No UI behavior change. The Node test covers `web/src`. | Preview drafts #28, #31, and #32 own further unavailable-state copy. |
| `upstream/pumpfun-source`, `upstream/raydium-cp-swap` | Full historical PumpSwap implementation, donation relay, and fees program. | Untouched provenance. | Not a runtime dependency of the editable program beyond the retained, now fail-closed, call text. |

## Explicitly not replaced by AMM v4

These PumpSwap behaviors do not have an AMM v4 equivalent in the pinned
`Initialize2` ABI, so this change makes them unavailable instead of mapping
them onto the wrong accounts:

- PumpSwap `create_pool` and `init_boost` during migration.
- PumpSwap coin-creator vault collection and `transfer_creator_fees_to_pump_v2`.
- `admin_cto_pool` creator replacement on a PumpSwap pool.
- Quote-chain reserves read from a PumpSwap pool account.
- Fee-sharing and donation instructions whose IDL accounts name PumpSwap or
  the donation relay.
- Token-2022 graduation. AMM v4 does not accept those mints. Curve launch
  math for Token-2022 is unchanged; graduation of those mints stays unavailable.

Curve create/buy/sell, curve-phase creator-vault collection, and curve-phase
sharing distribution stay in the program. They are not PumpSwap features.

## What remains

1. Relax Anchor constraints on `migrate`, `migrate_v2`, `multi_hop_curve_swap`,
   and `admin_cto` so a caller reaches `PumpSwapDisabled` without supplying
   PumpSwap program accounts or PDAs. Keep account order and discriminators.
2. After a reviewed AMM v4 migration exists, add a separate quote reader for
   that pool layout. Do not point `parse_pool` at it.
3. Post-graduation creator fee rights and CTO. AMM v4 LP fees are not the
   PumpSwap creator vault. That design is still open and is outside this slice.
4. Rehost pump-fees so Beam is the authorized caller. Do not satisfy
   `UnauthorizedProgram` by CPIing from PumpSwap.
5. Donation creation and collection. No Beam instruction exists. Do not add
   one that targets PumpSwap or an unauthorized relay.
6. When the in-flight SDK drafts land, extend the surface test to their new
   files. This branch only locks the SDK and preview files that are on `main`.
7. Local-validator evidence that the fail-closed handlers roll back and that
   no CPI is attempted. Host tests in this change do not execute Solana CPIs.

Host tests do not prove SBF behavior, transaction rollback, or that Anchor's
constraint checks are skipped. A transaction that fails a PumpSwap PDA
constraint never reaches the handler error.
