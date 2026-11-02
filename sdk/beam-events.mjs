// Local payload decoding only. A decoded record does not authenticate a chain event.
import { encodeBase58Pubkey } from './beam-native.mjs';

export const MAX_EVENT_BYTES = 1024 * 1024; // SDK resource policy, not an Anchor limit.
export const EVENT_DISCRIMINATORS = Object.freeze({
  CreateEvent: '1b72a94ddeeb6376',
  TradeEvent: 'bddb7fd34ee661ee',
});
const DISCRIMINATORS = Object.fromEntries(Object.entries(EVENT_DISCRIMINATORS).map(
  ([name, hex]) => [name, Uint8Array.from(hex.match(/../g), (byte) => Number.parseInt(byte, 16))],
));
const utf8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
const typedArrayTag = Object.getOwnPropertyDescriptor(typedArrayPrototype, Symbol.toStringTag).get;
const typedArrayByteLength = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'byteLength').get;

class Reader {
  constructor(bytes) {
    this.bytes = bytes;
    this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    this.offset = 8;
  }
  take(length, name) {
    if (!Number.isSafeInteger(length) || length < 0 || length > this.bytes.length - this.offset) {
      throw new RangeError(`truncated ${name}`);
    }
    const start = this.offset;
    this.offset += length;
    return this.bytes.subarray(start, this.offset);
  }
  number(length, method, name) {
    const start = this.offset;
    this.take(length, name);
    return this.view[method](start, true);
  }
  u64(name) { return this.number(8, 'getBigUint64', name); }
  i64(name) { return this.number(8, 'getBigInt64', name); }
  u32(name) { return this.number(4, 'getUint32', name); }
  u16(name) { return this.number(2, 'getUint16', name); }
  u8(name) { return this.take(1, name)[0]; }
  bool(name) {
    const value = this.u8(name);
    if (value > 1) throw new RangeError(`invalid Borsh boolean: ${name}`);
    return value === 1;
  }
  key(name) { return encodeBase58Pubkey(this.take(32, name)); }
  string(name) {
    const length = this.u32(`${name}.length`);
    const bytes = this.take(length, name);
    try { return utf8.decode(bytes); }
    catch { throw new RangeError(`invalid UTF-8: ${name}`); }
  }
  shareholders() {
    const count = this.u32('shareholders.length');
    // Each row is Pubkey32 + u16. The required fields after this vector
    // occupy another 88 bytes. Check both before allocating a row array.
    const available = this.bytes.length - this.offset - 88;
    if (available < 0 || count > Math.floor(available / 34)) {
      throw new RangeError('truncated shareholders or TradeEvent tail');
    }
    const rows = new Array(count);
    for (let index = 0; index < count; index += 1) {
      rows[index] = { address: this.key('shareholder.address'), shareBps: this.u16('shareholder.shareBps') };
    }
    return rows;
  }
  finish() {
    if (this.offset !== this.bytes.length) throw new RangeError('trailing event bytes');
  }
}

function readCreate(r) {
  return {
    name: r.string('name'), symbol: r.string('symbol'), uri: r.string('uri'),
    mint: r.key('mint'), bondingCurve: r.key('bondingCurve'), user: r.key('user'),
    creator: r.key('creator'), timestamp: r.i64('timestamp'),
    virtualTokenReserves: r.u64('virtualTokenReserves'), virtualSolReserves: r.u64('virtualSolReserves'),
    realTokenReserves: r.u64('realTokenReserves'), tokenTotalSupply: r.u64('tokenTotalSupply'),
    tokenProgram: r.key('tokenProgram'), isMayhemMode: r.bool('isMayhemMode'),
    isCashbackEnabled: r.bool('isCashbackEnabled'), quoteMint: r.key('quoteMint'),
    virtualQuoteReserves: r.u64('virtualQuoteReserves'), creatorFeeBps: r.u64('creatorFeeBps'),
    isHolderReward: r.bool('isHolderReward'), depth: r.u8('depth'),
  };
}

function readTrade(r) {
  return {
    mint: r.key('mint'), solAmount: r.u64('solAmount'), tokenAmount: r.u64('tokenAmount'),
    isBuy: r.bool('isBuy'), user: r.key('user'), timestamp: r.i64('timestamp'),
    virtualSolReserves: r.u64('virtualSolReserves'), virtualTokenReserves: r.u64('virtualTokenReserves'),
    realSolReserves: r.u64('realSolReserves'), realTokenReserves: r.u64('realTokenReserves'),
    feeRecipient: r.key('feeRecipient'), feeBasisPoints: r.u64('feeBasisPoints'), fee: r.u64('fee'),
    creator: r.key('creator'), creatorFeeBasisPoints: r.u64('creatorFeeBasisPoints'),
    creatorFee: r.u64('creatorFee'), trackVolume: r.bool('trackVolume'),
    totalUnclaimedTokens: r.u64('totalUnclaimedTokens'), totalClaimedTokens: r.u64('totalClaimedTokens'),
    currentSolVolume: r.u64('currentSolVolume'), lastUpdateTimestamp: r.i64('lastUpdateTimestamp'),
    ixName: r.string('ixName'), mayhemMode: r.bool('mayhemMode'),
    cashbackFeeBasisPoints: r.u64('cashbackFeeBasisPoints'), cashback: r.u64('cashback'),
    buybackFeeBasisPoints: r.u64('buybackFeeBasisPoints'), buybackFee: r.u64('buybackFee'),
    shareholders: r.shareholders(), quoteMint: r.key('quoteMint'), quoteAmount: r.u64('quoteAmount'),
    virtualQuoteReserves: r.u64('virtualQuoteReserves'), realQuoteReserves: r.u64('realQuoteReserves'),
    holderRewardsBps: r.u64('holderRewardsBps'), holderRewards: r.u64('holderRewards'),
    creatorFeeUnclaimed: r.u64('creatorFeeUnclaimed'), unparsedTrailingField: r.u64('unparsedTrailingField'),
  };
}

function decode(input, expectedName) {
  // Native getters establish the actual byte-view brand and length even if a
  // caller shadows properties or changes a view's prototype.
  if (typedArrayTag.call(input) !== 'Uint8Array') throw new TypeError('event payload must be a Uint8Array');
  const byteLength = typedArrayByteLength.call(input);
  if (byteLength < 8) throw new RangeError('truncated event discriminator');
  if (byteLength > MAX_EVENT_BYTES) throw new RangeError('event exceeds SDK payload limit');
  const bytes = new Uint8Array(input);
  const eventName = Object.keys(DISCRIMINATORS).find((name) =>
    DISCRIMINATORS[name].every((byte, index) => byte === bytes[index]));
  if (!eventName) throw new RangeError('unknown event discriminator');
  if (expectedName && eventName !== expectedName) throw new RangeError(`expected ${expectedName} discriminator`);
  const reader = new Reader(bytes);
  const data = eventName === 'CreateEvent' ? readCreate(reader) : readTrade(reader);
  reader.finish();
  return { eventName, data };
}

export function decodeCreateEvent(input) { return decode(input, 'CreateEvent').data; }
export function decodeTradeEvent(input) { return decode(input, 'TradeEvent').data; }
export function decodeBeamEvent(input) { return decode(input); }
