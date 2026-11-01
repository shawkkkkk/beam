# Local fixture indexer, interface v1

This dependency-free ESM library is a local, read-only development adapter.
It accepts caller-supplied synthetic blocks and exposes bounded queries. It
does not start a server, contact RPC, authenticate a chain, use a wallet, sign,
send transactions, or authorize trades or payouts.

The interface below is defined before implementation. It targets repository
base `b8120a8a58b1375ba3b8f0d2a590c49923e18021` and imports the existing
`sdk/beam-native.mjs` decoders without changing them. Its initial bytes come
from `sdk/fixtures/native-sol.json`, whose synthetic Anchor fixture identifies
contract source `65c96b5ef0f73338d77b39c4f001d4113fc77a44`. Beam's program ID
is an undeployed development identity. New synthetic variants are labeled
with their own local `sourceId`; none is a live chain observation.

## Adapter input: `beam-indexer-block/v1`

```js
{
  version: 'beam-indexer-block/v1',
  sourceId: 'beam.native-sol.synthetic.v1',
  slot: '100',
  hash: '<canonical base58 32-byte block identity>',
  parent: null, // or { slot: '99', hash: '<parent identity>' }
  commitment: 'processed', // processed | confirmed | finalized
  records: [
    {
      kind: 'complete-event',
      transactionIndex: 0,
      recordIndex: 0,
      programId: '<exact SDK PROGRAM_ID>',
      dataHex: '<exact 144-byte raw CompleteEvent payload, lowercase hex>'
    },
    {
      kind: 'curve-account',
      transactionIndex: 0,
      recordIndex: 1,
      owner: '<exact SDK PROGRAM_ID>',
      address: '<canonical base58 32-byte account address>',
      dataHex: '<exact 166-byte current BondingCurve bytes, lowercase hex>'
    }
  ]
}
```

Objects accept exactly their documented own data properties. Slots are
canonical unsigned-u64 decimal **strings**, never floating-point numbers.
Hashes and public keys must round-trip to exactly 32 bytes. Positions are
integer numbers in `0..2^32-1`; they specify the local adapter's order within a
block. An event position may represent an observed log index and an account
position a write index, but this adapter does not verify that assertion.
Record positions are unique across kinds. Identical repeats are deduplicated;
different contents at the same position fail the entire operation. Records
are sorted by transaction index and record index before indexing.

The source, program and account-owner fields are checked against the local
adapter configuration/SDK identity. These are caller-supplied assertions,
not RPC proofs, signature verification, PDA verification, or evidence of a
deployed Beam program. No mint association is invented for a curve account:
the current curve decoder has no mint field. Completion-event mint filters
use the actual decoded event field.

## Replay, finality and authorization boundary

`createFixtureIndexer({ sourceId, limits? })` creates an in-memory instance.
`sourceId` is a lowercase identifier of at most 64 characters. Limits are
bounded integer settings for known block identities, total records, records
per block, and blocks per replay. Orphan fingerprints remain bounded and
remembered so a previously seen block hash cannot acquire different contents.
Capacity failure rejects the operation; history is not silently evicted.

| Limit | Default | Maximum |
| --- | ---: | ---: |
| `maxBlocks` (including orphan fingerprints) | 128 | 1,024 |
| `maxRecords` (retained and per replay input) | 8,192 | 65,536 |
| `maxRecordsPerBlock` | 256 | 2,048 |
| `maxBatchBlocks` | 64 | 128 |

Every configured bound must be at least 1. Start a new local fixture instance
when its identity capacity is exhausted.

`replay({ blocks, rollbackTo? })` validates and applies the entire batch
atomically. No state, status, revision, cursor or finalized boundary changes
on failure. The first block has `parent: null`; later new blocks must name
the exact current tip and have a strictly greater slot. Skipped slots are
permitted. Exact block replays are idempotent regardless of input record
order. A duplicate hash with different slot, parent or contents is a conflict.
Commitment may advance; a duplicate with an older commitment cannot downgrade
retained finality.

To switch forks, explicitly provide `rollbackTo: { slot, hash }` naming a
retained ancestor before adding the replacement suffix. `rollbackTo: null`
clears history only when no block is finalized. A finalized block and its
ancestors cannot be removed. Finalizing a block also qualifies its ancestors
as finalized in this local history. Finality is fixture metadata, not a
validator attestation. Invalid replacement data leaves the original fork
intact.

Only decoded completion events and curve-account snapshots are retained.
Amounts and timestamps come from SDK `bigint` values and are returned as
exact canonical decimal strings, including signed event timestamps. There
are no derived trade volumes, prices, market caps, holder counts, fee claims,
or live freshness claims.

## Bounded read API: `beam-indexer-query/v1`

`query({ kind, mint?, address?, fromSlot?, toSlot?, commitment?, limit?, cursor? })`
returns ascending canonical order, a bounded page, and `nextCursor` or null.
`kind` is `complete-event` or `curve-account`; `mint` is only valid for events,
and `address` only for curves. Commitment defaults to `processed`; a minimum
commitment filter may request `confirmed` or `finalized`. Slot ranges are
inclusive. Page limits are 1..100, default 25. Unknown fields and malformed
types fail closed. `latestCurve(address, { commitment? })` returns the latest
eligible snapshot, or null, with the same metadata.

Every response includes the schema version, explicit fixture provenance, tip
and finalized block identities, an exact revision, and status. Status is
`unavailable` before any replay, then `fixture-ready` after replay.
`markSource({ state, detail })` can report only `unavailable`, `stale`, or
`error`, preserving previously indexed data while labeling it honestly.
A valid subsequent replay restores `fixture-ready`. No state is called live
or fresh. An empty fixture chain remains distinguishable from a missing
source, and query results never infer clock freshness from event timestamps.

Cursors are bounded, canonical, opaque consistency tokens. They bind the
source, full retained-state digest/revision, canonical filters, and offset.
Rollback, replacement, finality/status changes, changed filters or a different
dataset invalidate a cursor. Exact idempotent replay preserves it. Their
public checksum detects malformed tokens; cursors are not authentication or
access-control credentials. Callers may read all retained fixture data through
bounded pages. Returned data is detached from the internal state.

## Local acceptance

Run with Node 22 (or a compatible newer Node) from the repository root:

```sh
node --test backend/indexer/*.test.mjs sdk/beam-native.test.mjs
python3 scripts/check_fork.py
git diff --check
```

Tests cover canonical ordering, deduplication/conflicts, atomic failure,
explicit fork replacement, finalized-boundary protection, cursor binding,
owner/program rejection, exact byte lengths/booleans, exact bigint values,
bounded queries/capacity, detached results and fixture/error statuses. No RPC,
server, wallet, signing or contract execution is part of these checks. Host
tests do not establish chain parity, Solana rollback, finality, or security.
