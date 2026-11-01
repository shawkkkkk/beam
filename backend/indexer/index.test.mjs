import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { PROGRAM_ID, encodeBase58Pubkey } from '../../sdk/beam-native.mjs';
import { BLOCK_VERSION, QUERY_VERSION, IndexerError, createFixtureIndexer } from './index.mjs';

const fixtureBytes = readFileSync(new URL('../../sdk/fixtures/native-sol.json', import.meta.url));
const fixture = JSON.parse(fixtureBytes);
const SOURCE = 'beam.native-sol.synthetic.v1';
const MAX = (1n << 64n) - 1n;
const pk = (number) => encodeBase58Pubkey(new Uint8Array(32).fill(number));
const event = (index = 0, changes = {}) => ({
  kind: 'complete-event', transactionIndex: 0, recordIndex: index,
  programId: PROGRAM_ID, dataHex: fixture.completeEvent.dataHex, ...changes,
});
const curve = (index = 0, changes = {}) => ({
  kind: 'curve-account', transactionIndex: 0, recordIndex: index,
  owner: PROGRAM_ID, address: fixture.completeEvent.expected.bondingCurve,
  dataHex: fixture.currentBondingCurve.dataHex, ...changes,
});
const block = (slot = '100', number = 100, parent = null, records = [event()], changes = {}) => ({
  version: BLOCK_VERSION, sourceId: SOURCE, slot, hash: pk(number), parent,
  commitment: 'processed', records, ...changes,
});
const parentOf = (value) => ({ slot: value.slot, hash: value.hash });
const store = (options = {}) => createFixtureIndexer({ sourceId: SOURCE, ...options });
const events = (indexer, options = {}) => indexer.query({ kind: 'complete-event', ...options });
const code = (expected) => (error) => error instanceof IndexerError && error.code === expected;
const state = (indexer) => JSON.stringify({
  description: indexer.describe(), events: events(indexer, { limit: 100 }),
  curves: indexer.query({ kind: 'curve-account', limit: 100 }),
});
const timestampHex = (timestamp) => {
  const bytes = Buffer.from(fixture.completeEvent.dataHex, 'hex');
  new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).setBigInt64(104, timestamp, true);
  return bytes.toString('hex');
};

test('existing SDK fixture produces exact decoded records with explicit local provenance', () => {
  const indexer = store();
  assert.equal(indexer.describe().status.state, 'unavailable');
  indexer.replay({ blocks: [block('100', 100, null, [curve(1), event(0)])] });
  const result = events(indexer);
  assert.equal(result.version, QUERY_VERSION);
  assert.equal(result.freshness, 'fixture');
  assert.equal(result.provenance.live, false);
  assert.equal(result.provenance.verifiedChain, false);
  assert.equal(result.provenance.fixtureReference.sha256, createHash('sha256').update(fixtureBytes).digest('hex'));
  assert.equal(result.status.state, 'fixture-ready');
  assert.deepEqual(result.items[0].decoded, fixture.completeEvent.expected);
  assert.deepEqual(indexer.latestCurve(fixture.completeEvent.expected.bondingCurve).item.decoded, fixture.currentBondingCurve.expected);
  assert.equal(result.items[0].programId, PROGRAM_ID);
  assert.equal(indexer.describe().indexedRecords, 2);
  assert.equal(Object.hasOwn(result, 'volume'), false);
  assert.equal(Object.hasOwn(result, 'price'), false);
});

test('canonical ordering, duplicate positions and exact replay are deterministic and idempotent', () => {
  const left = store(); const right = store();
  const a = block('100', 100, null, [event(9), event(1), event(9), event(0, { transactionIndex: 1 })]);
  const b = { ...a, records: [a.records[3], a.records[1], a.records[0]] };
  left.replay({ blocks: [a] }); right.replay({ blocks: [b] });
  assert.deepEqual(events(left, { limit: 1 }), events(right, { limit: 1 }));
  assert.deepEqual(events(left).items.map((item) => [item.transactionIndex, item.recordIndex]), [[0, 1], [0, 9], [1, 0]]);
  const before = state(left);
  left.replay({ blocks: [b, b] });
  assert.equal(state(left), before);
  assert.equal(left.describe().indexedRecords, 3);
});

