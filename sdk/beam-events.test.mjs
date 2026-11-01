import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import {
  decodeBeamEvent, decodeCreateEvent, decodeTradeEvent, EVENT_DISCRIMINATORS, MAX_EVENT_BYTES,
} from './beam-events.mjs';

const fixture = JSON.parse(readFileSync(new URL('./fixtures/events.json', import.meta.url)));
const anchorRaw = readFileSync(new URL('./fixtures/events-anchor.json', import.meta.url));
const anchor = JSON.parse(anchorRaw);
const raw = (entry) => Buffer.from(entry.dataHex, 'hex');
const normalized = (value) => JSON.parse(JSON.stringify(value, (_, v) => typeof v === 'bigint' ? v.toString() : v));
const create = fixture.cases.find((entry) => entry.id === 'create-unicode');
const trade = fixture.cases.find((entry) => entry.id === 'trade-mixed');
const field = (entry, name) => entry.fields.find((item) => item.name === name);

test('actual Anchor payloads decode every source field with exact scalar types', () => {
  assert.equal(fixture.fixtureOnly, true);
  assert.equal(createHash('sha256').update(anchorRaw).digest('hex'), fixture.anchorRun.stdoutSha256);
  assert.deepEqual(anchor.events.map((entry) => entry.dataHex), fixture.cases.map((entry) => entry.dataHex));
  for (const entry of fixture.cases) {
    const result = decodeBeamEvent(raw(entry));
    assert.equal(result.eventName, entry.eventName);
    assert.deepEqual(normalized(result.data), entry.expected);
    assert.deepEqual(entry.eventName === 'CreateEvent' ? decodeCreateEvent(raw(entry)) : decodeTradeEvent(raw(entry)), result.data);
    for (const definition of entry.fields) {
      if (definition.type === 'u64' || definition.type === 'i64') assert.equal(typeof result.data[definition.name], 'bigint');
      if (definition.type === 'bool') assert.equal(typeof result.data[definition.name], 'boolean');
      if (definition.type === 'u8') assert.equal(typeof result.data[definition.name], 'number');
    }
  }
  assert.equal(decodeCreateEvent(raw(create)).symbol, '\ufeffB\0');
  assert.equal(decodeTradeEvent(raw(trade)).ixName, '\ufeffbuy\0🚀');
  assert.deepEqual(decodeTradeEvent(raw(trade)).shareholders.map((row) => row.shareBps), [0, 65535]);
});

test('selected source/layout witnesses and independent Rust bytes regenerate', () => {
  const process = spawnSync(globalThis.process.execPath, ['sdk/tools/generate-events-fixtures.mjs', '--check'], {
    cwd: new URL('../', import.meta.url), encoding: 'utf8',
  });
  assert.equal(process.status, 0, process.stderr || process.stdout);
  for (const name of ['CreateEvent', 'TradeEvent']) {
    assert.equal(createHash('sha256').update(`event:${name}`).digest('hex').slice(0, 16), EVENT_DISCRIMINATORS[name]);
  }
  const existing = JSON.parse(readFileSync(new URL('./fixtures/native-sol.json', import.meta.url)));
  assert.equal(createHash('sha256').update('event:CompleteEvent').digest('hex').slice(0, 16), existing.completeEvent.dataHex.slice(0, 16));
});

test('every truncation boundary and trailing extension is rejected', () => {
  for (const entry of fixture.cases) {
    const bytes = raw(entry);
    for (let length = 0; length < bytes.length; length += 1) {
      assert.throws(() => decodeBeamEvent(bytes.subarray(0, length)), RangeError, `${entry.id} length ${length}`);
    }
    assert.throws(() => decodeBeamEvent(Buffer.concat([bytes, Buffer.from([0])])), /trailing/);
  }
});

test('unknown, wrong-type and event-CPI envelope discriminators are rejected', () => {
  for (let offset = 0; offset < 8; offset += 1) {
    const bytes = raw(create); bytes[offset] ^= 255;
    assert.throws(() => decodeBeamEvent(bytes), /discriminator/);
  }
  assert.throws(() => decodeCreateEvent(raw(trade)), /expected CreateEvent/);
  assert.throws(() => decodeTradeEvent(raw(create)), /expected TradeEvent/);
  assert.throws(() => decodeBeamEvent(Buffer.concat([Buffer.from('e445a52e51cb9a1d', 'hex'), raw(create)])), /discriminator/);
  const existing = JSON.parse(readFileSync(new URL('./fixtures/native-sol.json', import.meta.url)));
  assert.throws(() => decodeBeamEvent(Buffer.from(existing.completeEvent.dataHex, 'hex')), /unknown/);
});

