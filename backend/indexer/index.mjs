import { createHash } from 'node:crypto';
import {
  PROGRAM_ID, decodeBase58Pubkey, encodeBase58Pubkey,
  decodeCompleteEvent, decodeCurrentBondingCurve,
} from '../../sdk/beam-native.mjs';

export const BLOCK_VERSION = 'beam-indexer-block/v1';
export const QUERY_VERSION = 'beam-indexer-query/v1';
const U64_MAX = (1n << 64n) - 1n;
const RANK = Object.freeze({ processed: 0, confirmed: 1, finalized: 2 });
const DEFAULTS = Object.freeze({ maxBlocks: 128, maxRecords: 8192, maxRecordsPerBlock: 256, maxBatchBlocks: 64 });
const MAXIMUMS = Object.freeze({ maxBlocks: 1024, maxRecords: 65536, maxRecordsPerBlock: 2048, maxBatchBlocks: 128 });
const has = (value, key) => Object.hasOwn(value, key);
const hash = (value) => createHash('sha256').update(value).digest('hex');

export class IndexerError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'IndexerError';
    this.code = code;
  }
}

function fail(code, message) { throw new IndexerError(code, message); }

function object(value, required, optional, name) {
  if (value === null || typeof value !== 'object'
    || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    fail('INVALID_ARGUMENT', `${name} must be a plain object`);
  }
  const allowed = new Set([...required, ...optional]);
  for (const key of Reflect.ownKeys(value)) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    if (typeof key !== 'string' || !allowed.has(key)
      || !has(descriptor, 'value') || !descriptor.enumerable) {
      fail('INVALID_ARGUMENT', `${name} has an unsupported property`);
    }
  }
  if (required.some((key) => !has(value, key))) fail('INVALID_ARGUMENT', `${name} is missing a required property`);
}

function integer(value, maximum, name, minimum = 0) {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || Object.is(value, -0) || value < minimum || value > maximum) {
    fail('INVALID_ARGUMENT', `${name} must be an integer in ${minimum}..${maximum}`);
  }
  return value;
}

function array(value, maximum, name) {
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype || value.length > maximum) fail('CAPACITY', `${name} exceeds its array bound`);
  for (let index = 0; index < value.length; index += 1) {
    const descriptor = Object.getOwnPropertyDescriptor(value, String(index));
    if (!descriptor || !has(descriptor, 'value') || !descriptor.enumerable) fail('INVALID_ARGUMENT', `${name} must contain only data elements`);
  }
  if (Reflect.ownKeys(value).length !== value.length + 1) fail('INVALID_ARGUMENT', `${name} has unsupported properties`);
  return value;
}

function u64(value, name) {
  if (typeof value !== 'string' || value.length > 20 || !/^(0|[1-9][0-9]*)$/.test(value)) {
    fail('INVALID_ARGUMENT', `${name} must be a canonical decimal u64 string`);
  }
  const number = BigInt(value);
  if (number > U64_MAX) fail('INVALID_ARGUMENT', `${name} exceeds u64`);
  return number;
}

function key(value, name) {
  try {
    if (encodeBase58Pubkey(decodeBase58Pubkey(value)) !== value) throw new Error('noncanonical');
  } catch {
    fail('INVALID_ARGUMENT', `${name} must be a canonical 32-byte base58 value`);
  }
  return value;
}

function commitment(value) {
  if (typeof value !== 'string' || !has(RANK, value)) fail('INVALID_ARGUMENT', 'invalid commitment');
  return value;
}

