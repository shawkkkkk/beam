/**
 * Read-only boundary for a future Beam indexer. This module performs no I/O.
 * Valid JSON from an API is still an unverified candidate: a trusted indexer
 * service and chain/replay checks must establish provenance before display as
 * verified chain data. Sample fixtures never acquire a slot or finality.
 */

export const NO_TRANSACTION_CAPABILITIES = Object.freeze({
  canConnectWallet: false,
  canSign: false,
  canSubmit: false,
  canTrade: false,
  canDonate: false,
  canStream: false,
} as const);

export type Finality = "processed" | "confirmed" | "finalized";
export type Freshness = "fresh" | "stale";
export type UInt64Decimal = string;
export type TokenLifecycle = "on_curve" | "curve_complete" | "migrated";

export type SourceProvenance = Readonly<{
  chain: "solana";
  indexerId: string;
  programId: string;
  slot: UInt64Decimal;
  finality: Finality;
  indexedAtMs: number;
  cursor: string | null;
}>;

export type IndexedToken = Readonly<{
  mint: string;
  tokenProgram: "spl_token" | "token_2022";
  creator: string;
  curveAccount: string;
  name: string;
  symbol: string;
  metadataUri: string;
  lifecycle: TokenLifecycle;
  ammV4Pool: string | null;
}>;

export type IndexedCurveAccount = Readonly<{
  address: string;
  ownerProgram: string;
  mint: string;
  realTokenReserves: UInt64Decimal;
  realSolReserves: UInt64Decimal;
  complete: boolean;
  migrated: boolean;
}>;

export type IndexedEvent = Readonly<{
  id: string;
  signature: string;
  eventIndex: number;
  slot: UInt64Decimal;
  mint: string;
  kind: "create" | "buy" | "sell" | "curve_complete" | "migrate";
}>;

export type IndexedPage = Readonly<{
  schemaVersion: 1;
  source: SourceProvenance;
  tokens: readonly IndexedToken[];
  accounts: readonly IndexedCurveAccount[];
  events: readonly IndexedEvent[];
}>;

export type SampleState<T> = Readonly<{
  kind: "sample";
  records: readonly T[];
  source: null;
  capabilities: typeof NO_TRANSACTION_CAPABILITIES;
}>;

export type CandidateState = Readonly<{
  kind: "unverified_indexer";
  page: IndexedPage;
  freshness: Freshness;
  checkedAtMs: number;
  capabilities: typeof NO_TRANSACTION_CAPABILITIES;
}>;

// Reserved for a separately reviewed trusted backend. This parser cannot
// construct this state or promote untrusted JSON into it.
declare const verifiedByTrustedService: unique symbol;
export type VerifiedState = Readonly<{
  kind: "verified";
  page: IndexedPage;
  freshness: Freshness;
  checkedAtMs: number;
  capabilities: typeof NO_TRANSACTION_CAPABILITIES;
  [verifiedByTrustedService]: true;
}>;

export type ReadOnlyState<T> =
  | SampleState<T>
  | CandidateState
  | VerifiedState
  | Readonly<{
      kind: "unavailable";
      reason: string;
      capabilities: typeof NO_TRANSACTION_CAPABILITIES;
    }>;

export function sampleState<T>(records: readonly T[]): SampleState<T> {
  return Object.freeze({
    kind: "sample" as const,
    records: Object.freeze([...records]),
    source: null,
    capabilities: NO_TRANSACTION_CAPABILITIES,
  });
}

/** Recompute freshness when rendering; a previously fresh page can age out. */
export function freshnessAt(
  source: Pick<SourceProvenance, "indexedAtMs">,
  nowMs: number,
  maxAgeMs: number,
): Freshness {
  if (
    !Number.isSafeInteger(nowMs) ||
    nowMs < 0 ||
    !Number.isSafeInteger(maxAgeMs) ||
    maxAgeMs <= 0
  ) {
    throw new TypeError("Invalid freshness policy");
  }
  const age = nowMs - source.indexedAtMs;
  return age >= 0 && age <= maxAgeMs ? "fresh" : "stale";
}

export function dataLabel<T>(
  state: ReadOnlyState<T>,
  nowMs: number,
  maxAgeMs: number,
): string {
  switch (state.kind) {
    case "sample":
      return "Sample data — no chain source";
    case "unverified_indexer":
      return `Unverified indexer data — ${freshnessAt(state.page.source, nowMs, maxAgeMs)}, ${state.page.source.finality} slot ${state.page.source.slot}`;
    case "verified":
      return `Verified chain data — ${freshnessAt(state.page.source, nowMs, maxAgeMs)}, ${state.page.source.finality} slot ${state.page.source.slot}`;
    case "unavailable":
      return "Chain data unavailable";
  }
}