test('an invalid later block rejects the whole batch without leaking earlier changes', () => {
  const indexer = store(); const root = block();
  indexer.replay({ blocks: [root] });
  const before = state(indexer);
  const second = block('101', 101, parentOf(root));
  const third = block('102', 102, parentOf(second), [event(0, { dataHex: '00' })]);
  assert.throws(() => indexer.replay({ blocks: [second, third] }), code('INVALID_DATA'));
  assert.equal(state(indexer), before);
  // The failed candidate did not reserve second's identity/fingerprint.
  indexer.replay({ blocks: [{ ...second, records: [event(0, { dataHex: timestampHex(456n) })] }] });
  assert.equal(events(indexer).items.at(-1).decoded.timestamp, '456');
});

test('fork replacement is explicit, atomic, and remembers conflicting orphan identities', () => {
  const indexer = store(); const root = block();
  const old = block('101', 101, parentOf(root), [event(0, { dataHex: timestampHex(111n) })]);
  indexer.replay({ blocks: [root, old] });
  const cursor = events(indexer, { limit: 1 }).nextCursor;
  const replacement = block('101', 103, parentOf(root), [event(0, { dataHex: timestampHex(222n) })]);
  assert.throws(() => indexer.replay({ blocks: [replacement] }), code('CHAIN_GAP'));
  const before = state(indexer);
  assert.throws(() => indexer.replay({ rollbackTo: parentOf(root), blocks: [{ ...replacement, records: [event(0, { dataHex: 'zz' })] }] }), code('INVALID_DATA'));
  assert.equal(state(indexer), before);
  indexer.replay({ rollbackTo: parentOf(root), blocks: [replacement] });
  assert.deepEqual(events(indexer).items.map((item) => item.decoded.timestamp), ['123', '222']);
  assert.throws(() => events(indexer, { limit: 1, cursor }), code('STALE_CURSOR'));
  const forkState = state(indexer);
  assert.throws(() => indexer.replay({ rollbackTo: parentOf(root), blocks: [{ ...old, records: [event(0, { dataHex: timestampHex(999n) })] }] }), code('CONFLICT'));
  assert.equal(state(indexer), forkState);
});

test('finality promotes ancestors, protects the boundary, and cannot be downgraded', () => {
  const indexer = store(); const root = block(); const second = block('101', 101, parentOf(root));
  const third = block('102', 102, parentOf(second));
  indexer.replay({ blocks: [root, second, third] });
  assert.equal(events(indexer, { commitment: 'finalized' }).items.length, 0);
  indexer.replay({ blocks: [{ ...second, commitment: 'finalized' }] });
  assert.deepEqual(indexer.describe().finalized, parentOf(second));
  assert.equal(events(indexer, { commitment: 'finalized' }).items.length, 2);
  const before = state(indexer);
  for (const rollbackTo of [null, parentOf(root)]) {
    assert.throws(() => indexer.replay({ rollbackTo, blocks: [] }), code('FINALIZED_BOUNDARY'));
    assert.equal(state(indexer), before);
  }
  indexer.replay({ blocks: [second] });
  assert.equal(state(indexer), before);
  indexer.replay({ rollbackTo: parentOf(second), blocks: [block('103', 104, parentOf(second))] });
  assert.deepEqual(indexer.describe().finalized, parentOf(second));
  assert.equal(indexer.describe().tip.slot, '103');
});

test('conflicting positions, block contents and source metadata reject without state change', () => {
  const indexer = store(); const root = block(); indexer.replay({ blocks: [root] });
  const before = state(indexer);
  for (const records of [[event(), event(0, { dataHex: timestampHex(3n) })], [event(), curve()]]) {
    assert.throws(() => indexer.replay({ blocks: [block('101', 101, parentOf(root), records)] }), code('CONFLICT'));
  }
  assert.throws(() => indexer.replay({ blocks: [{ ...root, records: [event(1)] }] }), code('CONFLICT'));
  assert.throws(() => indexer.replay({ blocks: [{ ...root, slot: '99' }] }), code('CONFLICT'));
  assert.throws(() => indexer.replay({ blocks: [block('101', 101, parentOf(root), [], { sourceId: 'other' })] }), code('INVALID_BLOCK'));
  assert.equal(state(indexer), before);
});