function freeze(value) {
  if (value !== null && typeof value === 'object' && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
}

function detached(value) { return structuredClone(value); }

function decodedStrings(decoded) {
  return Object.fromEntries(Object.entries(decoded).map(([name, value]) => {
    if (typeof value === 'bigint') return [name, value.toString()];
    if (!['string', 'boolean', 'number'].includes(typeof value)
      || (typeof value === 'number' && (!Number.isSafeInteger(value) || value < 0))) fail('INVALID_DATA', 'unsupported decoder result');
    return [name, value];
  }));
}

function normalizeRecord(input) {
  if (input === null || typeof input !== 'object') fail('INVALID_ARGUMENT', 'record must be an object');
  // Inspect the property descriptor before reading a caller-owned discriminator.
  const kindDescriptor = Object.getOwnPropertyDescriptor(input, 'kind');
  if (!kindDescriptor || !has(kindDescriptor, 'value')) fail('INVALID_ARGUMENT', 'record kind must be a data property');
  const kind = kindDescriptor.value;
  const event = kind === 'complete-event';
  if (!event && kind !== 'curve-account') fail('INVALID_ARGUMENT', 'unsupported record kind');
  object(input, event
    ? ['kind', 'transactionIndex', 'recordIndex', 'programId', 'dataHex']
    : ['kind', 'transactionIndex', 'recordIndex', 'owner', 'address', 'dataHex'], [], 'record');
  const transactionIndex = integer(input.transactionIndex, 0xffff_ffff, 'transactionIndex');
  const recordIndex = integer(input.recordIndex, 0xffff_ffff, 'recordIndex');
  const identity = event
    ? { programId: key(input.programId, 'programId') }
    : { owner: key(input.owner, 'owner'), address: key(input.address, 'address') };
  if (event && identity.programId !== PROGRAM_ID) fail('PROGRAM_MISMATCH', 'event program is not the configured development program');
  if (!event && identity.owner !== PROGRAM_ID) fail('OWNER_MISMATCH', 'curve owner is not the configured development program');
  const size = event ? 144 : 166;
  if (typeof input.dataHex !== 'string' || input.dataHex.length !== size * 2 || !/^[0-9a-f]+$/.test(input.dataHex)) {
    fail('INVALID_DATA', `record requires exactly ${size} lowercase hex bytes`);
  }
  let decoded;
  try {
    const bytes = Buffer.from(input.dataHex, 'hex');
    decoded = decodedStrings(event ? decodeCompleteEvent(bytes) : decodeCurrentBondingCurve(bytes));
  } catch (error) {
    if (error instanceof IndexerError) throw error;
    fail('INVALID_DATA', `malformed ${kind} bytes`);
  }
  const raw = { kind, transactionIndex, recordIndex, ...identity, dataHex: input.dataHex };
  return freeze({ ...raw, decoded, fingerprint: hash(JSON.stringify(raw)) });
}

function normalizeBlock(input, sourceId, limits) {
  object(input, ['version', 'sourceId', 'slot', 'hash', 'parent', 'commitment', 'records'], [], 'block');
  if (input.version !== BLOCK_VERSION || input.sourceId !== sourceId) fail('INVALID_BLOCK', 'block version/source does not match this adapter');
  const slot = u64(input.slot, 'slot');
  const blockHash = key(input.hash, 'block hash');
  let parent = null;
  if (input.parent !== null) {
    object(input.parent, ['slot', 'hash'], [], 'parent');
    const parentSlot = u64(input.parent.slot, 'parent slot');
    const parentHash = key(input.parent.hash, 'parent hash');
    if (parentSlot >= slot || parentHash === blockHash) fail('INVALID_BLOCK', 'parent must precede its child');
    parent = { slot: input.parent.slot, hash: parentHash };
  }
  const level = commitment(input.commitment);
  array(input.records, limits.maxRecordsPerBlock, 'block records');
  const unique = new Map();
  for (const inputRecord of input.records) {
    const record = normalizeRecord(inputRecord);
    const position = `${record.transactionIndex}:${record.recordIndex}`;
    const previous = unique.get(position);
    if (previous && previous.fingerprint !== record.fingerprint) fail('CONFLICT', 'conflicting records at one block position');
    unique.set(position, record);
  }
  const records = [...unique.values()].sort((a, b) => a.transactionIndex - b.transactionIndex || a.recordIndex - b.recordIndex);
  const fingerprint = hash(JSON.stringify({ sourceId, slot: input.slot, hash: blockHash, parent, records: records.map((record) => record.fingerprint) }));
  return freeze({ slot: input.slot, hash: blockHash, parent, commitment: level, records, fingerprint });
}

function normalizeFilters(input) {
  object(input, ['kind'], ['mint', 'address', 'fromSlot', 'toSlot', 'commitment', 'limit', 'cursor'], 'query');
  if (!['complete-event', 'curve-account'].includes(input.kind)) fail('INVALID_ARGUMENT', 'invalid query kind');
  if (has(input, 'mint') && input.kind !== 'complete-event') fail('INVALID_ARGUMENT', 'mint filters require completion events');
  if (has(input, 'address') && input.kind !== 'curve-account') fail('INVALID_ARGUMENT', 'address filters require curve accounts');
  const fromSlot = has(input, 'fromSlot') ? input.fromSlot : '0';
  const toSlot = has(input, 'toSlot') ? input.toSlot : U64_MAX.toString();
  if (u64(fromSlot, 'fromSlot') > u64(toSlot, 'toSlot')) fail('INVALID_ARGUMENT', 'slot range is reversed');
  return {
    kind: input.kind,
    mint: has(input, 'mint') ? key(input.mint, 'mint') : null,
    address: has(input, 'address') ? key(input.address, 'address') : null,
    fromSlot, toSlot,
    commitment: has(input, 'commitment') ? commitment(input.commitment) : 'processed',
  };
}

function cursorEncode(body) {
  const value = { ...body, checksum: hash(JSON.stringify(body)) };
  return `beamidx1.${Buffer.from(JSON.stringify(value)).toString('base64url')}`;
}

function cursorDecode(token, binding, maximumOffset) {
  if (typeof token !== 'string' || token.length > 2048 || !/^beamidx1\.[A-Za-z0-9_-]+$/.test(token)) fail('INVALID_CURSOR', 'invalid cursor encoding');
  let parsed;
  let text;
  try {
    const encoded = token.slice(9);
    const bytes = Buffer.from(encoded, 'base64url');
    if (bytes.toString('base64url') !== encoded) throw new Error('noncanonical');
    text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    parsed = JSON.parse(text);
    object(parsed, ['v', 'sourceId', 'stateDigest', 'revision', 'filtersHash', 'offset', 'checksum'], [], 'cursor');
    if (parsed.v !== 1) throw new Error('version');
    if (typeof parsed.sourceId !== 'string' || !/^[a-z0-9][a-z0-9._-]{0,63}$/.test(parsed.sourceId)) throw new Error('source');
    integer(parsed.offset, maximumOffset, 'cursor offset');
    u64(parsed.revision, 'cursor revision');
    for (const name of ['stateDigest', 'filtersHash', 'checksum']) {
      if (typeof parsed[name] !== 'string' || !/^[0-9a-f]{64}$/.test(parsed[name])) throw new Error('hash');
    }
  } catch {
    fail('INVALID_CURSOR', 'invalid cursor payload');
  }
  const body = { v: 1, sourceId: parsed.sourceId, stateDigest: parsed.stateDigest, revision: parsed.revision, filtersHash: parsed.filtersHash, offset: parsed.offset };
  if (JSON.stringify({ ...body, checksum: parsed.checksum }) !== text || hash(JSON.stringify(body)) !== parsed.checksum) fail('INVALID_CURSOR', 'cursor checksum/canonical form differs');
  for (const name of ['sourceId', 'stateDigest', 'revision', 'filtersHash']) {
    if (body[name] !== binding[name]) fail('STALE_CURSOR', 'cursor does not belong to this dataset, state, or query');
  }
  return body.offset;
}

class FixtureIndexer {
  #sourceId; #limits; #provenance;
  #blocks = []; #known = new Map(); #finalizedHash = null;
  #sourceState = { state: 'unavailable', detail: 'No fixture blocks have been replayed.' };
  #revision = 0n; #stateDigest;

  constructor(options) {
    object(options, ['sourceId'], ['limits'], 'options');
    if (typeof options.sourceId !== 'string' || !/^[a-z0-9][a-z0-9._-]{0,63}$/.test(options.sourceId)) fail('INVALID_ARGUMENT', 'sourceId must be a lowercase local identifier');
    const limits = { ...DEFAULTS };
    if (has(options, 'limits')) {
      object(options.limits, [], Object.keys(DEFAULTS), 'limits');
      for (const [name, value] of Object.entries(options.limits)) limits[name] = integer(value, MAXIMUMS[name], name, 1);
    }
    this.#sourceId = options.sourceId;
    this.#limits = freeze(limits);
    this.#provenance = freeze({
      kind: 'synthetic-fixture', live: false, verifiedChain: false,
      sourceId: options.sourceId, programId: PROGRAM_ID,
      sdkBaselineCommit: 'b8120a8a58b1375ba3b8f0d2a590c49923e18021',
      decoderBaselineSha256: '7fa25c7da37bd65e115b9eaf8456f4e590c8b2f0315ecc0437480d90afefd39b',
      fixtureReference: {
        path: 'sdk/fixtures/native-sol.json',
        sourceCommit: '65c96b5ef0f73338d77b39c4f001d4113fc77a44',
        sha256: '214b8911b0c360847ce16c0c556587eb3e12b691974d106b86bc10f264cccd78',
      },
    });
    this.#stateDigest = this.#digest(this.#blocks, this.#known, this.#finalizedHash, this.#sourceState);
  }

  #digest(blocks, known, finalizedHash, sourceState) {
    return hash(JSON.stringify({ provenance: this.#provenance, limits: this.#limits,
      blocks: blocks.map((block) => [block.hash, block.fingerprint, block.commitment]),
      known: [...known.entries()].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0), finalizedHash, sourceState }));
  }

  #commit(blocks, known, finalizedHash, sourceState) {
    const digest = this.#digest(blocks, known, finalizedHash, sourceState);
    if (digest === this.#stateDigest) return this.describe();
    if (this.#revision === U64_MAX) fail('CAPACITY', 'revision capacity exhausted');
    this.#blocks = blocks;
    this.#known = known;
    this.#finalizedHash = finalizedHash;
    this.#sourceState = sourceState;
    this.#stateDigest = digest;
    this.#revision += 1n;
    return this.describe();
  }

  #finalizedIndex(blocks = this.#blocks, finalizedHash = this.#finalizedHash) {
    return finalizedHash === null ? -1 : blocks.findIndex((block) => block.hash === finalizedHash);
  }

  #effective(block, index) {
    return index <= this.#finalizedIndex() ? 'finalized' : block.commitment;
  }

  describe() {
    const finalized = this.#blocks[this.#finalizedIndex()];
    const tip = this.#blocks.at(-1);
    const identity = (block) => block ? { slot: block.slot, hash: block.hash } : null;
    return detached({ version: QUERY_VERSION, provenance: this.#provenance,
      freshness: 'fixture', status: this.#sourceState,
      revision: this.#revision.toString(), stateDigest: this.#stateDigest,
      tip: identity(tip), finalized: identity(finalized),
      indexedBlocks: this.#blocks.length,
      indexedRecords: this.#blocks.reduce((total, block) => total + block.records.length, 0),
      knownBlockIdentities: this.#known.size, limits: this.#limits });
  }

  replay(input) {
    object(input, ['blocks'], ['rollbackTo'], 'replay');
    array(input.blocks, this.#limits.maxBatchBlocks, 'replay blocks');
    let inputRecords = 0;
    for (const block of input.blocks) {
      const descriptor = block && typeof block === 'object' ? Object.getOwnPropertyDescriptor(block, 'records') : null;
      if (!descriptor || !has(descriptor, 'value') || !Array.isArray(descriptor.value)) fail('INVALID_ARGUMENT', 'block records must be an array data property');
      inputRecords += descriptor.value.length;
      if (inputRecords > this.#limits.maxRecords) fail('CAPACITY', 'replay input records exceed the configured bound');
    }
    const blocks = this.#blocks.slice();
    const known = new Map(this.#known);
    let finalizedHash = this.#finalizedHash;
    if (has(input, 'rollbackTo')) {
      let index = -1;
      if (input.rollbackTo !== null) {
        object(input.rollbackTo, ['slot', 'hash'], [], 'rollback target');
        u64(input.rollbackTo.slot, 'rollback slot');
        key(input.rollbackTo.hash, 'rollback hash');
        index = blocks.findIndex((block) => block.hash === input.rollbackTo.hash && block.slot === input.rollbackTo.slot);
        if (index < 0) fail('CHAIN_GAP', 'rollback target is not a retained block');
      }
      if (index < this.#finalizedIndex(blocks, finalizedHash)) fail('FINALIZED_BOUNDARY', 'rollback would remove finalized history');
      blocks.splice(index + 1);
    }
    for (const raw of input.blocks) {
      const block = normalizeBlock(raw, this.#sourceId, this.#limits);
      if (known.has(block.hash) && known.get(block.hash) !== block.fingerprint) fail('CONFLICT', 'block identity has conflicting contents');
      let index = blocks.findIndex((candidate) => candidate.hash === block.hash);
      if (index >= 0) {
        if (RANK[block.commitment] > RANK[blocks[index].commitment]) blocks[index] = freeze({ ...blocks[index], commitment: block.commitment });
      } else {
        const tip = blocks.at(-1);
        if (tip ? block.parent?.hash !== tip.hash || block.parent.slot !== tip.slot : block.parent !== null) fail('CHAIN_GAP', 'new block must extend the exact retained tip');
        if (tip && BigInt(block.slot) <= BigInt(tip.slot)) fail('CHAIN_GAP', 'block slots must increase');
        if (!known.has(block.hash) && known.size === this.#limits.maxBlocks) fail('CAPACITY', 'known block identity capacity exhausted');
        known.set(block.hash, block.fingerprint);
        blocks.push(block);
        index = blocks.length - 1;
      }
      if (block.commitment === 'finalized' && index > this.#finalizedIndex(blocks, finalizedHash)) finalizedHash = block.hash;
      if (blocks.reduce((total, candidate) => total + candidate.records.length, 0) > this.#limits.maxRecords) fail('CAPACITY', 'retained record capacity exhausted');
    }
    return this.#commit(blocks, known, finalizedHash, { state: 'fixture-ready', detail: null });
  }

  markSource(input) {
    object(input, ['state', 'detail'], [], 'source status');
    if (!['unavailable', 'stale', 'error'].includes(input.state)
      || typeof input.detail !== 'string' || input.detail.length < 1 || input.detail.length > 200
      || /[\u0000-\u001f\u007f]/.test(input.detail)) fail('INVALID_ARGUMENT', 'invalid fixture source status');
    return this.#commit(this.#blocks, this.#known, this.#finalizedHash, { state: input.state, detail: input.detail });
  }

  #matching(filters) {
    const matches = [];
    for (let index = 0; index < this.#blocks.length; index += 1) {
      const block = this.#blocks[index];
      const level = this.#effective(block, index);
      if (RANK[level] < RANK[filters.commitment] || BigInt(block.slot) < BigInt(filters.fromSlot) || BigInt(block.slot) > BigInt(filters.toSlot)) continue;
      for (const record of block.records) {
        if (record.kind !== filters.kind || (filters.mint !== null && record.decoded.mint !== filters.mint)
          || (filters.address !== null && record.address !== filters.address)) continue;
        const { fingerprint, ...fields } = record;
        matches.push({ id: `${block.hash}:${record.transactionIndex}:${record.recordIndex}`, ...fields,
          block: { slot: block.slot, hash: block.hash, commitment: level } });
      }
    }
    return matches;
  }

  query(input) {
    const filters = normalizeFilters(input);
    const limit = has(input, 'limit') ? integer(input.limit, 100, 'limit', 1) : 25;
    const binding = { v: 1, sourceId: this.#sourceId, stateDigest: this.#stateDigest,
      revision: this.#revision.toString(), filtersHash: hash(JSON.stringify(filters)) };
    const matches = this.#matching(filters);
    const offset = has(input, 'cursor') ? cursorDecode(input.cursor, binding, this.#limits.maxRecords) : 0;
    if (offset > matches.length) fail('INVALID_CURSOR', 'cursor offset exceeds this result set');
    const nextOffset = Math.min(offset + limit, matches.length);
    return detached({ ...this.describe(), items: matches.slice(offset, nextOffset),
      nextCursor: nextOffset < matches.length ? cursorEncode({ ...binding, offset: nextOffset }) : null });
  }

  latestCurve(address, options = {}) {
    object(options, [], ['commitment'], 'latest curve options');
    const filters = normalizeFilters({ kind: 'curve-account', address,
      ...(has(options, 'commitment') ? { commitment: options.commitment } : {}) });
    return detached({ ...this.describe(), item: this.#matching(filters).at(-1) ?? null });
  }
}

export function createFixtureIndexer(options) { return new FixtureIndexer(options); }
