// Local-only helpers for the frozen legacy creator-fee source documented in CREATOR.md.
import { createHash } from 'node:crypto';
import { decodeBase58Pubkey, encodeBase58Pubkey } from './beam-native.mjs';

export const CREATOR_SOURCE_COMMIT = 'b8120a8a58b1375ba3b8f0d2a590c49923e18021';
export const CREATOR_PROGRAM_ID = 'GdaQ7RHbGKGEL5Px66tQP78S67XcgxYAzFUwja71qSJP';
export const INHERITED_FEES_PROGRAM_ID = 'pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ';
const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111';
const U64_MAX = (1n << 64n) - 1n;
const discriminator = (name) => Uint8Array.from(createHash('sha256').update(name).digest().subarray(0, 8));
const COLLECT_DISC = discriminator('global:collect_creator_fee');
const EVENT_DISC = discriminator('event:CollectCreatorFeeEvent');
const TYPED_ARRAY = Object.getPrototypeOf(Uint8Array.prototype);
const RAW_KIND = Object.getOwnPropertyDescriptor(TYPED_ARRAY, Symbol.toStringTag).get;
const RAW_BYTE_LENGTH = Object.getOwnPropertyDescriptor(TYPED_ARRAY, 'byteLength').get;
const ACCOUNT_FIELDS = ['creator', 'creatorVault', 'systemProgram', 'eventAuthority', 'program'];
const SNAPSHOT_FIELDS = [
  'creator', 'creatorVault', 'creatorOwner', 'creatorExecutable',
  'creatorLamports', 'creatorRentMinimum', 'vaultLamports', 'vaultZeroDataRentMinimum',
  'observedSlot', 'referenceSlot', 'maxSlotAge',
];

function record(value, fields, name) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)
      || ![Object.prototype, null].includes(Object.getPrototypeOf(value))) {
    throw new TypeError(`${name} must be a plain record`);
  }
  const keys = Reflect.ownKeys(value);
  if (keys.length !== fields.length || keys.some((key) => !fields.includes(key))) {
    throw new TypeError(`${name} must contain exactly: ${fields.join(', ')}`);
  }
  return Object.fromEntries(fields.map((key) => {
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    if (!descriptor || !Object.hasOwn(descriptor, 'value')) {
      throw new TypeError(`${name}.${key} must be an own data field`);
    }
    return [key, descriptor.value];
  }));
}

function pubkey(value) {
  decodeBase58Pubkey(value);
  return value;
}

function configuration(value) {
  const config = record(value, ['sourceCommit', 'programId'], 'config');
  if (config.sourceCommit !== CREATOR_SOURCE_COMMIT) {
    throw new RangeError('unsupported or stale creator sourceCommit');
  }
  if (pubkey(config.programId) !== CREATOR_PROGRAM_ID) {
    throw new RangeError('programId must match the frozen undeployed Beam development identity');
  }
  return config;
}

function u64(value, name) {
  if (typeof value !== 'bigint') throw new TypeError(`${name} must be bigint`);
  if (value < 0n || value > U64_MAX) throw new RangeError(`${name} must fit u64`);
  return value;
}

export function encodeCollectCreatorFee() {
  if (arguments.length !== 0) throw new TypeError('collect_creator_fee has no instruction arguments');
  return Uint8Array.from(COLLECT_DISC);
}

export function buildCollectCreatorFee(config, accounts) {
  if (arguments.length !== 2) throw new TypeError('expected config and five base accounts only');
  const accepted = configuration(config);
  const values = record(accounts, ACCOUNT_FIELDS, 'accounts');
  for (const name of ACCOUNT_FIELDS) pubkey(values[name]);
  if (values.systemProgram !== SYSTEM_PROGRAM_ID) throw new RangeError('wrong System program');
  if (values.program !== accepted.programId) throw new RangeError('wrong self-CPI program');
  return {
    programId: accepted.programId,
    keys: ACCOUNT_FIELDS.map((name, index) => ({
      pubkey: values[name], isSigner: false, isWritable: index < 2,
    })),
    data: encodeCollectCreatorFee(),
  };
}

export function decodeCollectCreatorFeeEvent(input) {
  // Read internal TypedArray slots, never caller-owned length/tag properties.
  if (arguments.length !== 1 || RAW_KIND.call(input) !== 'Uint8Array') {
    throw new TypeError('expected one raw event byte payload');
  }
  if (RAW_BYTE_LENGTH.call(input) !== 88) {
    throw new RangeError('expected an exact 88-byte CollectCreatorFeeEvent payload');
  }
  // TypedArray construction copies internal storage; .from() could call a
  // caller-controlled iterator and silently decode different bytes.
  const data = new Uint8Array(input);
  if (RAW_BYTE_LENGTH.call(data) !== 88 || !EVENT_DISC.every((byte, index) => data[index] === byte)) {
    throw new RangeError('expected an exact 88-byte CollectCreatorFeeEvent payload');
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return {
    timestamp: view.getBigInt64(8, true),
    creator: encodeBase58Pubkey(data.subarray(16, 48)),
    creatorFee: view.getBigUint64(48, true),
    quoteMint: encodeBase58Pubkey(data.subarray(56, 88)),
  };
}

export function previewCollectCreatorFee(config, snapshot) {
  if (arguments.length !== 2) throw new TypeError('expected config and one local snapshot');
  configuration(config);
  const state = record(snapshot, SNAPSHOT_FIELDS, 'snapshot');
  for (const key of ['creator', 'creatorVault', 'creatorOwner']) pubkey(state[key]);
  if (typeof state.creatorExecutable !== 'boolean') throw new TypeError('creatorExecutable must be boolean');
  for (const key of SNAPSHOT_FIELDS.slice(4)) u64(state[key], key);
  if (state.observedSlot > state.referenceSlot) throw new RangeError('snapshot observedSlot is in the future');
  if (state.referenceSlot - state.observedSlot > state.maxSlotAge) throw new RangeError('snapshot is stale');

  const availableLamports = state.vaultLamports > state.vaultZeroDataRentMinimum
    ? state.vaultLamports - state.vaultZeroDataRentMinimum : 0n;
  const rawSum = state.creatorLamports + availableLamports;
  const balanceSumExceedsU64 = rawSum > U64_MAX;
  const creatorBalanceForRentCheck = balanceSumExceedsU64 ? U64_MAX : rawSum;
  let sourceBranch;
  if (state.creatorOwner === INHERITED_FEES_PROGRAM_ID) sourceBranch = 'sharing-config-owner-rejected';
  else if (state.creatorExecutable) sourceBranch = 'creator-executable-noop';
  else if (availableLamports === 0n) sourceBranch = 'nothing-above-vault-rent';
  else if (creatorBalanceForRentCheck < state.creatorRentMinimum) sourceBranch = 'recipient-below-rent';
  else sourceBranch = 'transfer-attempt';
  const wouldAttemptTransfer = sourceBranch === 'transfer-attempt';
  return {
    scope: 'local-source-branch-preview',
    creator: state.creator, creatorVault: state.creatorVault,
    sourceBranch, wouldAttemptTransfer,
    transferLamports: wouldAttemptTransfer ? availableLamports : 0n,
    availableLamports, creatorBalanceForRentCheck, balanceSumExceedsU64,
    observedSlot: state.observedSlot, referenceSlot: state.referenceSlot, maxSlotAge: state.maxSlotAge,
  };
}