test('all six boolean fields accept only canonical Borsh bytes', () => {
  for (const entry of [create, trade]) {
    for (const definition of entry.fields.filter((item) => item.type === 'bool')) {
      for (let value = 0; value < 256; value += 1) {
        const bytes = raw(entry); bytes[definition.offset] = value;
        if (value < 2) assert.equal(decodeBeamEvent(bytes).data[definition.name], value === 1);
        else assert.throws(() => decodeBeamEvent(bytes), /Borsh boolean/);
      }
    }
  }
});

test('all u64 and i64 fields retain full precision and signedness', () => {
  for (const entry of [create, trade]) {
    for (const definition of entry.fields.filter((item) => ['u64', 'i64'].includes(item.type))) {
      const values = definition.type === 'u64'
        ? [0n, 9007199254740993n, 1n << 63n, (1n << 64n) - 1n]
        : [-(1n << 63n), -1n, 0n, (1n << 63n) - 1n];
      for (const value of values) {
        const bytes = raw(entry);
        bytes[definition.type === 'u64' ? 'writeBigUInt64LE' : 'writeBigInt64LE'](value, definition.offset);
        assert.equal(decodeBeamEvent(bytes).data[definition.name], value);
      }
    }
  }
});

test('fatal UTF-8 rejects invalid sequences in every nonempty string field', () => {
  for (const entry of [create, trade]) {
    for (const definition of entry.fields.filter((item) => item.type === 'String')) {
      for (const invalid of [[255], [192, 175], [237, 160, 128], [244, 144, 128, 128]]) {
        const bytes = raw(entry);
        bytes.set(invalid, definition.offset + 4);
        assert.throws(() => decodeBeamEvent(bytes), /UTF-8/);
      }
      const forged = raw(entry); forged.writeUInt32LE(0xffff_ffff, definition.offset);
      assert.throws(() => decodeBeamEvent(forged), /truncated/);
    }
  }
});

test('shareholder count and mandatory 88-byte tail are bounded before row allocation', () => {
  const offset = field(trade, 'shareholders').offset;
  for (const count of [3, 65535, 0xffff_ffff]) {
    const bytes = raw(trade); bytes.writeUInt32LE(count, offset);
    assert.throws(() => decodeTradeEvent(bytes), /truncated shareholders or TradeEvent tail/);
  }
  const empty = fixture.cases.find((entry) => entry.id === 'trade-empty');
  assert.equal(field(empty, 'unparsedTrailingField').offset + 8, raw(empty).length);
  assert.throws(() => decodeTradeEvent(raw(empty).subarray(0, -8)), /truncated/);
});

test('Buffer and nonzero-offset Uint8Array views use their exact range and preserve input', () => {
  for (const entry of fixture.cases) {
    const bytes = raw(entry);
    const storage = Buffer.alloc(bytes.length + 32, 0xcc); bytes.copy(storage, 13);
    const view = new Uint8Array(storage.buffer, storage.byteOffset + 13, bytes.length);
    const original = Buffer.from(storage);
    const result = decodeBeamEvent(view);
    assert.deepEqual(normalized(result.data), entry.expected);
    assert.deepEqual(storage, original);
    storage.fill(0);
    assert.deepEqual(normalized(result.data), entry.expected);
  }
});

test('raw byte input type and explicit SDK payload-size policy are enforced', () => {
  const forged = Object.create(Uint8Array.prototype, { length: { value: 8 }, byteLength: { value: 8 } });
  const wrongView = new Uint16Array(32); Object.setPrototypeOf(wrongView, Uint8Array.prototype);
  for (const input of [undefined, null, [], {}, 'base64', new ArrayBuffer(32), new DataView(new ArrayBuffer(32)),
    new Uint8ClampedArray(32), forged, wrongView]) {
    assert.throws(() => decodeBeamEvent(input), TypeError);
  }
  assert.throws(() => decodeBeamEvent(new Uint8Array(MAX_EVENT_BYTES + 1)), /SDK payload limit/);
  const oversized = new Uint8Array(MAX_EVENT_BYTES + 1);
  Object.defineProperty(oversized, 'byteLength', { value: 8 });
  assert.throws(() => decodeBeamEvent(oversized), /SDK payload limit/);
  const shadowed = raw(create);
  Object.defineProperty(shadowed, 'byteLength', { get() { throw new Error('caller getter'); } });
  assert.deepEqual(normalized(decodeCreateEvent(shadowed)), create.expected);
  const empty = fixture.cases.find((entry) => entry.id === 'create-empty');
  const bytes = raw(empty), offset = field(empty, 'uri').offset;
  const length = MAX_EVENT_BYTES - bytes.length;
  const prefix = Buffer.alloc(4); prefix.writeUInt32LE(length);
  const maximum = Buffer.concat([bytes.subarray(0, offset), prefix, Buffer.alloc(length, 65), bytes.subarray(offset + 4)]);
  assert.equal(maximum.length, MAX_EVENT_BYTES);
  assert.equal(decodeCreateEvent(maximum).uri.length, length);
});
