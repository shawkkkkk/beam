import { test } from "node:test";
import assert from "node:assert/strict";
import {
  AMM_V4_BUILD_CAPABILITY,
  presentAmmV4,
} from "../src/ammV4Presentation.ts";
import {
  NO_TRANSACTION_CAPABILITIES,
  sampleState,
  type CandidateState,
} from "../src/indexerAdapter.ts";

const mint = "test-mint";
const curveAccount = "test-curve";

function candidate(
  lifecycle: "on_curve" | "curve_complete" | "migrated",
  tokenProgram: "spl_token" | "token_2022" = "spl_token",
): CandidateState {
  return {
    kind: "unverified_indexer",
    page: {
      schemaVersion: 1,
      source: {
        chain: "solana",
        indexerId: "fixture",
        programId: "test-program",
        slot: "1",
        finality: "finalized",
        indexedAtMs: 1,
        cursor: null,
      },
      tokens: [
        {
          mint,
          tokenProgram,
          creator: "test-creator",
          curveAccount,
          quoteMint: null,
          name: "Fixture",
          symbol: "FIX",
          metadataUri: "",
          lifecycle,
          ammV4Pool: lifecycle === "migrated" ? "claimed-pool" : null,
        },
      ],
      accounts: [
        {
          address: curveAccount,
          ownerProgram: "test-program",
          mint,
          quoteMint: null,
          realTokenReserves: "0",
          realQuoteReserves: "0",
          complete: lifecycle !== "on_curve",
          migrated: lifecycle === "migrated",
        },
      ],
      events: [],
    },
    indexerTimestampAge: "recent",
    checkedAtMs: 1,
    capabilities: NO_TRANSACTION_CAPABILITIES,
  };
}

test("current build and sample data never assert graduation or liquidity", () => {
  assert.deepEqual(AMM_V4_BUILD_CAPABILITY, {
    migration: "disabled",
    verifiedLiquidity: false,
  });
  const view = presentAmmV4(sampleState([]), mint);
  assert.equal(view.phase, "unavailable");
  assert.equal(view.graduation, "unavailable");
  assert.equal(view.verifiedAmmV4Liquidity, false);
  assert.deepEqual(view.capabilities, NO_TRANSACTION_CAPABILITIES);
});

test("active and complete curves stay distinct from migration", () => {
  const active = presentAmmV4(candidate("on_curve"), mint);
  assert.equal(active.phase, "on_curve_claim");
  assert.equal(active.graduation, "disabled");

  const complete = presentAmmV4(candidate("curve_complete"), mint);
  assert.equal(complete.phase, "migration_pending_claim");
  assert.equal(complete.graduation, "disabled");
  assert.match(complete.label, /complete curve/);
  assert.match(complete.label, /Migration is pending and disabled/);
  assert.equal(complete.verifiedAmmV4Liquidity, false);
});

test("Token-2022 completion has an explicit unsupported graduation state", () => {
  const view = presentAmmV4(candidate("curve_complete", "token_2022"), mint);
  assert.equal(view.phase, "curve_complete_claim");
  assert.equal(view.graduation, "unsupported_token_2022");
  assert.match(view.label, /does not support Token-2022 graduation/);
});

test("forged finalized pool and capability claims never become verified", () => {
  const state = candidate("migrated") as CandidateState & {
    verified: boolean;
    canTrade: boolean;
    migrationEnabled: boolean;
    price: string;
  };
  state.verified = true;
  state.canTrade = true;
  state.migrationEnabled = true;
  state.price = "$999B";
  const view = presentAmmV4(state, mint);
  assert.equal(view.phase, "migration_claimed_unverified");
  assert.equal(view.verifiedAmmV4Liquidity, false);
  assert.deepEqual(view.capabilities, NO_TRANSACTION_CAPABILITIES);
  assert.equal("pool" in view, false);
  assert.equal("price" in view, false);
  assert.equal("liquidity" in view, false);
  assert.match(view.label, /not verified/);
});

test("missing and unsupported state cannot masquerade as authenticated data", () => {
  assert.equal(
    presentAmmV4(candidate("on_curve"), "absent").phase,
    "unavailable",
  );
  assert.equal(
    presentAmmV4({ kind: "verified" } as never, mint).phase,
    "unavailable",
  );
});
