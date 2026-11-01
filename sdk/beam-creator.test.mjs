import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import {
  CREATOR_SOURCE_COMMIT, CREATOR_PROGRAM_ID, INHERITED_FEES_PROGRAM_ID,
  encodeCollectCreatorFee, buildCollectCreatorFee, decodeCollectCreatorFeeEvent,
  previewCollectCreatorFee,
} from './beam-creator.mjs';

const fixtureBytes = readFileSync(new URL('./fixtures/creator-native-sol.json', import.meta.url));
const fixture = JSON.parse(fixtureBytes);
const receipt = JSON.parse(readFileSync(new URL('./fixtures/creator-anchor-receipt.json', import.meta.url)));
const config = { sourceCommit: CREATOR_SOURCE_COMMIT, programId: CREATOR_PROGRAM_ID };
const accounts = fixture.collectCreatorFee.accounts;
const U64_MAX = (1n << 64n) - 1n;
const hex = (bytes) => Buffer.from(bytes).toString('hex');
const strings = (value) => Object.fromEntries(Object.entries(value).map(([key, item]) => [key, typeof item === 'bigint' ? item.toString() : item]));
const snapshot = (changes = {}) => ({
  creator: accounts.creator, creatorVault: accounts.creatorVault,
  creatorOwner: accounts.systemProgram, creatorExecutable: false,
  creatorLamports: 5n, creatorRentMinimum: 10n,
  vaultLamports: 105n, vaultZeroDataRentMinimum: 100n,
  observedSlot: 100n, referenceSlot: 105n, maxSlotAge: 5n,
  ...changes,
});

test('fixture and producer match the observed independent Anchor host receipt', () => {
  assert.equal(createHash('sha256').update(fixtureBytes).digest('hex'), receipt.stdoutSha256);
  const source = readFileSync(new URL('./tools/generate-creator-fixture.rs', import.meta.url));
  assert.equal(createHash('sha256').update(source).digest('hex'), receipt.sourceSha256);
  assert.equal(receipt.exitCode, 0);
  assert.equal(receipt.head, CREATOR_SOURCE_COMMIT);
  assert.equal(receipt.noHandlersOrNetworkOrTransactions, true);
  assert.equal(fixture.provenance.runtimeHandlerExecuted, false);
  assert.equal(fixture.provenance.kind, 'actual Anchor host serialization of synthetic values');
});

test('legacy no-argument bytes and all five metas match actual Anchor output', () => {
  assert.equal(fixture.programId, CREATOR_PROGRAM_ID);
  const instruction = buildCollectCreatorFee(config, accounts);
  assert.equal(instruction.programId, fixture.programId);
  assert.equal(hex(encodeCollectCreatorFee()), fixture.collectCreatorFee.dataHex);
  assert.equal(hex(instruction.data), fixture.collectCreatorFee.dataHex);
  assert.deepEqual(instruction.keys, fixture.collectCreatorFee.metas);
  assert.equal(instruction.keys.length, 5);
  assert.ok(instruction.keys.every((meta) => meta.isSigner === false));
  assert.deepEqual(instruction.keys.map((meta) => meta.isWritable), [true, true, false, false, false]);
  assert.equal(instruction.keys[0].pubkey, accounts.creator);
});

test('configuration is explicit and rejects stale source or substituted program', () => {
  for (const bad of [undefined, null, {}, { programId: CREATOR_PROGRAM_ID }]) {
    assert.throws(() => buildCollectCreatorFee(bad, accounts));
    assert.throws(() => previewCollectCreatorFee(bad, snapshot()));
  }
  assert.throws(() => buildCollectCreatorFee({ ...config, sourceCommit: 'old' }, accounts), /stale/);
  assert.throws(() => previewCollectCreatorFee({ ...config, sourceCommit: 'old' }, snapshot()), /stale/);
  assert.throws(() => buildCollectCreatorFee({ ...config, programId: accounts.creator }, accounts), /development identity/);
  assert.throws(() => buildCollectCreatorFee({ ...config, signingEnabled: true }, accounts), /exactly/);
});

test('missing, malformed and extra accounts fail closed without adding signers or payout fields', () => {
  for (const name of Object.keys(accounts)) {
    const missing = { ...accounts }; delete missing[name];
    assert.throws(() => buildCollectCreatorFee(config, missing), /exactly/);
    assert.throws(() => buildCollectCreatorFee(config, { ...accounts, [name]: '0' }), /base58/);
  }
  assert.throws(() => buildCollectCreatorFee(config, { ...accounts, systemProgram: accounts.creator }), /System program/);
  assert.throws(() => buildCollectCreatorFee(config, { ...accounts, program: accounts.creator }), /self-CPI/);
  assert.throws(() => buildCollectCreatorFee(config, { ...accounts, payoutWallet: accounts.creator }), /exactly/);
  assert.throws(() => buildCollectCreatorFee(config, { ...accounts, remainingAccounts: [] }), /exactly/);
  assert.throws(() => buildCollectCreatorFee(config, accounts, { amount: 1n }), /base accounts only/);
  assert.throws(() => encodeCollectCreatorFee({ amount: 1n }), /no instruction arguments/);
});

