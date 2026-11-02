# Implementation status

## Implemented

- Complete final Pump archive pinned and verified by original Git blob hashes.
- Editable Pump Main fork, with unchanged files retaining their exact bytes.
- Separate Beam modules; hooks retain existing source line positions.
- Mayhem creation/enablement rejected. Inert serialized flags/account slots remain; Beam does not deploy the companion Mayhem program.
- Legacy `migrate`, `migrate_v2`, `multi_hop_curve_swap` and `admin_cto` handlers fail before transfers/CPIs. They are temporary compatibility gaps, not permanent product deletions or PumpSwap fallbacks. Other retained quote-chain, creator-fee/sharing, CTO and donation source still has historical PumpSwap assumptions; finished Beam needs replacements or explicit unsupported states, with no active PumpSwap dependency.
- New `configure_raydium`, `set_raydium_enabled` and `migrate_raydium` candidates targeting Raydium CPMM. The user subsequently chose Raydium AMM v4. This CPMM path is a superseded development candidate; activation and execution now fail closed even with `experimental-raydium`.
- That native-SOL CPMM candidate validates the fixed executable, canonical administrator-pinned AmmConfig, mint ordering, vault/LP/observation PDAs, completion and real reserves/rent. It funds WSOL, invokes initialize, checks/burns received LP, records a one-time receipt and clears curve reserve fields in the same instruction. Runtime rollback remains unverified, and the code is not an AMM v4 adapter.
- Caller setup funding is separate from curve liquidity. The budget covers Raydium CPIs; outer ATA/receipt rent and transaction fees are additional. Refunds cannot exceed the caller's unspent contribution or include prior authority donations.
- Standard SPL Token and a restricted Token-2022 metadata extension subset were supported by the superseded CPMM candidate. AMM v4 does not accept Token-2022 mints; those curve launches need a separately reviewed graduation plan rather than silent removal.
- Inherited Global migration-fee behavior retained. Test-only Beam curve-fee arithmetic now reflects the requested 42/5/3 bps community/treasury/founder split; it does not rewrite inherited creator or migration fees or enforce a live rate.
- Source-only [AMM v4 ABI guards](../programs/beam/src/beam_amm_v4.rs) encode the pinned 19-account `Initialize2` path and reject Token-2022 mint owners. They are not called by a Beam instruction and do not migrate a token.
- Fee-allocation arithmetic and selected host differential tests. The curve policy is 0.50% all-in, split 42/5/3 bps community/treasury/founder, with creators paid from the 42 bps community share. The creator basis-point count is unset. See [open owner decisions](OPEN_DECISIONS.md).
- [PR #38](https://github.com/shawkkkkk/beam/pull/38) merged `tests/amm-v4-validator`, a local banks run of the pinned 19-account `Initialize2` processor. It does not add a Beam migration instruction or enable graduation. LP stays locked permanently. Token-2022 migration stays rejected.
- [PR #7](https://github.com/shawkkkkk/beam/pull/7) merged native-SOL create/buy/sell ABI fixtures and local encoders. They do not resolve live accounts, sign or submit transactions. Draft [PR #13](https://github.com/shawkkkkk/beam/pull/13) owns legacy creator-fee fixtures, [PR #18](https://github.com/shawkkkkk/beam/pull/18) owns CreateV2 construction, and [PR #20](https://github.com/shawkkkkk/beam/pull/20) owns raw Create/Trade event decoding; these are not part of this main-branch snapshot.
- [PR #6](https://github.com/shawkkkkk/beam/pull/6) merged a Beam-branded website preview with sample market data and browser-local drafts. It has no wallet, chain feed, metadata upload, donation, callout or livestream service.

Separate draft [PR #21](https://github.com/shawkkkkk/beam/pull/21) owns a
local mock livestream authorization and moderation lifecycle under
`backend/livestream/`. It is not on main and has no media provider or playable
stream; the [task ledger](TASKS.md) records adjacent draft backend scopes.

## Draft work, not merged or activated

| Area | Current draft chain | Limit |
| --- | --- | --- |
| AMM v4 configuration, accounts and funding | [#22 → #29 → #30](TASKS.md#draft-review-queue-prs-2234) | Source guards and accounting only. No Beam migration instruction calls them. |
| Synthetic AMM v4 settlement and account snapshot | [#25 → #33 → #34](TASKS.md#draft-review-queue-prs-2234) | Local-bank tests use mock processors. The source-pinned snapshot decoder does not authenticate a deployed Raydium binary or settle a real pool. |
| Beam fee policy | [#23, #24 and #26](TASKS.md#draft-review-queue-prs-2234) | Strict arithmetic, disabled policy state and local-bank initializer tests; no executable fee quote, trade collection or creator payout routing. |
| Client and preview | SDK [#20](https://github.com/shawkkkkk/beam/pull/20) and [#27](https://github.com/shawkkkkk/beam/pull/27); web [#17 → #28 → #31 → #32](TASKS.md#draft-review-queue-prs-2234) | Fixture decoders and unverified, read-only preview states. No wallet, RPC-authenticated market feed, provider actions or live trade. |

See the [task ledger](TASKS.md) for dated heads, target branches, checks and
stacked review order. A passing draft test does not make its feature live.

## Outstanding before use

1. Beam-controlled fee configuration and vaults. Pump's inherited fee-program namespaces/authority assumptions must be replaced or rehosted. Source compatibility does not establish that Pump's deployed fee program accepts Beam's ID.
2. Replace the superseded CPMM candidate with an independently verified Raydium AMM v4 adapter. The venue stays Raydium AMM v4 (`raydium-io/raydium-amm` `91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6`, program `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8`, 19-account `Initialize2`). CPMM stays fail-closed. AMM v4 accepts classic SPL Token mints only. Token-2022 trades on the curve and is rejected for migration until a safe plan lands. LP is locked permanently; the current candidate's burn is not a fee claim, and a lock does not add a withdrawal path. Any post-graduation Beam fee is unset ([open owner decisions](OPEN_DECISIONS.md)). Keep migration gated.
3. Non-SOL migration, Raydium AMM v4 quote-chain routing, AMM-dependent creator/sharing/CTO operations, and compatible donation creation/collection. [Merged PR #12](https://github.com/shawkkkkk/beam/pull/12) makes a nested graduated `create_v2` PumpSwap quote fail closed; it does not quote AMM v4. Remove all active PumpSwap runtime, PDA and ownership assumptions outside the immutable upstream archive; fail-closed handlers are interim gaps, not the finished behavior.
4. Regenerated Beam IDL, complete wallet builders, real indexer and production UI. The merged native-SOL fixture encoder is only a local construction layer; retained dependency IDLs are reference interfaces, not a finalized Beam client API.
   Preserve creator-fee recipient selection and build independent, authorized services for donation choices, callouts, and livestreaming. The pinned on-chain source does not supply those off-chain products. See the [feature matrix](FEATURES.md).
5. SBF build and local-validator differential tests: every retained instruction, actual Raydium CPIs, rollback, authority checks, rent, extensions, replay rejection and LP treatment.
6. Deployment-key assignment, independent review and devnet end-to-end validation. This branch deploys nothing.

The [task ledger](TASKS.md) tracks current ownership and bounded follow-up
work. [PR #2](https://github.com/shawkkkkk/beam/pull/2) added checked
post-completion quote accounting, fee/rent liability protection, and
account-alias guards to the development branch. Those host-tested guards do
not establish CPI success or fee-sweep/rollback safety.
[The separate root-model PR #3](https://github.com/shawkkkkk/beam/pull/3)
was closed unmerged because its contract and fee policy conflict with the
current `programs/beam` baseline. Its reviewed sample-data website was
carried into merged [PR #6](https://github.com/shawkkkkk/beam/pull/6).

Creator payouts and creator-selected wallet behavior remain requirements.
They are paid from the 42 bps community share. The curve fee is 0.50%
all-in: 42/5/3 basis points for community/treasury/founder, including 8
basis points under platform control. The creator basis-point count and the
other unset values are only in [open owner decisions](OPEN_DECISIONS.md).
The optional disabled, three-second buy-side quote
prototype models an initial 99% fee of gross buyer debit. A 99% fee on net
base would be about 49.75% of gross (0.99 / 1.99), so it is not the same rule.
Its launch surcharge is separate from ordinary allocation. Its recipient is
unset in [open owner decisions](OPEN_DECISIONS.md), and the prototype has no
withdrawal path. There is no enabled contract path, live
trade, sell-handler change or promised outcome.

## Contribution and security status

At the last maintainer check on 1 November 2026, Beam's
[Slop listing](https://slop.cash/projects/beam) was paused with a security
scan queued. The page was inaccessible to this documentation review on
2 November 2026, so its current scan and listing status are **unknown here**.
Verify the live status and maintainer direction before claiming an available
Slop task, clearance, reward funding or a changed development scope. A Slop
task's automatic closure does not approve or merge a GitHub pull request.
The repository still has no root license; maintainers must clarify
contribution terms before inviting substantial external submissions.

## Meaning of checks

Host compilation checks Rust/Anchor types. Differential tests cover selected quote/fee functions, decoding/serialization boundaries and retained instruction encodings, including semantic errors and source line numbers. Build-directory prefixes are normalized because the reference is compiled from another directory.

These checks do not establish full instruction execution parity, SBF byte identity, CPI success, current-chain behavior or security. Source equality is separate from program-binary equality.

Any isolated local-bank harness uses synthetic accounts and valueless balances
to exercise Beam code. It does not execute captured original Pump binaries,
prove Raydium CPI behavior or submit a transaction to a live network.

Upstream declares the reconstruction complete and removed captured binaries and its research toolchain. This branch pins the final source; it does not independently re-prove the historical reconstruction claim.

The supplied ATH/fee comparisons motivate a product hypothesis. Different cohorts and sums of separate ATHs do not isolate venue effects or measure liquidity durability. Beam makes no claim that Raydium guarantees better price performance.