test('parent identity/order are exact while skipped and large slots are accepted', () => {
  const indexer = store(); const root = block((MAX - 5n).toString());
  const child = block((MAX - 1n).toString(), 101, parentOf(root));
  assert.throws(() => indexer.replay({ blocks: [child, root] }), code('CHAIN_GAP'));
  assert.equal(indexer.describe().indexedBlocks, 0);
  indexer.replay({ blocks: [root, child] });
  assert.equal(events(indexer, { fromSlot: child.slot, toSlot: MAX.toString() }).items.length, 1);
  const before = state(indexer);
  assert.throws(() => indexer.replay({ blocks: [block(MAX.toString(), 102, { slot: (MAX - 2n).toString(), hash: child.hash })] }), code('CHAIN_GAP'));
  assert.equal(state(indexer), before);
});

test('canonical integer, object, array and identity types are enforced before mutation', () => {
  const indexer = store(); const before = state(indexer);
  for (const slot of [100, 100n, '01', '+1', ' 1', '-1', (MAX + 1n).toString(), '9'.repeat(1000)]) {
    assert.throws(() => indexer.replay({ blocks: [block(slot)] }), code('INVALID_ARGUMENT'));
  }
  for (const recordIndex of [-1, -0, 1.5, NaN, 0x1_0000_0000, '0']) {
    assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [event(recordIndex)])] }), code('INVALID_ARGUMENT'));
  }
  for (const hash of ['0', '2', '1'.repeat(33), 'z'.repeat(1000)]) {
    assert.throws(() => indexer.replay({ blocks: [{ ...block(), hash }] }), code('INVALID_ARGUMENT'));
  }
  assert.throws(() => indexer.replay({ blocks: [{ ...block(), extra: true }] }), code('INVALID_ARGUMENT'));
  assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [{ ...event(), extra: true }])] }), code('INVALID_ARGUMENT'));
  let getterCalled = false;
  const accessor = { ...event() };
  Object.defineProperty(accessor, 'kind', { enumerable: true, get() { getterCalled = true; return 'complete-event'; } });
  assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [accessor])] }), code('INVALID_ARGUMENT'));
  assert.equal(getterCalled, false);
  assert.throws(() => indexer.replay({ blocks: new Array(1) }), code('INVALID_ARGUMENT'));
  assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, new Array(1))] }), code('INVALID_ARGUMENT'));
  assert.equal(state(indexer), before);
});

test('wrong event programs and account owners are rejected independently of decoded bytes', () => {
  const indexer = store(); const before = state(indexer);
  assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [event(0, { programId: pk(7) })])] }), code('PROGRAM_MISMATCH'));
  assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [curve(0, { owner: pk(7) })])] }), code('OWNER_MISMATCH'));
  assert.equal(state(indexer), before);
});

test('malformed lengths, discriminators, booleans and noncanonical hex reject atomically', () => {
  const indexer = store(); const before = state(indexer);
  const malformed = [event(0, { dataHex: fixture.completeEvent.dataHex.slice(2) }),
    event(0, { dataHex: `${fixture.completeEvent.dataHex}00` }),
    event(0, { dataHex: fixture.completeEvent.dataHex.toUpperCase() }),
    event(0, { dataHex: Buffer.from(fixture.completeEvent.dataHex, 'hex') })];
  const wrong = Buffer.from(fixture.completeEvent.dataHex, 'hex'); wrong[0] ^= 1;
  malformed.push(event(0, { dataHex: wrong.toString('hex') }));
  const boolean = Buffer.from(fixture.currentBondingCurve.dataHex, 'hex'); boolean[48] = 2;
  malformed.push(curve(0, { dataHex: boolean.toString('hex') }));
  for (const record of malformed) {
    assert.throws(() => indexer.replay({ blocks: [block('100', 100, null, [record])] }), code('INVALID_DATA'));
    assert.equal(state(indexer), before);
  }
});

test('u64 amounts and signed timestamps remain exact beyond Number precision', () => {
  const indexer = store();
  const bytes = Buffer.from(fixture.currentBondingCurve.dataHex, 'hex');
  new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).setBigUint64(8, MAX, true);
  indexer.replay({ blocks: [block(MAX.toString(), 100, null, [
    event(0, { dataHex: timestampHex(-(1n << 63n)) }), curve(1, { dataHex: bytes.toString('hex') }),
  ])] });
  assert.equal(indexer.describe().tip.slot, MAX.toString());
  assert.equal(events(indexer).items[0].decoded.timestamp, '-9223372036854775808');
  assert.equal(indexer.latestCurve(fixture.completeEvent.expected.bondingCurve).item.decoded.virtualTokenReserves, '18446744073709551615');
  assert.doesNotThrow(() => JSON.stringify(events(indexer)));
});

