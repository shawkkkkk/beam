import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { PROGRAM_ID } from './beam-native.mjs';
import { encodeBuyV2, encodeSellV2, buildBuyV2, buildSellV2 } from './beam-v2-trades.mjs';

const fixture = JSON.parse(readFileSync(new URL('./fixtures/v2-trades.json', import.meta.url)));
const raw = readFileSync(new URL('./fixtures/v2-trades-anchor.txt', import.meta.url));
const receipt = JSON.parse(readFileSync(new URL('./fixtures/v2-trades-receipt.json', import.meta.url)));
const hex = (bytes) => Buffer.from(bytes).toString('hex');
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

function anchorResult(kind) {
  const lines = raw.toString('utf8').split('\n').filter((line) => line.startsWith(`${kind}.`));
  const metas = lines.slice(2).map((line, index) => {
    const [label, pubkey, signer, writable] = line.split(',');
    assert.equal(label, `${kind}.meta=${index}`);
    return { pubkey, isSigner: signer === 'true', isWritable: writable === 'true' };
  });
  return {
    programId: lines[0].split('=')[1],
    dataHex: lines[1].split('=')[1],
    metas,
  };
}

test('compiled Anchor receipt binds the golden data and every ordered meta', () => {
  assert.equal(fixture.sourceCommit, '4e32d302f5dfe7650030db9cbde0bcc5edf9aaaf');
  assert.equal(fixture.fixtureOnly, true);
  assert.equal(fixture.programId, PROGRAM_ID);
  assert.equal(sha256(raw), fixture.anchorStdoutSha256);
  assert.equal(sha256(raw), receipt.stdoutSha256);
  assert.equal(raw.length, receipt.stdoutBytes);
  assert.equal(fixture.sourceCommit, receipt.sourceCommit);
  assert.equal(receipt.exitCode, 0);
  assert.equal(receipt.runtimeHandlersExecuted, false);
  assert.equal(sha256(readFileSync(new URL('./tools/generate-v2-trade-fixtures.rs', import.meta.url))),
    fixture.producerSha256);
  assert.equal(fixture.producerSha256, receipt.producerSha256);
  for (const kind of ['buyV2', 'sellV2']) {
    const output = anchorResult(kind);
    assert.equal(output.programId, fixture.programId);
    assert.equal(output.dataHex, fixture[kind].dataHex);
    assert.deepEqual(output.metas, fixture[kind].metas);
    assert.deepEqual(Object.values(fixture[kind].accounts), output.metas.map((meta) => meta.pubkey));
  }
  assert.equal(fixture.buyV2.metas.length, 27);
  assert.equal(fixture.sellV2.metas.length, 26);
});

test('account names follow selected Anchor source structs in declaration order', () => {
  for (const [kind, path, rustName] of [
    ['buyV2', '../programs/beam/src/buy_v2.rs', 'BuyV2'],
    ['sellV2', '../programs/beam/src/sell_v2.rs', 'SellV2'],
  ]) {
    const source = readFileSync(new URL(path, import.meta.url), 'utf8');
    const match = source.match(new RegExp(`pub struct ${rustName}<'info>\\{([\\s\\S]*?)\\n\\}`));
    assert.ok(match, `${rustName} account struct missing`);
    const names = [...match[1].matchAll(/pub ([a-z_]+):/g)]
      .map((field) => field[1].replace(/_([a-z])/g, (_, letter) => letter.toUpperCase()));
    assert.deepEqual(names, Object.keys(fixture[kind].accounts));
  }
});

test('BuyV2 and SellV2 local bytes and privileges match compiled Anchor output', () => {
  for (const [kind, encode, build] of [
    ['buyV2', encodeBuyV2, buildBuyV2],
    ['sellV2', encodeSellV2, buildSellV2],
  ]) {
    const expected = fixture[kind];
    assert.equal(hex(encode(expected.args)), expected.dataHex);
    const instruction = build(expected.accounts, expected.args);
    assert.equal(instruction.programId, PROGRAM_ID);
    assert.equal(hex(instruction.data), expected.dataHex);
    assert.deepEqual(instruction.keys, expected.metas);
  }
  assert.equal(fixture.buyV2.metas[13].isSigner, true);
  assert.equal(fixture.sellV2.metas[13].isSigner, true);
  assert.equal(fixture.buyV2.metas[19].isWritable, false); // global volume accumulator
  assert.equal(fixture.sellV2.metas[19].isWritable, true); // user volume accumulator
});

test('trade amounts preserve u64 precision and reject unsafe or ambiguous values', () => {
  const buy = fixture.buyV2.args;
  const sell = fixture.sellV2.args;
  assert.equal(hex(encodeBuyV2({ ...buy, amount: (1n << 64n) - 1n })).slice(16, 32),
    'ffffffffffffffff');
  assert.equal(hex(encodeSellV2({ ...sell, minSolOutput: 0n })).slice(-16),
    '0000000000000000');
  for (const value of [1n << 64n, -1n, Number.MAX_SAFE_INTEGER + 1, '01', '1.5']) {
    assert.throws(() => encodeBuyV2({ ...buy, amount: value }));
  }
  assert.throws(() => encodeSellV2({ ...sell, minSolOutput: '9'.repeat(1000) }), /fit u64/);
  assert.throws(() => encodeBuyV2({ ...buy, partialFill: 1 }), /boolean/);
  assert.equal(hex(encodeBuyV2({ ...buy, partialFill: true })).slice(-2), '01');
});

test('builders reject extra or malformed accounts and substituted self-programs', () => {
  const buy = fixture.buyV2;
  const missing = { ...buy.accounts };
  delete missing.feeConfig;
  assert.throws(() => buildBuyV2(missing, buy.args), /exactly/);
  assert.throws(() => buildBuyV2({ ...buy.accounts, payoutWallet: buy.accounts.user }, buy.args), /exactly/);
  assert.throws(() => buildBuyV2({ ...buy.accounts, program: buy.accounts.user }, buy.args), /self-CPI/);
  assert.throws(() => buildSellV2({ ...fixture.sellV2.accounts, feeProgram: '0' }, fixture.sellV2.args),
    /base58/);
  assert.throws(() => encodeBuyV2({ ...buy.args, trackVolume: true }), /exactly/);
  assert.throws(() => encodeSellV2({ ...fixture.sellV2.args, partialFill: false }), /exactly/);
  let invoked = false;
  const accessor = { ...buy.accounts };
  Object.defineProperty(accessor, 'user', { enumerable: true, get() { invoked = true; return buy.accounts.user; } });
  assert.throws(() => buildBuyV2(accessor, buy.args), /own data field/);
  assert.equal(invoked, false);
});
