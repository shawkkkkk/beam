# Implementation status

## Implemented

- Complete final Pump archive pinned and verified by original Git blob hashes.
- Editable Pump Main fork, with unchanged files retaining their exact bytes.
- Separate Beam modules; hooks retain existing source line positions.
- Mayhem creation/enablement rejected. Inert serialized flags/account slots remain; Beam does not deploy the companion Mayhem program.
- Legacy `migrate`, `migrate_v2`, `multi_hop_curve_swap` and `admin_cto` handlers fail before transfers/CPIs. They are temporarily unavailable rather than silently routed back to PumpSwap. Other retained quote-chain code still understands historical PumpSwap layouts; it is not yet a Raydium router.
- New `configure_raydium`, `set_raydium_enabled` and `migrate_raydium` candidates.
- Gated native-SOL candidate validates the fixed executable, canonical administrator-pinned AmmConfig, mint ordering, vault/LP/observation PDAs, completion and real reserves/rent. It funds WSOL, invokes initialize, checks/burns received LP, records a one-time receipt and clears curve reserve fields in the same instruction. Runtime rollback remains unverified.
- Caller setup funding is separate from curve liquidity. The budget covers Raydium CPIs; outer ATA/receipt rent and transaction fees are additional. Refunds cannot exceed the caller's unspent contribution or include prior authority donations.
- Standard SPL Token and a restricted Token-2022 metadata extension subset supported by the migration candidate. Other extensions and non-SOL quote assets fail closed.
- Inherited Global migration-fee behavior retained. The new 45/5 bps allocation does not silently rewrite creator or migration fees.
- Fee-allocation arithmetic and selected host differential tests.

## Outstanding before use

1. Beam-controlled fee configuration and vaults. Pump's inherited fee-program namespaces/authority assumptions must be replaced or rehosted. Source compatibility does not establish that Pump's deployed fee program accepts Beam's ID.
2. Principal-lock/fee-rights integration for recurring community buybacks. Current candidate burns LP; keep migration gated until the final policy is implemented and tested.
3. Non-SOL migration, Raydium quote-chain routing and AMM-dependent creator operations.
4. Regenerated Beam IDL, wallet builders, real indexer and production UI. Retained dependency IDLs are reference interfaces, not a finalized Beam client API.
5. SBF build and local-validator differential tests: every retained instruction, actual Raydium CPIs, rollback, authority checks, rent, extensions, replay rejection and LP treatment.
6. Deployment-key assignment, independent review and devnet end-to-end validation. This branch deploys nothing.

The [task ledger](TASKS.md) tracks current ownership and bounded follow-up
work. [PR #2](https://github.com/shawkkkkk/beam/pull/2) added checked
post-completion quote accounting, fee/rent liability protection, and
account-alias guards to the development branch. Those host-tested guards do
not establish CPI success or fee-sweep/rollback safety.
[The separate root-model PR #3](https://github.com/shawkkkkk/beam/pull/3)
has different venue and economic assumptions and is not the current
`programs/beam` contract; its website uses sample data.

## Meaning of checks

Host compilation checks Rust/Anchor types. Differential tests cover selected quote/fee functions, decoding/serialization boundaries and retained instruction encodings, including semantic errors and source line numbers. Build-directory prefixes are normalized because the reference is compiled from another directory.

These checks do not establish full instruction execution parity, SBF byte identity, CPI success, current-chain behavior or security. Source equality is separate from program-binary equality.

Upstream declares the reconstruction complete and removed captured binaries and its research toolchain. This branch pins the final source; it does not independently re-prove the historical reconstruction claim.

The supplied ATH/fee comparisons motivate a product hypothesis. Different cohorts and sums of separate ATHs do not isolate venue effects or measure liquidity durability. Beam makes no claim that Raydium guarantees better price performance.
