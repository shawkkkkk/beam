# Upstream attribution

Beam uses the pump.fun ($PUMP) program work as published at
[lalalune/pumpfun-source](https://github.com/lalalune/pumpfun-source), byte for
byte, at this commit:

[`5703df02131ab944408e0cf679b4bef9c22e1a3d`](https://github.com/lalalune/pumpfun-source/commit/5703df02131ab944408e0cf679b4bef9c22e1a3d)

The pin is recorded in [`upstream/manifest.json`](../upstream/manifest.json).
The tree under `upstream/pumpfun-source` is that commit. Its 380 Git blobs,
paths, and file modes match the GitHub tree of the commit, and they match
[`upstream/files.json`](../upstream/files.json).

That repository is the completed community reconstruction of pump.fun's
on-chain programs. pump.fun is the original author of the program. Shaw
([@lalalune](https://github.com/lalalune)) and the
[pumpfun-source contributors](https://github.com/lalalune/pumpfun-source/graphs/contributors)
published the reconstruction. The archived MIT notice in
`upstream/pumpfun-source/LICENSE` reads "Copyright (c) 2026 pump.fun
contributors."

This repository does not re-prove that reconstruction from deployed pump.fun
bytecode. Byte identity here is identity with the published commit.

## What is byte-identical

`python3 scripts/check_fork.py` does two checks:

1. It hashes every path in `upstream/files.json` as a Git blob and compares
   that hash with the file under `upstream/pumpfun-source`.
2. It compares every file in the pinned Pump Main package,
   `upstream/pumpfun-source/programs/pump`, with the same path in
   `programs/beam`. A retained file whose bytes differ must be listed in
   `allowed_modified_paths`. A `programs/beam` file that is not in that
   package must be listed in `allowed_added_paths`.

On the current tree the command reports 380 archive blobs, 62 byte-identical
retained files, and four explicitly modified files. The modified paths it
prints are `Cargo.lock`, `Cargo.toml`, `src/create_v2.rs`, and `src/lib.rs`.

The 62 byte-identical retained files, relative to `programs/pump` and
`programs/beam`, are:

- `idls/mayhem.json`
- `idls/pump_amm.json`
- `idls/pump_fees.json`
- `link/ordered.ld`
- `pump-token-ext/Cargo.toml`
- `pump-token-ext/src/lib.rs`
- `src/admin_set_idl_authority.rs`
- `src/buy.rs`
- `src/buy_v2.rs`
- `src/cashback.rs`
- `src/create.rs`
- `src/creator/admin_cto.rs`
- `src/creator/collect_creator_fee.rs`
- `src/creator/distribute_creator_fee_v2.rs`
- `src/creator/distribute_creator_fees.rs`
- `src/creator/distribute_fee_to_holders.rs`
- `src/creator/migrate_bonding_curve_creator.rs`
- `src/creator/set_creator.rs`
- `src/creator/set_metaplex_creator.rs`
- `src/curve_quote_payments.rs`
- `src/extend_account.rs`
- `src/fee_cpi.rs`
- `src/fee_recipient.rs`
- `src/global_init.rs`
- `src/incentives.rs`
- `src/legacy_native_payments.rs`
- `src/mayhem_virtual_params.rs`
- `src/migrate.rs`
- `src/migrate_v2.rs`
- `src/migration_native_payments.rs`
- `src/multi_hop_curve_swap.rs`
- `src/pump_quote.rs`
- `src/quote_control/initialize_quote_control.rs`
- `src/quote_control/mod.rs`
- `src/quote_control/set_quote_control.rs`
- `src/quote_control/state.rs`
- `src/quote_control/update_quote_control_mints.rs`
- `src/sell.rs`
- `src/sell_v2.rs`
- `src/set_params.rs`
- `src/set_reserved_fee_recipients.rs`
- `src/sweep_fees.rs`
- `src/token_incentives/claim_token_incentives.rs`
- `src/token_incentives/close_user_volume_accumulator.rs`
- `src/token_incentives/init_user_volume_accumulator.rs`
- `src/token_incentives/mod.rs`
- `src/token_incentives/volume_accumulator.rs`
- `src/trade_types.rs`
- `src/trade_v3.rs`
- `src/util/constant_product.rs`
- `src/util/fee.rs`
- `src/util/payments.rs`
- `src/util/transfer_fee.rs`
- `src/util/user_payments.rs`
- `src/util/withheld.rs`
- `src/v2_native_payments.rs`
- `src/v2_quote_token_transfers.rs`
- `src/v2_recipient_payments.rs`
- `src/v2_token_transfers.rs`
- `src/v2_user_token_transfers.rs`
- `src/v3_native_transfer_fee.rs`
- `src/volume_sync.rs`

Those retained files include the inherited fee CPI, fee math, buy and sell
handlers, and creator-fee collection. Their bytes are the published Pump Main
bytes. The archive also keeps the other reconstructed programs — PumpSwap /
AMM, Fees, Mayhem, and Donation Relay — unmodified under
`upstream/pumpfun-source`. They are reference snapshots. They are not the
program crate Beam builds.

## Declared Beam edits

`programs/beam` is the Pump Main package plus the delta below. It is not an
unmodified copy of the publication, and Beam's compiled program is not claimed
to match pump.fun's deployed program. The published Pump program id in
`declare_id!` is `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`. Beam replaces
it with the undeployed placeholder
`GdaQ7RHbGKGEL5Px66tQP78S67XcgxYAzFUwja71qSJP`.

### Retained files whose bytes differ

These four paths are `allowed_modified_paths` in `upstream/manifest.json`.

- `Cargo.toml`. The package name is `beam-program`. Default features select
  `raydium-devnet`, and `experimental-raydium` is an empty feature flag. The
  `pump-token-ext` path points at the archived crate. A dev-dependency,
  `pump-reference`, compiles the pristine `pump-recovered` package for
  differential tests.
- `Cargo.lock`. The lockfile adds the `beam-program` package entry and its
  dependency on `pump-recovered`. The rest of the lockfile matches the pin.
- `src/lib.rs`. Besides the program id, the file adds
  `configure_raydium`, `set_raydium_enabled`, and `migrate_raydium`, and it
  declares `beam_policy`, `beam_economics`, `beam_amm_v4`, and `beam_raydium`.
  `toggle_mayhem_mode` and `set_mayhem_virtual_params` call
  `beam_policy::mayhem` before the retained handler, so Mayhem enablement is
  rejected. `migrate`, `migrate_v2`, `multi_hop_curve_swap`, and `admin_cto`
  call `beam_policy::legacy_amm` before the retained handler, so those
  PumpSwap interfaces fail closed before transfers or CPIs.
- `src/create_v2.rs`. Creation calls `beam_policy::mayhem`, so a Mayhem create
  is rejected. The graduated-ancestor quote branch no longer reads PumpSwap
  pool accounts. It returns `BeamError::PumpSwapDisabled`. The `AMM` and
  `pool_record` imports used by that read are removed.

### Files that exist only in Beam

These paths are `allowed_added_paths`. They are not in the pinned Pump Main
package.

- `src/beam_policy.rs`. Mayhem selection and the legacy PumpSwap migration,
  router, and AMM-dependent CTO entry points fail closed. Serialized Mayhem
  fields and account slots stay in the retained layouts.
- `src/beam_raydium.rs`. A Raydium CPMM migration candidate. The user selected
  Raydium AMM v4, so this candidate is superseded. `set_enabled(true)` and
  `execute` both return `MigrationNotEnabled`. The candidate burns LP and does
  not leave Beam an LP fee claim. It is not an AMM v4 adapter, and the retained
  quote-chain code does not route through Raydium.
- `src/beam_economics.rs`. Provisional allocation arithmetic for 42 / 5 / 3
  nominal basis points (community, treasury, founder), rounded once with split
  dust assigned to the community. It does not replace the retained creator-fee
  code in `src/fee_cpi.rs` or `src/util/fee.rs`, and it is not wired into trade
  collection. Creator payouts are paid from the 42 bps community share. The
  basis-point count is unset and is listed only in
  [open owner decisions](OPEN_DECISIONS.md). The inherited Pump fee CPI still
  governs the retained trade path.
- `src/beam_amm_v4.rs`. Source-only guards for the Raydium AMM v4
  `Initialize2` ABI at `raydium-io/raydium-amm`
  `91739d8d9f5cf347e0ce14ebe4d2bb7069fb31b6`. No Beam instruction calls this
  module. It performs no CPI and collects no fee.
- `tests/compatibility.rs`. Host tests that compile the pristine upstream
  crate beside Beam and compare selected retained math, layouts, and
  instruction data.

Any other byte change under `programs/beam`, or any added file outside that
list, fails `scripts/check_fork.py`.

## License notices

This repository has no root `LICENSE`. Beam's new files do not receive a
license from this document, and this document does not choose one.

The notices archived with the pinned sources stay in place:

- `upstream/pumpfun-source/LICENSE` — MIT License, "Copyright (c) 2026
  pump.fun contributors."
- `upstream/pumpfun-source/programs/donation-relay/error-model/LICENSE` and
  `NOTICE` — Apache-2.0 reconstruction adapted from Anchor, with its own
  copyright notice.
- `upstream/pumpfun-source/programs/mayhem/vendor/historical-anchor/LICENSE` —
  Apache-2.0.
- `upstream/raydium-cp-swap/LICENSE` — Apache-2.0 for the separate Raydium
  CPMM reference snapshot at `3d69122790e97cd6aff24e8ba95940793773f58c`. That
  pin is not the pump.fun program and is not part of the 380-blob
  pumpfun-source check. `upstream/manifest.json` also records the AMM v4 ABI
  reference above as Apache-2.0 source, without vendoring that repository.

## Separate reference pins

Raydium CPMM and Raydium AMM v4 are documented in the
[README source baseline](../README.md) and in `upstream/manifest.json`. They
are not the pump.fun publication, and byte identity with pumpfun-source does
not describe them.
