import { test } from "node:test";
import assert from "node:assert/strict";
import {
  DRAFTS_KEY,
  WATCHLIST_KEY,
  nextDraftId,
  orderNewFeed,
  parseDrafts,
  parseWatchlist,
  readDrafts,
  readWatchlist,
  toggleWatchlist,
  validateCreateV2Metadata,
  validateLocalDraftOptions,
  writeDrafts,
  writeWatchlist,
} from "../src/previewState.ts";

test("watchlist storage rejects malformed values and survives storage failures", () => {
  for (const value of ["{", "{}", "null", "[1]", '["ok",null]', '[""]']) {
    assert.deepEqual(parseWatchlist(value), [], value);
  }
  assert.deepEqual(parseWatchlist('["a","b","a"]'), ["a", "b"]);
  const throwing = {
    getItem(): string {
      throw new Error("denied");
    },
    setItem(): void {
      throw new Error("denied");
    },
  };
  assert.deepEqual(readWatchlist(throwing), []);
  assert.equal(writeWatchlist(throwing, ["a"]), false);
  const values = new Map<string, string>();
  const working = {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
  };
  assert.equal(writeWatchlist(working, ["a"]), true);
  assert.equal(values.get(WATCHLIST_KEY), '["a"]');
  assert.deepEqual(readWatchlist(working), ["a"]);
});

test("drafts and watchlist round-trip independently with guarded records", () => {
  const draft = {
    id: "draft-mho6di00-0",
    name: "Beam",
    symbol: "BEAM",
    description: "Local",
    metadataUri: "",
    createdAt: 1_700_000_000_000,
  };
  const values = new Map<string, string>();
  const storage = {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
  };
  assert.equal(writeDrafts(storage, [draft]), true);
  assert.equal(writeWatchlist(storage, [draft.id]), true);
  assert.deepEqual(readDrafts(storage), [draft]);
  assert.deepEqual(readWatchlist(storage), [draft.id]);
  assert.equal(values.has(DRAFTS_KEY), true);
  assert.deepEqual(
    parseDrafts(JSON.stringify([{ ...draft, id: "bad" }, draft, draft])),
    [draft],
  );
  const throwing = {
    getItem(): string {
      throw new Error("denied");
    },
    setItem(): void {
      throw new Error("denied");
    },
  };
  assert.deepEqual(readDrafts(throwing), []);
  assert.equal(writeDrafts(throwing, [draft]), false);
});

test("new drafts remain before sample tokens and repeated IDs stay unique", () => {
  const id = nextDraftId(new Set<string>(), 123);
  const second = nextDraftId(new Set([id]), 123);
  assert.notEqual(id, second);
  assert.deepEqual(
    orderNewFeed(["newest", "older"], ["sample-new", "sample-old"]),
    ["newest", "older", "sample-new", "sample-old"],
  );
  assert.deepEqual(toggleWatchlist(toggleWatchlist([], id), id), []);
});

test("CreateV2 metadata limits count UTF-8 bytes", () => {
  assert.equal(
    validateCreateV2Metadata("é".repeat(16), "BEAM", "u".repeat(200)),
    null,
  );
  assert.match(
    validateCreateV2Metadata("é".repeat(17), "BEAM", "") ?? "",
    /32 UTF-8 bytes/,
  );
  assert.match(
    validateCreateV2Metadata("Beam", "ABCDEFGHIJKLMN", "") ?? "",
    /13 UTF-8 bytes/,
  );
  assert.match(
    validateCreateV2Metadata("Beam", "BEAM", "é".repeat(101)) ?? "",
    /200 UTF-8 bytes/,
  );
});

test("local creator, charity, callout, and stream plans persist without activation", () => {
  const draft = {
    id: "draft-mho6di00-0",
    name: "Beam",
    symbol: "BEAM",
    description: "Local",
    metadataUri: "",
    createdAt: 1_700_000_000_000,
    creatorFeeWallet: "11111111111111111111111111111111",
    donationChoice: "donate.gg" as const,
    charityName: "Intended charity",
    callout: "Hello community",
    livestreamUrl: "https://example.org/stream",
  };
  assert.deepEqual(parseDrafts(JSON.stringify([draft])), [draft]);
  assert.equal(
    validateLocalDraftOptions(
      draft.creatorFeeWallet,
      draft.charityName,
      draft.callout,
      draft.livestreamUrl,
    ),
    null,
  );
  assert.match(
    validateLocalDraftOptions("bad address", "", "", "") ?? "",
    /Solana address/,
  );
  assert.match(
    validateLocalDraftOptions("", "", "", "javascript:alert(1)") ?? "",
    /HTTPS URL/,
  );
  assert.deepEqual(
    parseDrafts(JSON.stringify([{ ...draft, donationChoice: "other" }])),
    [],
  );
});