test('records require own data fields instead of inherited or accessor configuration', () => {
  let called = false;
  const accessor = { programId: CREATOR_PROGRAM_ID };
  Object.defineProperty(accessor, 'sourceCommit', { enumerable: true, get() { called = true; return CREATOR_SOURCE_COMMIT; } });
  assert.throws(() => buildCollectCreatorFee(accessor, accounts), /data field/);
  assert.equal(called, false);
  assert.throws(() => buildCollectCreatorFee(Object.create(config), accounts), /plain record/);
  assert.throws(() => buildCollectCreatorFee(config, []), /plain record/);
});

test('instruction results cannot mutate later encoded discriminator values', () => {
  const changed = buildCollectCreatorFee(config, accounts);
  changed.data.fill(0); changed.keys[0].isSigner = true;
  const next = buildCollectCreatorFee(config, accounts);
  assert.equal(hex(next.data), fixture.collectCreatorFee.dataHex);
  assert.equal(next.keys[0].isSigner, false);
});

test('strict event decoder agrees with actual Anchor i64/u64 boundary payloads', () => {
  for (const event of fixture.events) {
    const bytes = Buffer.from(event.dataHex, 'hex');
    assert.equal(bytes.length, 88);
    assert.deepEqual(strings(decodeCollectCreatorFeeEvent(bytes)), event.expected);
    const backing = new Uint8Array(93); backing.set(bytes, 3);
    assert.deepEqual(strings(decodeCollectCreatorFeeEvent(backing.subarray(3, 91))), event.expected);
  }
  assert.equal(decodeCollectCreatorFeeEvent(Buffer.from(fixture.events[1].dataHex, 'hex')).timestamp, -(1n << 63n));
  assert.equal(decodeCollectCreatorFeeEvent(Buffer.from(fixture.events[2].dataHex, 'hex')).creatorFee, U64_MAX);
});

test('event decoder rejects truncated, trailing, wrong-discriminator and enveloped inputs', () => {
  const bytes = Buffer.from(fixture.events[0].dataHex, 'hex');
  for (const size of [0, 7, 8, 16, 48, 56, 87]) assert.throws(() => decodeCollectCreatorFeeEvent(bytes.subarray(0, size)), /88-byte/);
  assert.throws(() => decodeCollectCreatorFeeEvent(Buffer.concat([bytes, Buffer.from([0])])), /88-byte/);
  const wrong = Buffer.from(bytes); wrong[0] ^= 1;
  assert.throws(() => decodeCollectCreatorFeeEvent(wrong), /CollectCreatorFeeEvent/);
  for (const input of [null, [...bytes], bytes.toString('base64'), `Program data: ${bytes.toString('base64')}`]) {
    assert.throws(() => decodeCollectCreatorFeeEvent(input), /raw event byte/);
  }
  assert.throws(() => decodeCollectCreatorFeeEvent(bytes, CREATOR_PROGRAM_ID), /one raw/);
});

test('shared event quote mint is decoded rather than falsely labeled a legacy receipt', () => {
  const decoded = decodeCollectCreatorFeeEvent(Buffer.from(fixture.events[2].dataHex, 'hex'));
  assert.notEqual(decoded.quoteMint, accounts.systemProgram);
  assert.equal(decoded.quoteMint, fixture.events[2].expected.quoteMint);
});

test('local preview preserves the destination and the exact recipient-rent equality boundary', () => {
  const result = previewCollectCreatorFee(config, snapshot());
  assert.equal(result.creator, accounts.creator);
  assert.equal(result.creatorVault, accounts.creatorVault);
  assert.equal(result.sourceBranch, 'transfer-attempt');
  assert.equal(result.wouldAttemptTransfer, true);
  assert.equal(result.transferLamports, 5n);
  assert.equal(result.creatorBalanceForRentCheck, 10n);
  assert.equal(result.scope, 'local-source-branch-preview');
  const below = previewCollectCreatorFee(config, snapshot({ creatorLamports: 4n }));
  assert.equal(below.sourceBranch, 'recipient-below-rent');
  assert.equal(below.transferLamports, 0n);
});

test('vault reserve uses saturating subtraction, including zero and maximum amounts', () => {
  for (const balance of [0n, 99n, 100n]) {
    const result = previewCollectCreatorFee(config, snapshot({ vaultLamports: balance }));
    assert.equal(result.sourceBranch, 'nothing-above-vault-rent');
    assert.equal(result.availableLamports, 0n);
    assert.equal(result.transferLamports, 0n);
  }
  const maximum = previewCollectCreatorFee(config, snapshot({ vaultLamports: U64_MAX, vaultZeroDataRentMinimum: 0n, creatorLamports: 0n }));
  assert.equal(maximum.transferLamports, U64_MAX);
});

