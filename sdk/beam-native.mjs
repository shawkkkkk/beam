// Development-only ABI helpers for the retained native-SOL Create/Buy/Sell
// instructions in programs/beam/src/lib.rs at canonical commit 65c96b5.
// These encode account metas; they do not derive PDAs or authorize execution.

export const PROGRAM_ID = 'GdaQ7RHbGKGEL5Px66tQP78S67XcgxYAzFUwja71qSJP';
const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
const U64_MAX = (1n << 64n) - 1n;
const CREATE_DISC = Uint8Array.from([24, 30, 200, 40, 5, 28, 7, 119]);
const BUY_DISC = Uint8Array.from([102, 6, 61, 18, 1, 218, 235, 234]);
const SELL_DISC = Uint8Array.from([51, 230, 133, 164, 1, 127, 131, 173]);
const COMPLETE_EVENT_DISC = Uint8Array.from([95, 114, 97, 156, 212, 46, 152, 8]);
const CURVE_DISC = Uint8Array.from([23, 183, 248, 55, 96, 216, 172, 96]);

function concat(parts) {
  const out = new Uint8Array(parts.reduce((length, part) => length + part.length, 0));
  let offset = 0;
  for (const part of parts) { out.set(part, offset); offset += part.length; }
  return out;
}

function equal(a, b) {
  return a.length === b.length && a.every((byte, index) => byte === b[index]);
}

export function decodeBase58Pubkey(value) {
  if (typeof value !== 'string' || !value.length) throw new TypeError('pubkey must be a base58 string');
  if (value.length > 44) throw new RangeError('base58 pubkey is too long');
  let n = 0n;
  for (const char of value) {
    const digit = ALPHABET.indexOf(char);
    if (digit < 0) throw new RangeError('invalid base58 pubkey');
    n = n * 58n + BigInt(digit);
  }
  const leading = value.match(/^1*/)[0].length;
  const tail = [];
  while (n > 0n) { tail.push(Number(n & 255n)); n >>= 8n; }
  const result = Uint8Array.from([...Array(leading).fill(0), ...tail.reverse()]);
  if (result.length !== 32) throw new RangeError('pubkey must decode to 32 bytes');
  return result;
}

export function encodeBase58Pubkey(bytes) {
  if (!(bytes instanceof Uint8Array) || bytes.length !== 32) throw new TypeError('pubkey must be 32 bytes');
  let n = 0n;
  for (const byte of bytes) n = (n << 8n) | BigInt(byte);
  let result = '';
  while (n > 0n) { result = ALPHABET[Number(n % 58n)] + result; n /= 58n; }
  for (const byte of bytes) { if (byte !== 0) break; result = `1${result}`; }
  return result;
}

function u64(value, name) {
  let n;
  if (typeof value === 'bigint') n = value;
  else if (typeof value === 'number' && Number.isSafeInteger(value)) n = BigInt(value);
  else if (typeof value === 'string' && /^(0|[1-9][0-9]*)$/.test(value)) {
    if (value.length > 20) throw new RangeError(`${name} must fit u64`);
    n = BigInt(value);
  }
  else throw new TypeError(`${name} must be a safe integer, bigint, or canonical decimal string`);
  if (n < 0n || n > U64_MAX) throw new RangeError(`${name} must fit u64`);
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, n, true);
  return out;
}

function borshString(value, name) {
  if (typeof value !== 'string') throw new TypeError(`${name} must be a string`);
  const bytes = new TextEncoder().encode(value);
  if (bytes.length > 0xffff_ffff) throw new RangeError(`${name} is too long`);
  const len = new Uint8Array(4);
  new DataView(len.buffer).setUint32(0, bytes.length, true);
  return concat([len, bytes]);
}

function bool(value, name) {
  if (typeof value !== 'boolean') throw new TypeError(`${name} must be boolean`);
  return Uint8Array.from([value ? 1 : 0]);
}

export function encodeCreate({ name, symbol, uri, creator }) {
  return concat([
    CREATE_DISC, borshString(name, 'name'),
    borshString(symbol, 'symbol'), borshString(uri, 'uri'),
    decodeBase58Pubkey(creator),
  ]);
}

export function encodeBuy({ amount, maxSolCost, trackVolume, partialFill }) {
  return concat([
    BUY_DISC, u64(amount, 'amount'),
    u64(maxSolCost, 'maxSolCost'), bool(trackVolume, 'trackVolume'),
    bool(partialFill, 'partialFill'),
  ]);
}

export function encodeSell({ amount, minSolOutput }) {
  return concat([
    SELL_DISC, u64(amount, 'amount'),
    u64(minSolOutput, 'minSolOutput'),
  ]);
}