export function canDisplayAsVerified<T>(
  state: ReadOnlyState<T>,
  nowMs: number,
  maxAgeMs: number,
): state is VerifiedState {
  return (
    state.kind === "verified" &&
    freshnessAt(state.page.source, nowMs, maxAgeMs) === "fresh" &&
    state.page.source.finality === "finalized"
  );
}

const U64_MAX = 18_446_744_073_709_551_615n;
const encoder = new TextEncoder();
const base58Alphabet =
  "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

function record(value: unknown, field: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function boundedText(
  value: unknown,
  field: string,
  maxBytes: number,
  allowEmpty = false,
): string {
  if (
    typeof value !== "string" ||
    (!allowEmpty && value.length === 0) ||
    encoder.encode(value).length > maxBytes
  ) {
    throw new TypeError(`${field} must be a bounded string`);
  }
  return value;
}

function safeTime(value: unknown, field: string, nowMs: number): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < 0 ||
    value > nowMs + 60_000
  ) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }
  return value;
}

function u64(value: unknown, field: string): UInt64Decimal {
  if (
    typeof value !== "string" ||
    !/^(0|[1-9][0-9]*)$/.test(value) ||
    value.length > 20 ||
    BigInt(value) > U64_MAX
  ) {
    throw new TypeError(`${field} must be a canonical u64 decimal string`);
  }
  return value;
}

function decodedBase58Length(value: string): number {
  let number = 0n;
  for (const char of value) {
    const digit = base58Alphabet.indexOf(char);
    if (digit < 0) return -1;
    number = number * 58n + BigInt(digit);
  }
  let bytes = 0;
  while (number > 0n) {
    number >>= 8n;
    bytes++;
  }
  return (value.match(/^1*/)?.[0].length ?? 0) + bytes;
}

function base58(value: unknown, field: string, bytes: 32 | 64): string {
  if (
    typeof value !== "string" ||
    value.length < bytes ||
    value.length > (bytes === 32 ? 44 : 88) ||
    decodedBase58Length(value) !== bytes
  ) {
    throw new TypeError(`${field} must encode ${bytes} bytes in base58`);
  }
  return value;
}

function boundedArray(value: unknown, field: string, max: number): unknown[] {
  if (!Array.isArray(value) || value.length > max)
    throw new TypeError(`${field} must be an array of at most ${max}`);
  return value;
}

function boolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean")
    throw new TypeError(`${field} must be boolean`);
  return value;
}