test('bounded pages, inclusive filters, commitments and latest account snapshots are explicit', () => {
  const indexer = store(); const root = block('100', 100, null, Array.from({ length: 7 }, (_, index) => event(index)));
  const second = block('103', 101, parentOf(root), [curve()], { commitment: 'confirmed' });
  const third = block('105', 102, parentOf(second), [curve(0, { dataHex: fixture.currentBondingCurve.dataHex })]);
  indexer.replay({ blocks: [root, second, third] });
  const ids = []; let cursor;
  do {
    const page = events(indexer, { limit: 2, ...(cursor ? { cursor } : {}) });
    assert.ok(page.items.length <= 2); ids.push(...page.items.map((item) => item.id)); cursor = page.nextCursor;
  } while (cursor);
  assert.equal(new Set(ids).size, 7);
  assert.equal(events(indexer, { mint: fixture.completeEvent.expected.mint }).items.length, 7);
  assert.equal(events(indexer, { mint: pk(99) }).items.length, 0);
  assert.equal(events(indexer, { fromSlot: '101', toSlot: '104' }).items.length, 0);
  assert.equal(indexer.latestCurve(fixture.completeEvent.expected.bondingCurve).item.block.slot, '105');
  assert.equal(indexer.latestCurve(fixture.completeEvent.expected.bondingCurve, { commitment: 'confirmed' }).item.block.slot, '103');
  assert.equal(indexer.latestCurve(pk(98)).item, null);
  for (const limit of [0, 101, null, '2', 1.5, -0]) assert.throws(() => events(indexer, { limit }), code('INVALID_ARGUMENT'));
  assert.throws(() => events(indexer, { address: pk(7) }), code('INVALID_ARGUMENT'));
  assert.throws(() => indexer.query({ kind: 'curve-account', mint: pk(7) }), code('INVALID_ARGUMENT'));
  assert.throws(() => events(indexer, { fromSlot: '9', toSlot: '8' }), code('INVALID_ARGUMENT'));
});

test('cursors bind filters, full contents, source identity, finality and source status', () => {
  const root = block('100', 100, null, [event(0), event(1), event(2)]); const indexer = store();
  indexer.replay({ blocks: [root] });
  const cursor = events(indexer, { limit: 1 }).nextCursor;
  assert.equal(events(indexer, { limit: 2, cursor }).items.length, 2);
  assert.equal(events(indexer, { limit: 1, cursor, fromSlot: '0', toSlot: MAX.toString() }).items[0].recordIndex, 1);
  assert.throws(() => events(indexer, { cursor, mint: fixture.completeEvent.expected.mint }), code('STALE_CURSOR'));
  const other = store({ sourceId: 'other' }); other.replay({ blocks: [{ ...root, sourceId: 'other' }] });
  assert.throws(() => events(other, { cursor }), code('STALE_CURSOR'));
  const changed = store(); changed.replay({ blocks: [{ ...root, records: [event(0), event(1), event(2, { dataHex: timestampHex(999n) })] }] });
  assert.throws(() => events(changed, { cursor }), code('STALE_CURSOR'));
  indexer.markSource({ state: 'error', detail: 'Synthetic adapter failed.' });
  assert.throws(() => events(indexer, { cursor }), code('STALE_CURSOR'));
  indexer.replay({ blocks: [root] });
  const promotedCursor = events(indexer, { limit: 1 }).nextCursor;
  indexer.replay({ blocks: [{ ...root, commitment: 'finalized' }] });
  assert.throws(() => events(indexer, { cursor: promotedCursor }), code('STALE_CURSOR'));
});