// Name, signer, writable, in Anchor's generated ToAccountMetas order.
const CREATE_ACCOUNTS = [
  ['mint', true, true], ['mintAuthority', false, false],
  ['bondingCurve', false, true], ['associatedBondingCurve', false, true],
  ['global', false, false], ['mplTokenMetadata', false, false],
  ['metadata', false, true], ['user', true, true],
  ['systemProgram', false, false], ['tokenProgram', false, false],
  ['associatedTokenProgram', false, false], ['rent', false, false],
  ['eventAuthority', false, false], ['program', false, false],
];
const BUY_ACCOUNTS = [
  ['global', false, false], ['feeRecipient', false, true],
  ['mint', false, false], ['bondingCurve', false, true],
  ['associatedBondingCurve', false, true], ['associatedUser', false, true],
  ['user', true, true], ['systemProgram', false, false],
  ['tokenProgram', false, false], ['creatorVault', false, true],
  ['eventAuthority', false, false], ['program', false, false],
  ['globalVolumeAccumulator', false, false], ['userVolumeAccumulator', false, true],
  ['feeConfig', false, false], ['feeProgram', false, false],
];
const SELL_ACCOUNTS = [
  ['global', false, false], ['feeRecipient', false, true],
  ['mint', false, false], ['bondingCurve', false, true],
  ['associatedBondingCurve', false, true], ['associatedUser', false, true],
  ['user', true, true], ['systemProgram', false, false],
  ['creatorVault', false, true], ['tokenProgram', false, false],
  ['eventAuthority', false, false], ['program', false, false],
  ['feeConfig', false, false], ['feeProgram', false, false],
];

function build(layout, accounts, data) {
  if (accounts === null || typeof accounts !== 'object' || Array.isArray(accounts)) {
    throw new TypeError('accounts must be an object');
  }
  const keys = layout.map(([name, isSigner, isWritable]) => {
    const pubkey = accounts[name];
    decodeBase58Pubkey(pubkey);
    return { pubkey, isSigner, isWritable };
  });
  if (accounts.program !== PROGRAM_ID) throw new RangeError('self-CPI program account must be the Beam program');
  return { programId: PROGRAM_ID, keys, data };
}

export function buildCreate(accounts, args) { return build(CREATE_ACCOUNTS, accounts, encodeCreate(args)); }
export function buildBuy(accounts, args) { return build(BUY_ACCOUNTS, accounts, encodeBuy(args)); }
export function buildSell(accounts, args) { return build(SELL_ACCOUNTS, accounts, encodeSell(args)); }

export function decodeCompleteEvent(input) {
  if (!(input instanceof Uint8Array)) throw new TypeError('event payload must be bytes');
  const data = input;
  if (data.length !== 144 || !equal(data.subarray(0, 8), COMPLETE_EVENT_DISC)) {
    throw new RangeError('expected a 144-byte CompleteEvent payload');
  }
  return {
    user: encodeBase58Pubkey(data.subarray(8, 40)),
    mint: encodeBase58Pubkey(data.subarray(40, 72)),
    bondingCurve: encodeBase58Pubkey(data.subarray(72, 104)),
    timestamp: new DataView(data.buffer, data.byteOffset, data.byteLength).getBigInt64(104, true),
    quoteMint: encodeBase58Pubkey(data.subarray(112, 144)),
  };
}

export function decodeCurrentBondingCurve(input) {
  if (!(input instanceof Uint8Array)) throw new TypeError('curve account must be bytes');
  const data = input;
  if (data.length !== 166 || !equal(data.subarray(0, 8), CURVE_DISC)) {
    throw new RangeError('expected a 166-byte current BondingCurve account');
  }
  const flag = (offset) => {
    if (data[offset] > 1) throw new RangeError('invalid Borsh boolean');
    return data[offset] === 1;
  };
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return {
    virtualTokenReserves: view.getBigUint64(8, true),
    virtualQuoteReserves: view.getBigUint64(16, true),
    realTokenReserves: view.getBigUint64(24, true),
    realQuoteReserves: view.getBigUint64(32, true),
    tokenTotalSupply: view.getBigUint64(40, true),
    complete: flag(48), creator: encodeBase58Pubkey(data.subarray(49, 81)),
    isMayhemMode: flag(81), isCashbackCoin: flag(82),
    quoteMint: encodeBase58Pubkey(data.subarray(83, 115)),
    creatorFeeBps: view.getBigUint64(115, true),
    canEditCreatorFee: flag(123), isHolderReward: flag(124),
    creatorFee: view.getBigUint64(125, true),
    protocolFees: view.getBigUint64(133, true), depth: data[141],
    initialVirtualQuoteReserves: view.getBigUint64(142, true),
    postCompleteBaseOut: view.getBigUint64(150, true),
    postCompleteQuoteIn: view.getBigUint64(158, true),
  };
}