/** Decode and sanitize a versioned API payload without fetching or trusting it. */
export function parseIndexerPage(
  input: unknown,
  options: Readonly<{
    expectedProgramId: string;
    nowMs: number;
    maxAgeMs: number;
  }>,
): CandidateState {
  if (
    !Number.isSafeInteger(options.nowMs) ||
    options.nowMs < 0 ||
    !Number.isSafeInteger(options.maxAgeMs) ||
    options.maxAgeMs <= 0
  ) {
    throw new TypeError("Invalid freshness policy");
  }
  const root = record(input, "page");
  if (root.schemaVersion !== 1)
    throw new TypeError("Unsupported indexer schema version");
  const rawSource = record(root.source, "source");
  if (rawSource.chain !== "solana") throw new TypeError("Unexpected chain");
  const programId = base58(rawSource.programId, "source.programId", 32);
  if (programId !== options.expectedProgramId)
    throw new TypeError("Unexpected Beam program ID");
  const source: SourceProvenance = {
    chain: "solana",
    indexerId: boundedText(rawSource.indexerId, "source.indexerId", 128),
    programId,
    slot: u64(rawSource.slot, "source.slot"),
    finality:
      rawSource.finality === "processed" ||
      rawSource.finality === "confirmed" ||
      rawSource.finality === "finalized"
        ? rawSource.finality
        : (() => {
            throw new TypeError("Invalid finality");
          })(),
    indexedAtMs: safeTime(
      rawSource.indexedAtMs,
      "source.indexedAtMs",
      options.nowMs,
    ),
    cursor:
      rawSource.cursor === null
        ? null
        : boundedText(rawSource.cursor, "source.cursor", 256),
  };

  const accounts = boundedArray(root.accounts, "accounts", 100).map(
    (value, index): IndexedCurveAccount => {
      const raw = record(value, `accounts[${index}]`);
      const account: IndexedCurveAccount = {
        address: base58(raw.address, "account.address", 32),
        ownerProgram: base58(raw.ownerProgram, "account.ownerProgram", 32),
        mint: base58(raw.mint, "account.mint", 32),
        realTokenReserves: u64(
          raw.realTokenReserves,
          "account.realTokenReserves",
        ),
        realSolReserves: u64(raw.realSolReserves, "account.realSolReserves"),
        complete: boolean(raw.complete, "account.complete"),
        migrated: boolean(raw.migrated, "account.migrated"),
      };
      if (
        account.ownerProgram !== programId ||
        (account.migrated && !account.complete)
      )
        throw new TypeError("Invalid curve account provenance or lifecycle");
      return account;
    },
  );
  const accountByAddress = new Map(
    accounts.map((account) => [account.address, account]),
  );
  if (accountByAddress.size !== accounts.length)
    throw new TypeError("Duplicate curve account");

  const tokens = boundedArray(root.tokens, "tokens", 100).map(
    (value, index): IndexedToken => {
      const raw = record(value, `tokens[${index}]`);
      const lifecycle = raw.lifecycle;
      if (
        lifecycle !== "on_curve" &&
        lifecycle !== "curve_complete" &&
        lifecycle !== "migrated"
      )
        throw new TypeError("Invalid token lifecycle");
      if (
        raw.tokenProgram !== "spl_token" &&
        raw.tokenProgram !== "token_2022"
      ) {
        throw new TypeError("Invalid token program");
      }
      const token: IndexedToken = {
        mint: base58(raw.mint, "token.mint", 32),
        tokenProgram: raw.tokenProgram,
        creator: base58(raw.creator, "token.creator", 32),
        curveAccount: base58(raw.curveAccount, "token.curveAccount", 32),
        name: boundedText(raw.name, "token.name", 32),
        symbol: boundedText(raw.symbol, "token.symbol", 13),
        metadataUri: boundedText(
          raw.metadataUri,
          "token.metadataUri",
          200,
          true,
        ),
        lifecycle,
        ammV4Pool:
          raw.ammV4Pool === null
            ? null
            : base58(raw.ammV4Pool, "token.ammV4Pool", 32),
      };
      const account = accountByAddress.get(token.curveAccount);
      if (
        !account ||
        account.mint !== token.mint ||
        account.complete !== (lifecycle !== "on_curve") ||
        account.migrated !== (lifecycle === "migrated") ||
        (token.ammV4Pool !== null) !== (lifecycle === "migrated") ||
        (lifecycle === "migrated" && token.tokenProgram !== "spl_token")
      ) {
        throw new TypeError(
          "Token and curve account disagree on lifecycle or mint",
        );
      }
      return token;
    },
  );
  const tokenByMint = new Map(tokens.map((token) => [token.mint, token]));
  if (tokenByMint.size !== tokens.length)
    throw new TypeError("Duplicate token mint");

  const events = boundedArray(root.events, "events", 200).map(
    (value, index): IndexedEvent => {
      const raw = record(value, `events[${index}]`);
      const kind = raw.kind;
      if (
        kind !== "create" &&
        kind !== "buy" &&
        kind !== "sell" &&
        kind !== "curve_complete" &&
        kind !== "migrate"
      )
        throw new TypeError("Invalid event kind");
      const signature = base58(raw.signature, "event.signature", 64);
      const eventIndex = raw.eventIndex;
      if (
        typeof eventIndex !== "number" ||
        !Number.isSafeInteger(eventIndex) ||
        eventIndex < 0 ||
        eventIndex > 10_000
      )
        throw new TypeError("Invalid event index");
      const slot = u64(raw.slot, "event.slot");
      const mint = base58(raw.mint, "event.mint", 32);
      if (BigInt(slot) > BigInt(source.slot) || !tokenByMint.has(mint))
        throw new TypeError("Event is outside the page source or token set");
      return {
        id: `${signature}:${eventIndex}`,
        signature,
        eventIndex,
        slot,
        mint,
        kind,
      };
    },
  );
  if (new Set(events.map((event) => event.id)).size !== events.length)
    throw new TypeError("Duplicate event");

  const page: IndexedPage = Object.freeze({
    schemaVersion: 1,
    source: Object.freeze(source),
    tokens: Object.freeze(tokens.map((token) => Object.freeze(token))),
    accounts: Object.freeze(accounts.map((account) => Object.freeze(account))),
    events: Object.freeze(events.map((event) => Object.freeze(event))),
  });
  return Object.freeze({
    kind: "unverified_indexer",
    page,
    freshness: freshnessAt(source, options.nowMs, options.maxAgeMs),
    checkedAtMs: options.nowMs,
    capabilities: NO_TRANSACTION_CAPABILITIES,
  });
}