test('malformed, duplicate-key and out-of-bounds cursors fail closed', () => {
  const indexer = store(); indexer.replay({ blocks: [block('100', 100, null, [event(0), event(1)])] });
  const cursor = events(indexer, { limit: 1 }).nextCursor;
  for (const bad of [null, '', `${cursor}=`, `${cursor}!`, 'beamidx1.' + 'x'.repeat(2048), 'beamidx1._w']) {
    assert.throws(() => events(indexer, { cursor: bad }), code('INVALID_CURSOR'));
  }
  const parsed = JSON.parse(Buffer.from(cursor.slice(9), 'base64url').toString());
  const encode = (body) => `beamidx1.${Buffer.from(JSON.stringify({ ...body, checksum: createHash('sha256').update(JSON.stringify(body)).digest('hex') })).toString('base64url')}`;
  const { checksum, ...body } = parsed;
  assert.throws(() => events(indexer, { cursor: encode({ ...body, offset: 100 }) }), code('INVALID_CURSOR'));
  assert.throws(() => events(indexer, { cursor: encode({ ...body, offset: 100000 }) }), code('INVALID_CURSOR'));
  assert.throws(() => events(indexer, { cursor: encode({ ...body, sourceId: {} }) }), code('INVALID_CURSOR'));
  const duplicate = Buffer.from(cursor.slice(9), 'base64url').toString().replace('"v":1', '"v":1,"v":1');
  assert.throws(() => events(indexer, { cursor: `beamidx1.${Buffer.from(duplicate).toString('base64url')}` }), code('INVALID_CURSOR'));
});

test('capacity bounds include batch work, retained records and remembered orphan identities', () => {
  assert.throws(() => store({ limits: { maxBlocks: 0 } }), code('INVALID_ARGUMENT'));
  assert.throws(() => store({ limits: { maxRecords: 65537 } }), code('INVALID_ARGUMENT'));
  const root = block(); const child = block('101', 101, parentOf(root));
  const batch = store({ limits: { maxBatchBlocks: 1 } });
  assert.throws(() => batch.replay({ blocks: [root, child] }), code('CAPACITY'));
  assert.equal(batch.describe().indexedBlocks, 0);
  const perBlock = store({ limits: { maxRecordsPerBlock: 1 } });
  assert.throws(() => perBlock.replay({ blocks: [block('100', 100, null, [event(0), event(1)])] }), code('CAPACITY'));
  const retained = store({ limits: { maxRecords: 1 } }); retained.replay({ blocks: [root] });
  const before = state(retained);
  assert.throws(() => retained.replay({ blocks: [child] }), code('CAPACITY'));
  assert.equal(state(retained), before);
  const identities = store({ limits: { maxBlocks: 2 } }); identities.replay({ blocks: [root, child] });
  const identityState = state(identities);
  assert.throws(() => identities.replay({ rollbackTo: parentOf(root), blocks: [block('101', 102, parentOf(root))] }), code('CAPACITY'));
  assert.equal(state(identities), identityState);
});

test('input and returned objects cannot mutate indexed contents or configuration', () => {
  const indexer = store(); const input = block('100', 100, null, [event(), curve(1)]);
  indexer.replay({ blocks: [input] }); const before = state(indexer);
  input.records[0].dataHex = timestampHex(999n); input.hash = pk(99);
  const page = events(indexer); page.items[0].decoded.timestamp = '999'; page.items[0].dataHex = '00';
  page.provenance.sourceId = 'changed'; page.tip.hash = pk(98); page.limits.maxRecords = 1;
  indexer.latestCurve(fixture.completeEvent.expected.bondingCurve).item.decoded.creator = pk(97);
  assert.equal(state(indexer), before);
});

test('empty fixtures, unavailable, stale and error states remain explicit without invented freshness', () => {
  const indexer = store(); assert.equal(events(indexer).status.state, 'unavailable');
  indexer.replay({ blocks: [] });
  assert.equal(events(indexer).status.state, 'fixture-ready');
  assert.equal(events(indexer).tip, null);
  assert.deepEqual(events(indexer).items, []);
  indexer.replay({ blocks: [block()] });
  for (const stateName of ['unavailable', 'stale', 'error']) {
    indexer.markSource({ state: stateName, detail: 'Local fixture adapter is paused.' });
    assert.equal(events(indexer).status.state, stateName);
    assert.equal(events(indexer).items.length, 1);
    assert.equal(events(indexer).freshness, 'fixture');
    assert.equal(events(indexer).provenance.live, false);
  }
  const before = state(indexer);
  for (const value of [{ state: 'fresh', detail: 'wrong' }, { state: 'error', detail: '' }, { state: 'error', detail: 'x\n' }]) {
    assert.throws(() => indexer.markSource(value), code('INVALID_ARGUMENT'));
    assert.equal(state(indexer), before);
  }
  indexer.replay({ blocks: [block()] });
  assert.equal(events(indexer).status.state, 'fixture-ready');
});
