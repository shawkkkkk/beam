import { test } from "node:test";
import assert from "node:assert/strict";
import { localFeaturePlans } from "../src/localFeaturePresentation.ts";

test("empty local feature choices create no status section", () => {
  assert.deepEqual(localFeaturePlans({}), []);
  assert.deepEqual(
    localFeaturePlans({ charityName: "Unselected", donationChoice: "none" }),
    [],
  );
});

test("each optional plan remains unavailable with its own missing dependency", () => {
  const cases = [
    [
      { creatorFeeWallet: "11111111111111111111111111111111" },
      "creator_wallet",
      /on chain/,
    ],
    [
      { donationChoice: "donate.gg" as const, charityName: "Synthetic choice" },
      "charity",
      /provider/,
    ],
    [{ callout: "Hello" }, "callout", /moderation/],
    [
      { livestreamUrl: "https://example.invalid/stream" },
      "livestream",
      /playback/,
    ],
  ] as const;
  for (const [input, kind, reason] of cases) {
    const [plan] = localFeaturePlans(input);
    assert.equal(plan.kind, kind);
    assert.equal(plan.source, "local_draft");
    assert.equal(plan.availability, "unavailable");
    assert.match(plan.reason, reason);
    assert.equal("action" in plan, false);
    assert.equal("href" in plan, false);
  }
});

test("combined plans preserve stable order and immutable text-only values", () => {
  const markup = '<img src=x onerror="window.__beamXss=1">';
  const stream = "https://example.invalid/stream";
  const plans = localFeaturePlans({
    creatorFeeWallet: "11111111111111111111111111111111",
    donationChoice: "donate.gg",
    charityName: markup,
    callout: markup,
    livestreamUrl: stream,
  });
  assert.deepEqual(
    plans.map((plan) => plan.kind),
    ["creator_wallet", "charity", "callout", "livestream"],
  );
  assert.equal(plans[1].value, markup);
  assert.equal(plans[2].value, markup);
  assert.equal(plans[3].value, stream);
  assert.equal(Object.isFrozen(plans), true);
  assert.equal(plans.every(Object.isFrozen), true);
});
