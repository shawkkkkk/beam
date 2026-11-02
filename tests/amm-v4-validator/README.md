# AMM v4 migration validation

Local-validator evidence for task scope 3. This crate does not add a Beam
migration instruction and does not enable graduation.

It runs beside draft PRs #33 and #34. Those drafts exercise a synthetic AMM
processor and a source-layout snapshot. This harness cross-program invokes
`raydium_amm::processor::Processor::process` from the pinned source. It does
not copy those modules.

## Pinned program and config

| Item | Value |
| --- | --- |
| Source | `raydium-io/raydium-amm` `91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6` |
| Program id | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` (crate default, not the devnet feature) |
| Config PDA seed | `amm_config_account_seed` |
| Config layout | 544-byte `AmmConfig` |
| Pool layout | 752-byte `AmmInfo` |
| Target orders | 2208 bytes |
| Create-pool fee destination | `7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5` |
| AMM owner recorded on the pool | `GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ` |
| OpenBook program, not an account in this path | `srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX` |
| Instruction | tag `1`, 26 bytes, 19 accounts |

`CreateConfig` and `UpdateConfig` require the hardcoded mainnet admin signature.
This harness does not have that key. It installs the canonical config PDA as a
544-byte fixture with `create_pool_fee = 25_000_000` lamports. The pinned
processor reads that field and transfers it.

The runtime is `solana-program-test` 3.0.0. The AMM processor is the pinned
Rust entry, called through CPI. SPL Token and associated-token execution use
the BPF programs bundled with that framework (`spl_token` 3.5.0 and
`spl_associated_token_account` 1.1.1). This is not a deployed-bytecode match
and not an SBF build of the pinned AMM crate.

## What the tests cover

- One successful 19-account `Initialize2` CPI for classic SPL coin and native
  SOL pc. Pool owner, status 6, mint, vault, LP, market, and target-order
  identities match the pin. Vault amounts equal the planned reserves. User LP
  is `sqrt(coin * pc) - 10^decimals`.
- The configured create-pool fee is added to the canonical WSOL fee
  destination. A pre-existing donation on that account stays.
- Curve coin retained balance, curve pc liability, curve pc donation, and
  user-source donations are unchanged. Only the planned amounts move.
- A second migration of the same market is rejected before another CPI because
  the pool account is no longer empty. A direct forward CPI then receives the
  pinned program's `RepeatCreateAmm` (custom 49). Balances do not change.
- A pool PDA that already holds lamports is rejected and the curve balances
  stay put.
- After a successful inner CPI, the probe returns an error. The banks
  transaction rolls back the pool, fee debit, and curve transfers.
- A Token-2022 coin mint is rejected by the pinned program with
  `InvalidSplTokenProgram` (custom 38) after the fee transfer is attempted.
  The failed transaction leaves the fee destination, curve balances, and the
  curve-launch account unchanged. `programs/beam` create paths are not edited.

## Run

From this directory, with Rust 1.99.1:

```sh
cargo test --locked -- --test-threads=1
```

Migration remains disabled. LP custody, post-graduation fee rights, and a
Token-2022 graduation plan are still open. An SBF build of this source pin and
a `solana-test-validator` run were not produced here.
