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
- Fee-allocation arithmetic and selected host differential tests.
- [PR #7](https://github.com/shawkkkkk/beam/pull/7) merged native-SOL create/buy/sell ABI fixtures and local encoders. They do not resolve live accounts, sign or submit transactions. [PR #13](https://github.com/shawkkkkk/beam/pull/13) is a separate draft for local legacy creator-fee fixtures; it is not part of this main-branch snapshot.
- [PR #6](https://github.com/shawkkkkk/beam/pull/6) merged a Beam-branded website preview with sample market data and browser-local drafts. It has no wallet, chain feed, metadata upload, donation, callout or livestream service.

## Outstanding before use

1. Beam-controlled fee configuration and vaults. Pump's inherited fee-program namespaces/authority assumptions must be replaced or rehosted. Source compatibility does not establish that Pump's deployed fee program accepts Beam's ID.
2. Replace the superseded CPMM candidate with an independently verified Raydium AMM v4 adapter. AMM v4 accepts classic SPL Token mints only, while Beam's retained curve also supports restricted Token-2022 mints; do not silently delete those launches or route them to an unsupported pool. Design principal-lock/fee-rights integration for recurring community buybacks and post-graduation creator claims; current candidate burns LP. Keep migration gated until the final venue, policy and tests are implemented.
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

Creator payouts and creator-selected wallet behavior remain requirements,
while funding from the community share versus an additional charge is
undecided. The optional 99%-to-base, three-second buy-only model has no
enabled contract path or validated specification; treat it as a disabled,
test-only proposal, with no live trades or promised outcome.

Beam's [Slop listing](https://slop.cash/projects/beam) was paused with a
security scan queued at the last maintainer check. Clearance and reward terms
have not been established in this repository.

## Meaning of checks

Host compilation checks Rust/Anchor types. Differential tests cover selected quote/fee functions, decoding/serialization boundaries and retained instruction encodings, including semantic errors and source line numbers. Build-directory prefixes are normalized because the reference is compiled from another directory.

These checks do not establish full instruction execution parity, SBF byte identity, CPI success, current-chain behavior or security. Source equality is separate from program-binary equality.

Upstream declares the reconstruction complete and removed captured binaries and its research toolchain. This branch pins the final source; it does not independently re-prove the historical reconstruction claim.

The supplied ATH/fee comparisons motivate a product hypothesis. Different cohorts and sums of separate ATHs do not isolate venue effects or measure liquidity durability. Beam makes no claim that Raydium guarantees better price performance.
