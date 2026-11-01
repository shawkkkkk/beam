// Local ABI serialization only. These base metas do not form executable trades.
import { PROGRAM_ID, decodeBase58Pubkey } from './beam-native.mjs';

const U64_MAX = (1n << 64n) - 1n;
const BUY_V2_DISC = Uint8Array.from([184, 23, 238, 97, 103, 197, 211, 61]);
const SELL_V2_DISC = Uint8Array.from([93, 246, 130, 60, 231, 233, 64, 178]);

// Anchor ToAccountMetas(None) order at Beam source commit 4e32d30.
const BUY_V2_ACCOUNTS = [
  ['global', false, false], ['baseMint', false, false], ['quoteMint', false, false],
  ['baseTokenProgram', false, false], ['quoteTokenProgram', false, false],
  ['associatedTokenProgram', false, false], ['feeRecipient', false, true],
  ['associatedQuoteFeeRecipient', false, true], ['buybackFeeRecipient', false, true],
  ['associatedQuoteBuybackFeeRecipient', false, true], ['bondingCurve', false, true],
  ['associatedBaseBondingCurve', false, true], ['associatedQuoteBondingCurve', false, true],
  ['user', true, true], ['associatedBaseUser', false, true],
  ['associatedQuoteUser', false, true], ['creatorVault', false, true],
  ['associatedCreatorVault', false, true], ['sharingConfig', false, false],
  ['globalVolumeAccumulator', false, false], ['userVolumeAccumulator', false, true],
  ['associatedUserVolumeAccumulator', false, true], ['feeConfig', false, false],
  ['feeProgram', false, false], ['systemProgram', false, false],
  ['eventAuthority', false, false], ['program', false, false],
];
const SELL_V2_ACCOUNTS = [
  ['global', false, false], ['baseMint', false, false], ['quoteMint', false, false],
  ['baseTokenProgram', false, false], ['quoteTokenProgram', false, false],
  ['associatedTokenProgram', false, false], ['feeRecipient', false, true],
  ['associatedQuoteFeeRecipient', false, true], ['buybackFeeRecipient', false, true],
  ['associatedQuoteBuybackFeeRecipient', false, true], ['bondingCurve', false, true],
  ['associatedBaseBondingCurve', false, true], ['associatedQuoteBondingCurve', false, true],
  ['user', true, true], ['associatedBaseUser', false, true],
  ['associatedQuoteUser', false, true], ['creatorVault', false, true],
  ['associatedCreatorVault', false, true], ['sharingConfig', false, false],
  ['userVolumeAccumulator', false, true], ['associatedUserVolumeAccumulator', false, true],
  ['feeConfig', false, false], ['feeProgram', false, false],
  ['systemProgram', false, false], ['eventAuthority', false, false],
  ['program', false, false],
];

function exactRecord(value, fields, label) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)
      || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    throw new TypeError(`${label} must be a plain record`);
  }
  const keys = Reflect.ownKeys(value);
  if (keys.length !== fields.length || keys.some((key) => !fields.includes(key))) {
    throw new TypeError(`${label} must contain exactly: ${fields.join(', ')}`);
  }
  return Object.fromEntries(fields.map((field) => {
    const descriptor = Object.getOwnPropertyDescriptor(value, field);
    if (!descriptor || !Object.hasOwn(descriptor, 'value')) {
      throw new TypeError(`${label}.${field} must be an own data field`);
    }
    return [field, descriptor.value];
  }));
}

function u64(value, name) {
  let amount;
  if (typeof value === 'bigint') amount = value;
  else if (typeof value === 'number' && Number.isSafeInteger(value)) amount = BigInt(value);
  else if (typeof value === 'string' && /^(0|[1-9][0-9]*)$/.test(value)) {
    if (value.length > 20) throw new RangeError(`${name} must fit u64`);
    amount = BigInt(value);
  } else throw new TypeError(`${name} must be a safe integer, bigint, or canonical decimal string`);
  if (amount < 0n || amount > U64_MAX) throw new RangeError(`${name} must fit u64`);
  const bytes = new Uint8Array(8);
  new DataView(bytes.buffer).setBigUint64(0, amount, true);
  return bytes;
}

function concat(parts) {
  const bytes = new Uint8Array(parts.reduce((length, part) => length + part.length, 0));
  let offset = 0;
  for (const part of parts) { bytes.set(part, offset); offset += part.length; }
  return bytes;
}

export function encodeBuyV2(input) {
  const args = exactRecord(input, ['amount', 'maxSolCost', 'partialFill'], 'BuyV2 args');
  if (typeof args.partialFill !== 'boolean') throw new TypeError('partialFill must be boolean');
  return concat([BUY_V2_DISC, u64(args.amount, 'amount'),
    u64(args.maxSolCost, 'maxSolCost'), Uint8Array.from([Number(args.partialFill)])]);
}

export function encodeSellV2(input) {
  const args = exactRecord(input, ['amount', 'minSolOutput'], 'SellV2 args');
  return concat([SELL_V2_DISC, u64(args.amount, 'amount'),
    u64(args.minSolOutput, 'minSolOutput')]);
}

function build(layout, accounts, data) {
  const values = exactRecord(accounts, layout.map(([name]) => name), 'accounts');
  const keys = layout.map(([name, isSigner, isWritable]) => {
    decodeBase58Pubkey(values[name]);
    return { pubkey: values[name], isSigner, isWritable };
  });
  if (values.program !== PROGRAM_ID) throw new RangeError('self-CPI program must be Beam');
  return { programId: PROGRAM_ID, keys, data };
}

export function buildBuyV2(accounts, args) {
  return build(BUY_V2_ACCOUNTS, accounts, encodeBuyV2(args));
}

export function buildSellV2(accounts, args) {
  return build(SELL_V2_ACCOUNTS, accounts, encodeSellV2(args));
}
