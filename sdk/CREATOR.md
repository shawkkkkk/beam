# Local creator-fee client

This lane adds a local client for the retained **legacy native-SOL**
`collect_creator_fee` instruction, the shared `CollectCreatorFeeEvent` byte
layout, and a bounded preview of the handler's rent/balance branches. It does
not change the contract, authenticate chain state, connect to RPC, sign or send.
V2 token collection and sharing distribution are outside this interface.

The task-start source is `b8120a8a58b1375ba3b8f0d2a590c49923e18021` in
`shawkkkkk/beam`. The program ID is the existing undeployed development
placeholder `GdaQ7RHbGKGEL5Px66tQP78S67XcgxYAzFUwja71qSJP`.
The authoritative handlers and layouts are:

- `programs/beam/src/creator/collect_creator_fee.rs`: legacy account constraints,
  source-order refusal/no-op branches, rent arithmetic and event fields.
- `programs/beam/src/lib.rs`: the no-argument entrypoint and declared identity.
- `programs/beam/src/creator/distribute_creator_fees.rs`: the inherited
  `FEES` owner rejected by direct collection. This does not integrate/rehost
  that fee program for Beam.
- Anchor 0.31.1, pinned by the program manifest/lockfile: instruction/event
  discriminators, Borsh serialization and the two appended `event_cpi` accounts.

The creator handler's SHA-256 at this source is
`e06da0d7e32e77d08300f5750261f838deec0d0480df85d236edea09e06369c9`.
`sdk/beam-native.mjs` remains read-only; its public-key helpers are reused.
There is no repository-root license; this change does not declare one or alter
upstream notices.

## Interface and authorization boundary (defined before implementation)

`encodeCollectCreatorFee()` takes no arguments and returns a new eight-byte
Anchor instruction discriminator. Supplying arguments is an error.

`buildCollectCreatorFee(config, accounts)` returns `{ programId, keys, data }`.
`config` must explicitly contain only `sourceCommit` matching the source above
and `programId` matching the development ID. Missing/unsupported revisions or
identities fail closed; this is compatibility configuration, not deployment
verification. The exact account object has these five base accounts:

| Client name | Anchor field | Signer | Writable |
| --- | --- | --- | --- |
| `creator` | `creator` | false | true |
| `creatorVault` | `creator_vault` | false | true |
| `systemProgram` | `system_program` | false | false |
| `eventAuthority` | appended `event_authority` | false | false |
| `program` | appended `program` | false | false |

Every public key must decode to 32 bytes. The system and self-program identities
are checked. The helper does **not** derive or verify the creator-vault/event
PDAs, account owners/data, executable flags, outer transaction accounts or state.
Unknown fields (including `payoutWallet` and `remainingAccounts`) are errors.
Only these five base metas are supported; no extra accounts are silently added
or claimed to have been validated. The legacy handler does not read remaining
accounts. This object is for local construction/inspection, not proof that a
transaction can execute.

Collection is permissionless in the retained source: `creator` is an
`AccountInfo`, **not a signer**. The receiving address is that same `creator`;
the vault is constrained by `[b"creator-vault", creator.key()]`. The client
neither invents a second payout address nor authorizes arbitrary later creator
changes. A transaction fee payer/signature and actual runtime validation are
separate concerns not implemented here.

`decodeCollectCreatorFeeEvent(bytes)` accepts exactly 88 raw bytes: discriminator,
`timestamp:i64`, `creator:Pubkey`, `creator_fee:u64`, `quote_mint:Pubkey`, in that
order. It returns `{ timestamp, creator, creatorFee, quoteMint }`, with both
integers as `bigint`. Truncation, trailing bytes, the wrong discriminator and
non-byte inputs reject. Internal TypedArray brand and storage length are checked;
shadowed `length`/`byteLength` properties, fake prototypes and custom iterators
cannot replace the underlying bytes. Genuine Uint8Array/Buffer subarray offsets
are honored. The caller must keep storage stable during the synchronous copy;
this is not an atomic shared-memory snapshot. A decoded payload is
not proof of origin, finality, execution or a transfer. This event type is shared
by other handlers; a nonzero quote mint is structurally valid and does not prove
a legacy native collection. Log/base64/CPI envelopes are not accepted implicitly.

`previewCollectCreatorFee(config, snapshot)` accepts only these explicit fields:

- `creator`, `creatorVault`, `creatorOwner`: base58 public keys;
- `creatorExecutable`: a boolean;
- `creatorLamports`, `creatorRentMinimum`, `vaultLamports`,
  `vaultZeroDataRentMinimum`: exact `bigint` values in `0..=u64::MAX`;