test('sharing-owner rejection precedes executable no-op; other owners are not treated as signers', () => {
  const blocked = previewCollectCreatorFee(config, snapshot({ creatorOwner: INHERITED_FEES_PROGRAM_ID, creatorExecutable: true }));
  assert.equal(blocked.sourceBranch, 'sharing-config-owner-rejected');
  assert.equal(blocked.wouldAttemptTransfer, false);
  const executable = previewCollectCreatorFee(config, snapshot({ creatorExecutable: true }));
  assert.equal(executable.sourceBranch, 'creator-executable-noop');
  assert.equal(executable.transferLamports, 0n);
  assert.equal(previewCollectCreatorFee(config, snapshot({ creatorOwner: accounts.creatorVault })).sourceBranch, 'transfer-attempt');
});

test('recipient addition matches the source saturating rent gate and exposes overflow', () => {
  const result = previewCollectCreatorFee(config, snapshot({
    creatorLamports: U64_MAX, creatorRentMinimum: U64_MAX,
    vaultLamports: 1n, vaultZeroDataRentMinimum: 0n,
  }));
  assert.equal(result.creatorBalanceForRentCheck, U64_MAX);
  assert.equal(result.balanceSumExceedsU64, true);
  assert.equal(result.sourceBranch, 'transfer-attempt');
  assert.equal(result.transferLamports, 1n); // This does not assert actual CPI success.
});

test('preview rejects missing or non-bigint amounts, invalid flags and invented payout fields', () => {
  for (const key of Object.keys(snapshot())) {
    const missing = snapshot(); delete missing[key];
    assert.throws(() => previewCollectCreatorFee(config, missing), /exactly/);
  }
  for (const key of ['creatorLamports', 'creatorRentMinimum', 'vaultLamports', 'vaultZeroDataRentMinimum', 'observedSlot', 'referenceSlot', 'maxSlotAge']) {
    for (const value of [0, '0', Number.MAX_SAFE_INTEGER + 1]) assert.throws(() => previewCollectCreatorFee(config, snapshot({ [key]: value })), /bigint/);
    for (const value of [-1n, U64_MAX + 1n]) assert.throws(() => previewCollectCreatorFee(config, snapshot({ [key]: value })), /u64/);
  }
  assert.throws(() => previewCollectCreatorFee(config, snapshot({ creatorExecutable: 0 })), /boolean/);
  assert.throws(() => previewCollectCreatorFee(config, snapshot({ payoutWallet: accounts.creator })), /exactly/);
});

test('slot freshness is explicit, bounded and only relative to the supplied reference', () => {
  assert.equal(previewCollectCreatorFee(config, snapshot()).observedSlot, 100n);
  assert.throws(() => previewCollectCreatorFee(config, snapshot({ referenceSlot: 106n })), /stale/);
  assert.throws(() => previewCollectCreatorFee(config, snapshot({ referenceSlot: 99n })), /future/);
  const max = previewCollectCreatorFee(config, snapshot({ observedSlot: U64_MAX, referenceSlot: U64_MAX, maxSlotAge: 0n }));
  assert.equal(max.observedSlot, U64_MAX);
});

test('event length comes from real typed-array storage rather than shadow properties', () => {
  const event = Buffer.from(fixture.events[0].dataHex, 'hex');
  for (const size of [87, 89]) {
    const input = new Uint8Array(size);
    input.set(event.subarray(0, Math.min(size, event.length)));
    Object.defineProperties(input, { length: { value: 88 }, byteLength: { value: 88 } });
    assert.throws(() => decodeCollectCreatorFeeEvent(input), /88-byte/);
  }
});

test('exact raw Uint8Array and Buffer storage ignores custom getters and iterators', () => {
  const expected = fixture.events[0].expected;
  const event = Buffer.from(fixture.events[0].dataHex, 'hex');
  for (const input of [Uint8Array.from(event), Buffer.from(event)]) {
    let touched = false;
    Object.defineProperties(input, {
      length: { get() { touched = true; throw new Error('shadow length must not run'); } },
      byteLength: { get() { touched = true; throw new Error('shadow byteLength must not run'); } },
      [Symbol.iterator]: { value: function* () { touched = true; yield* Buffer.from(fixture.events[2].dataHex, 'hex'); } },
    });
    assert.deepEqual(strings(decodeCollectCreatorFeeEvent(input)), expected);
    assert.equal(touched, false);
  }
});

test('fake byte-view prototypes and non-byte views reject before using supplied iteration', () => {
  const event = Buffer.from(fixture.events[0].dataHex, 'hex');
  const fake = Object.create(Uint8Array.prototype);
  const wide = Object.setPrototypeOf(new Uint16Array(44), Uint8Array.prototype);
  const dataView = Object.setPrototypeOf(new DataView(new ArrayBuffer(88)), Uint8Array.prototype);
  for (const input of [fake, wide, dataView]) {
    let iterated = false;
    Object.defineProperties(input, {
      length: { value: 88 }, byteLength: { value: 88 },
      [Symbol.iterator]: { value: function* () { iterated = true; yield* event; } },
    });
    assert.throws(() => decodeCollectCreatorFeeEvent(input), TypeError);
    assert.equal(iterated, false);
  }
  assert.throws(() => decodeCollectCreatorFeeEvent(new Proxy(Uint8Array.from(event), {})), TypeError);
});
