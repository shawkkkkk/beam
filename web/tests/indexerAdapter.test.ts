import { test } from "node:test";
import assert from "node:assert/strict";
import {
  NO_TRANSACTION_CAPABILITIES,
  canDisplayAsVerified,
  dataLabel,
  freshnessAt,
  parseIndexerPage,
  sampleState,
} from "../src/indexerAdapter.ts";

// Public keys copied from the repository's fixture-only native-SOL ABI page.
const programId = "GdaQ7RHbGKGEL5Px66tQP78S67XcgxYAzFUwja71qSJP";
const mint = "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi";
const creator = "US517G5965aydkZ46HS38QLi7UQiSojurfbQfKCELFx";
const curve = "CktRuQ2mttgRGkXJtyksdKHjUdc2C4TgDzyB98oEzy8";
const pool = "GgBaCs3NCBuZN12kCJgAW63ydqohFkHEdfdEXBPzLHq";
const otherProgram = "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR";
const signature = "1".repeat(64);
const nowMs = 1_800_000_000_000;
const options = { expectedProgramId: programId, nowMs, maxAgeMs: 30_000 };

function fixture() {
  return {
    schemaVersion: 1,
    source: {
      chain: "solana",
      indexerId: "indexer-test",
      programId,
      slot: "9007199254740993",
      finality: "confirmed",
      indexedAtMs: nowMs - 1_000,
      cursor: "page-1",
    },
    tokens: [
      {
        mint,
        tokenProgram: "spl_token",
        creator,
        curveAccount: curve,
        name: "Beam test",
        symbol: "BEAM",
        metadataUri: "https://example.invalid/token.json",
        lifecycle: "on_curve",
        ammV4Pool: null,
        marketCap: "$999B",
        sendTransaction: true,
      },
    ],
    accounts: [
      {
        address: curve,
        ownerProgram: programId,
        mint,
        realTokenReserves: "18446744073709551615",
        realSolReserves: "0",
        complete: false,
        migrated: false,
        wallet: "fabricated",
      },
    ],
    events: [
      {
        signature,
        eventIndex: 0,
        slot: "9007199254740992",
        mint,
        kind: "create",
        volume: "$999B",
      },
    ],
    canTrade: true,
  };
}

test("sample fixtures carry no chain provenance or transaction capability", () => {
  const state = sampleState([{ name: "Illustrative" }]);
  assert.equal(state.kind, "sample");
  assert.equal(state.source, null);
  assert.match(
    dataLabel(state, nowMs, 30_000),
    /Sample data — no chain source/,
  );
  assert.equal(canDisplayAsVerified(state, nowMs, 30_000), false);
  assert.deepEqual(state.capabilities, NO_TRANSACTION_CAPABILITIES);
  assert.equal(Object.values(state.capabilities).some(Boolean), false);
});

test("valid API shape stays unverified and strips fabricated metrics/actions", () => {
  const result = parseIndexerPage(fixture(), options);
  assert.equal(result.kind, "unverified_indexer");
  assert.equal(result.freshness, "fresh");
  assert.equal(
    freshnessAt(result.page.source, nowMs + 31_000, 30_000),
    "stale",
  );
  assert.equal(result.page.source.slot, "9007199254740993");
  assert.equal(result.page.source.finality, "confirmed");
  assert.equal(result.page.events[0].id, `${signature}:0`);
  assert.equal(canDisplayAsVerified(result, nowMs, 30_000), false);
  assert.match(
    dataLabel(result, nowMs, 30_000),
    /Unverified indexer data — fresh, confirmed slot/,
  );
  assert.equal("marketCap" in result.page.tokens[0], false);
  assert.equal("sendTransaction" in result.page.tokens[0], false);
  assert.equal("volume" in result.page.events[0], false);
  assert.equal("wallet" in result.page.accounts[0], false);
  assert.equal("canTrade" in result, false);
  assert.equal(Object.values(result.capabilities).some(Boolean), false);
  assert.equal(Object.isFrozen(result.page.source), true);
  assert.equal(Object.isFrozen(result.page.tokens[0]), true);
});

test("finalized data is still only an indexer candidate; freshness is explicit", () => {
  const page = fixture();
  page.source.finality = "finalized";
  page.source.indexedAtMs = nowMs - 90_000;
  const result = parseIndexerPage(page, options);
  assert.equal(result.freshness, "stale");
  assert.equal(canDisplayAsVerified(result, nowMs, 30_000), false);
  assert.match(dataLabel(result, nowMs, 30_000), /stale, finalized/);
});

test("migration requires distinct completed and migrated state with a pool", () => {
  const page = fixture();
  page.tokens[0].lifecycle = "curve_complete";
  page.accounts[0].complete = true;
  assert.equal(
    parseIndexerPage(page, options).page.tokens[0].lifecycle,
    "curve_complete",
  );
  page.tokens[0].lifecycle = "migrated";
  assert.throws(() => parseIndexerPage(page, options), /lifecycle/);
  page.tokens[0].ammV4Pool = pool;
  page.accounts[0].migrated = true;
  assert.equal(
    parseIndexerPage(page, options).page.tokens[0].lifecycle,
    "migrated",
  );
  page.tokens[0].tokenProgram = "token_2022";
  assert.throws(() => parseIndexerPage(page, options), /lifecycle/);
});

test("rejects wrong program, account owner, duplicate events, overflow and future slots", () => {
  const wrongProgram = fixture();
  wrongProgram.source.programId = otherProgram;
  assert.throws(() => parseIndexerPage(wrongProgram, options), /program ID/);
  const wrongOwner = fixture();
  wrongOwner.accounts[0].ownerProgram = otherProgram;
  assert.throws(() => parseIndexerPage(wrongOwner, options), /provenance/);
  const duplicate = fixture();
  duplicate.events.push({ ...duplicate.events[0] });
  assert.throws(() => parseIndexerPage(duplicate, options), /Duplicate event/);
  const overflow = fixture();
  overflow.accounts[0].realSolReserves = "18446744073709551616";
  assert.throws(() => parseIndexerPage(overflow, options), /canonical u64/);
  const future = fixture();
  future.events[0].slot = "9007199254740994";
  assert.throws(() => parseIndexerPage(future, options), /outside the page/);
});

test("rejects malformed metadata, base58, oversized pages and future timestamps", () => {
  const badMint = fixture();
  badMint.tokens[0].mint = "not-base58";
  assert.throws(() => parseIndexerPage(badMint, options), /base58/);
  const longName = fixture();
  longName.tokens[0].name = "é".repeat(17);
  assert.throws(() => parseIndexerPage(longName, options), /bounded string/);
  const manyTokens = fixture();
  manyTokens.tokens = Array(101).fill(manyTokens.tokens[0]);
  assert.throws(() => parseIndexerPage(manyTokens, options), /at most 100/);
  const futureTime = fixture();
  futureTime.source.indexedAtMs = nowMs + 60_001;
  assert.throws(() => parseIndexerPage(futureTime, options), /timestamp/);
});