- `observedSlot`, `referenceSlot`, `maxSlotAge`: exact `bigint` u64 values.

The zero-data rent minimum is specifically the supplied `Rent::minimum_balance(0)`
used by the handler; the creator minimum must correspond to the supplied
creator's actual data length. No rent schedule is invented. Both balances/rent
and slot metadata are caller supplied and unauthenticated. A future observation
or age exceeding `maxSlotAge` rejects; this only checks internal consistency
against the caller's reference slot, not real chain freshness. Missing or
malformed fields reject before branch preview.

For well-formed input the preview preserves the source order: reject inherited
sharing-program ownership, no-op for an executable creator, compute
`available = max(vaultLamports - vaultZeroDataRentMinimum, 0)`, no-op if zero,
and compare the **saturating** u64 sum of creator balance plus available against
the creator rent minimum. A successful comparison means only `transfer-attempt`:
`transferLamports` is the amount passed to System transfer if actual account
constraints and CPI conditions hold. `balanceSumExceedsU64` flags the raw sum's
overflow; this API does not promise that CPI accepts it. Other branches report
zero attempted transfer. No rewards, trading fees, post-graduation rights or
actual collectible entitlement are calculated.

## Acceptance and fixture provenance

Before publishing, tests must cover discriminator/field order, the five exact
metas and lack of creator-signer requirement, configuration refusal, malformed
keys/accounts/events, i64/u64 boundaries, executable/sharing-owner branches,
zero/under-rent/exact-rent amounts, saturating addition and stale/future slots.
All inputs are synthetic; no wallet or transaction is used.

`fixtures/creator-native-sol.json` is the exact 2,368-byte stdout from the
reviewed `tools/generate-creator-fixture.rs`, executed independently against a
separate task-start `b8120a8` Beam package. The actual Anchor 0.31.1 output
contains one no-argument discriminator, five ordered metas (all nonsigners) and
three 88-byte synthetic event payloads. `fixtures/creator-anchor-receipt.json`
records the observed command, exit 0, source/producer/output hashes and tool
versions. The producer SHA-256 is
`88a6ae88e0eb8ef02a4ae9f33f965fa125d2e3b7eb4d36ea88750a20e0dac89d`;
stdout SHA-256 is
`78eb6e27f773bdccf420e0d03a975c4e97f839b3095679d38f7b44e3620a2b44`.
These are actual `InstructionData`, `ToAccountMetas(None)` and `Event::data()`
results, not handler execution. The synthetic account keys are not valid PDA
examples; serialized type-minimum/maximum events are not transfer receipts.
The host build emitted existing contract warnings, retained by the independent
runner; they are not claimed absent or converted into SDK failures.

To regenerate, use a disposable checkout of the exact source commit and a
Rust/Cargo environment compatible with its locked package. Copy this producer
into a temporary conventional example in that scratch checkout, then run:

```sh
# In the separate disposable b8120a8 checkout; do not commit the temporary example.
mkdir -p programs/beam/examples
cp /path/to/creator-client/sdk/tools/generate-creator-fixture.rs \
  programs/beam/examples/beam_creator_fixture.rs
BEAM_CREATOR_FIXTURE_SOURCE_COMMIT="$(git rev-parse HEAD)" \
  cargo run --locked --manifest-path programs/beam/Cargo.toml \
  --example beam_creator_fixture > /tmp/creator-anchor.json
```

The original run used Rust 1.94.1, Cargo 1.89.0 and `RUSTFLAGS=-C debuginfo=0`;
its cache paths and exact environment are in the receipt. No contract/archive
file is changed in this contribution. The temporary example is outside the
source-provenance check's allowlist and belongs only in the disposable checkout.

Observed local checks: the author's Node 24.19.0 run and an independent peer's
Node 22.23.4 run initially each passed 20 tests. Three later raw-byte-view
regressions reproduced failures on the original decoder, then all 23 tests
(18 creator plus five retained native) passed after the intrinsic storage guard
fix on Node 22.23.4 and Node 24.19.0. These are local results,
not a hosted-CI claim:

```sh
node --test sdk/beam-native.test.mjs sdk/beam-creator.test.mjs  # 23 passed:18 creator+5 retained
python3 scripts/check_fork.py  # 380 archive blobs;62 retained equal,4 declared changes
git diff --check
```

Host serialization/SDK tests do not establish SBF/CPI success, PDA validation,
transaction rollback, authenticated events or a deployed Beam integration.
