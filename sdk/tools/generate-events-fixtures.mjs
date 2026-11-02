// Source-layout witness and independent comparison with actual Rust/Anchor bytes.
// Local fixture files only; production decoder is deliberately not imported.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { encodeBase58Pubkey } from '../beam-native.mjs';

const root = new URL('../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, root));
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const camel = (name) => name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
const key = (value) => encodeBase58Pubkey(new Uint8Array(32).fill(value));
const source = read('programs/beam/src/lib.rs').toString('utf8');
const reexport = read('programs/beam/src/trade_types.rs').toString('utf8').match(
  /pub\s+use\s+crate::pump_fees::\{\s*types::\{FeeTier,\s*Fees,\s*Shareholder\},\s*\};/,
);
assert.ok(reexport, 'Shareholder IDL reexport changed');
assert.match(source, /declare_program!\(pump_fees\);/);
assert.match(source, /use crate::trade_types::Shareholder;/);
const lock = read('programs/beam/Cargo.lock').toString('utf8');
for (const [name, version] of [['anchor-lang', '0.31.1'], ['anchor-attribute-event', '0.31.2'],
  ['anchor-derive-serde', '0.31.2'], ['borsh', '0.10.4']]) {
  assert.ok(lock.includes(`name = "${name}"\nversion = "${version}"`), `locked dependency changed: ${name}`);
}
const idl = JSON.parse(read('programs/beam/idls/pump_fees.json'));
const shareholder = idl.types.find((type) => type.name === 'Shareholder');
assert.deepEqual(shareholder.type, { kind: 'struct', fields: [
  { name: 'address', type: 'pubkey' }, { name: 'share_bps', type: 'u16' },
] });

function definition(name) {
  const pattern = new RegExp(`#\\[event\\]\\s*#\\[derive\\(Clone\\)\\]\\s*pub struct ${name}\\s*\\{([^}]*)\\}`);
  const match = source.match(pattern);
  assert.ok(match, `default #[event] definition missing: ${name}`);
  const fieldPattern = /pub\s+(\w+)\s*:\s*(Vec<Shareholder>|Pubkey|String|u64|i64|bool|u8)\s*,?/g;
  const fields = [...match[1].matchAll(fieldPattern)].map((field) => ({
    rustName: field[1], name: camel(field[1]), type: field[2],
  }));
  assert.equal(match[1].replace(fieldPattern, '').replace(/[\s,]/g, ''), '', 'unsupported field syntax');
  return { definitionSha256: sha256(match[0]), fields };
}
const definitions = { CreateEvent: definition('CreateEvent'), TradeEvent: definition('TradeEvent') };
assert.equal(definitions.CreateEvent.fields.length, 20);
assert.equal(definitions.TradeEvent.fields.length, 36);

const U64_MAX = (1n << 64n) - 1n;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const emptyCreate = {
  name: '', symbol: '', uri: '', mint: key(0), bondingCurve: key(0), user: key(0), creator: key(0),
  timestamp: I64_MAX, virtualTokenReserves: 0n, virtualSolReserves: 0n, realTokenReserves: 0n,
  tokenTotalSupply: 0n, tokenProgram: key(255), isMayhemMode: false, isCashbackEnabled: true,
  quoteMint: key(0), virtualQuoteReserves: 0n, creatorFeeBps: 0n, isHolderReward: false, depth: 0,
};
const emptyTrade = {
  mint: key(0), solAmount: 0n, tokenAmount: 0n, isBuy: false, user: key(0), timestamp: 0n,
  virtualSolReserves: 0n, virtualTokenReserves: 0n, realSolReserves: 0n, realTokenReserves: 0n,
  feeRecipient: key(0), feeBasisPoints: 0n, fee: 0n, creator: key(0), creatorFeeBasisPoints: 0n,
  creatorFee: 0n, trackVolume: true, totalUnclaimedTokens: 0n, totalClaimedTokens: 0n,
  currentSolVolume: 0n, lastUpdateTimestamp: 0n, ixName: '', mayhemMode: false,
  cashbackFeeBasisPoints: 0n, cashback: 0n, buybackFeeBasisPoints: 0n, buybackFee: 0n,
  shareholders: [], quoteMint: key(0), quoteAmount: 0n, virtualQuoteReserves: 0n,
  realQuoteReserves: 0n, holderRewardsBps: 0n, holderRewards: 0n,
  creatorFeeUnclaimed: 0n, unparsedTrailingField: 0n,
};
const cases = [
  { id: 'create-unicode', eventName: 'CreateEvent', expected: { ...emptyCreate,
    name: 'Beam 🚀', symbol: '\ufeffB\0', uri: 'https://example.invalid/🛸.json',
    mint: key(1), bondingCurve: key(2), user: key(3), creator: key(4), timestamp: I64_MIN,
    virtualTokenReserves: U64_MAX, virtualSolReserves: 9007199254740993n,
    realTokenReserves: 1n, tokenProgram: key(5), isMayhemMode: true, isCashbackEnabled: false,
    quoteMint: key(6), virtualQuoteReserves: 42n, creatorFeeBps: 65535n, isHolderReward: true, depth: 255,
  } },
  { id: 'create-empty', eventName: 'CreateEvent', expected: emptyCreate },
  { id: 'trade-mixed', eventName: 'TradeEvent', expected: { ...emptyTrade,
    mint: key(7), solAmount: U64_MAX, tokenAmount: 9007199254740995n, isBuy: true,
    user: key(8), timestamp: I64_MIN, virtualTokenReserves: 1n, realSolReserves: 2n,
    realTokenReserves: 3n, feeRecipient: key(9), feeBasisPoints: 50n, fee: 17n,
    creator: key(10), creatorFeeBasisPoints: 30n, creatorFee: 19n, trackVolume: false,
    totalUnclaimedTokens: U64_MAX, totalClaimedTokens: 9n, currentSolVolume: 9007199254740993n,
    lastUpdateTimestamp: I64_MAX, ixName: '\ufeffbuy\0🚀', mayhemMode: true,
    cashbackFeeBasisPoints: 11n, cashback: 12n, buybackFeeBasisPoints: 13n, buybackFee: 14n,
    shareholders: [{ address: key(11), shareBps: 0 }, { address: key(12), shareBps: 65535 }],
    quoteMint: key(13), quoteAmount: U64_MAX, virtualQuoteReserves: 15n,
    realQuoteReserves: 16n, holderRewardsBps: 17n, holderRewards: 18n,
    creatorFeeUnclaimed: 19n, unparsedTrailingField: U64_MAX,
  } },
  { id: 'trade-empty', eventName: 'TradeEvent', expected: emptyTrade },
  { id: 'trade-sequential', eventName: 'TradeEvent', expected: { ...emptyTrade,
    mint: key(21), user: key(22), feeRecipient: key(23), creator: key(24), quoteMint: key(25),
    solAmount: 1n, tokenAmount: 2n, timestamp: -3n, virtualSolReserves: 4n,
    virtualTokenReserves: 5n, realSolReserves: 6n, realTokenReserves: 7n,
    feeBasisPoints: 8n, fee: 9n, creatorFeeBasisPoints: 10n, creatorFee: 11n,
    totalUnclaimedTokens: 12n, totalClaimedTokens: 13n, currentSolVolume: 14n,
    lastUpdateTimestamp: -15n, ixName: 'sell', cashbackFeeBasisPoints: 16n,
    cashback: 17n, buybackFeeBasisPoints: 18n, buybackFee: 19n, quoteAmount: 20n,
    virtualQuoteReserves: 21n, realQuoteReserves: 22n, holderRewardsBps: 23n,
    holderRewards: 24n, creatorFeeUnclaimed: 25n, unparsedTrailingField: 26n,
    shareholders: [{ address: key(200), shareBps: 0 }, { address: key(201), shareBps: 1 },
      { address: key(202), shareBps: 65535 }],
  } },
];

// Independent local writer for the source fields, used only to validate the
// actual Rust bytes and document offsets. It never imports beam-events.mjs.
function number(value, width, signed = false) {
  const buffer = Buffer.alloc(width);
  if (width === 8) buffer[signed ? 'writeBigInt64LE' : 'writeBigUInt64LE'](value);
  else if (width === 4) buffer.writeUInt32LE(value);
  else if (width === 2) buffer.writeUInt16LE(value);
  else buffer[0] = value;
  return buffer;
}
function keyBytes(value) {
  // Synthetic keys are repeated bytes; use an independent finite lookup.
  for (let byte = 0; byte <= 255; byte += 1) if (key(byte) === value) return Buffer.alloc(32, byte);
  throw new Error('fixture key is not a synthetic repeated-byte value');
}
function encode(type, value) {
  if (type === 'Pubkey') return keyBytes(value);
  if (type === 'String') {
    const bytes = Buffer.from(value, 'utf8');
    return Buffer.concat([number(bytes.length, 4), bytes]);
  }
  if (type === 'Vec<Shareholder>') return Buffer.concat([number(value.length, 4),
    ...value.flatMap((row) => [keyBytes(row.address), number(row.shareBps, 2)])]);
  if (type === 'bool') return number(Number(value), 1);
  return number(value, type === 'u8' ? 1 : 8, type === 'i64');
}

const anchorRaw = read('sdk/fixtures/events-anchor.json');
const anchor = JSON.parse(anchorRaw);
const receipt = JSON.parse(read('sdk/fixtures/events-anchor.receipt.json'));
assert.equal(receipt.exitCode, 0);
assert.equal(receipt.head, 'b8120a8a58b1375ba3b8f0d2a590c49923e18021');
assert.equal(sha256(anchorRaw), receipt.stdoutSha256);
assert.equal(sha256(read('sdk/tools/generate-event-fixtures.rs')), receipt.sourceSha256);
assert.deepEqual(anchor.events.map((entry) => entry.id), cases.map((entry) => entry.id));

for (const entry of cases) {
  const discriminator = createHash('sha256').update(`event:${entry.eventName}`).digest().subarray(0, 8);
  const parts = [discriminator];
  let offset = 8;
  entry.fields = definitions[entry.eventName].fields.map((field) => {
    const bytes = encode(field.type, entry.expected[field.name]);
    const witness = { ...field, offset, byteLength: bytes.length };
    offset += bytes.length; parts.push(bytes);
    return witness;
  });
  entry.dataHex = anchor.events.find((actual) => actual.id === entry.id).dataHex;
  assert.equal(Buffer.concat(parts).toString('hex'), entry.dataHex, `actual Rust/source writer mismatch: ${entry.id}`);
}
const fixture = {
  sourceCommit: receipt.head, fixtureOnly: true,
  provenance: 'Synthetic values serialized by actual locked beam-program Anchor Event::data, independently compared with a source-layout writer; no handlers or chain data.',
  anchorRun: { ...receipt, stdoutFile: 'events-anchor.json', receiptFile: 'events-anchor.receipt.json' },
  sourceWitnesses: {
    eventFile: { path: 'programs/beam/src/lib.rs', baselineSha256: 'a410881812cd75e5f5b242ceb49faf8a73bc3ed01db1e76fbe1d989f084f5443', definitions },
    shareholderIdl: { path: 'programs/beam/idls/pump_fees.json', baselineSha256: '6b9fa3eccc6e06273076a47b581f9fcb9e531bb7c3c88acbcd44db55a4002b29', definition: shareholder },
    reexport: { path: 'programs/beam/src/trade_types.rs', selectedSha256: sha256(reexport[0]) },
    defaultEventDiscriminator: 'SHA-256(UTF8("event:" + exact Rust struct name))[0..8]',
    lockedDependencies: { 'anchor-lang': '0.31.1', 'anchor-attribute-event': '0.31.2', 'anchor-derive-serde': '0.31.2', borsh: '0.10.4' },
    crateChecksums: { 'anchor-attribute-event@0.31.2': '7fb2e11a313bd117800be14f7ad786c03a3f89faf3159cdc2e5326bcc6b5ea04', 'borsh@0.10.4': '115e54d64eb62cdebad391c19efc9dce4981c690c85a33a12199d99bb9546fee' },
  },
  cases,
};
const output = `${JSON.stringify(fixture, (_, value) => typeof value === 'bigint' ? value.toString() : value, 2)}\n`;
const destination = new URL('../fixtures/events.json', import.meta.url);
if (process.argv.includes('--check')) assert.equal(readFileSync(destination, 'utf8'), output, 'event fixture/source witness drift');
else writeFileSync(destination, output);
console.log(`Validated ${cases.length} actual Anchor payloads and selected source witnesses: ${fileURLToPath(destination)}`);
