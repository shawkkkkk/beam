import test from "node:test";
import assert from "node:assert/strict";
import { CalloutError, createCalloutService } from "./service.mjs";
import { createMockAdapters, MOCK_CHANNELS, MOCK_REQUEST } from "./mock-adapters.mjs";

const channelId = MOCK_CHANNELS[0];
const author = "mock-author-a";
const moderator = "mock-moderator-a";
const viewer = "mock-viewer";

function environment(options = {}) {
  let now = 1000;
  const clock = () => now;
  const mocks = createMockAdapters(clock);
  const service = createCalloutService({
    auth: mocks.auth, moderation: mocks.moderation, clock,
    channels: [...MOCK_CHANNELS], ...options,
  });
  return { service, mocks, clock, setTime(value) { now = value; } };
}

function submit(service, key = "submit-1", extra = {}) {
  return service.submit({ ...MOCK_REQUEST, session: author, idempotencyKey: key, ...extra });
}

async function claimed(service, key = "submit-1", extra = {}) {
  const pending = await submit(service, key, extra);
  return service.claimReview({ session: moderator, channelId, id: pending.id, expectedRevision: pending.revision, idempotencyKey: `claim-${key}` });
}

async function approved(service, key = "submit-1", extra = {}) {
  const record = await claimed(service, key, extra);
  return service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: record.revision, decision: "approve", idempotencyKey: `approve-${key}` });
}

function rejectsCode(operation, code) {
  return assert.rejects(operation, (error) => error instanceof CalloutError && error.code === code);
}

test("complete scheduled display workflow uses current time and never extends on retries", async () => {
  const env = environment();
  const pending = await submit(env.service);
  assert.equal(pending.state, "pending");
  assert.equal(pending.displayState, "awaiting-review");
  assert.deepEqual(pending.provenance, { source: "synthetic-local", verifiedOnChain: false, externalActionsEnabled: false });
  assert.equal((await env.service.queryDisplay({ session: viewer, channelId })).state, "empty");
  const claim = await env.service.claimReview({ session: moderator, channelId, id: pending.id, expectedRevision: 1, idempotencyKey: "claim-1" });
  const reviewRequest = { session: moderator, channelId, id: claim.id, expectedRevision: 2, decision: "approve", startsAt: 2000, idempotencyKey: "approve-1" };
  const result = await env.service.moderate(reviewRequest);
  assert.equal(result.state, "approved");
  assert.equal(result.displayState, "scheduled");
  assert.equal(result.endsAt, 12000);
  env.setTime(1999);
  assert.equal((await env.service.queryDisplay({ session: viewer, channelId })).items.length, 0);
  env.setTime(2000);
  const visible = await env.service.queryDisplay({ session: viewer, channelId });
  assert.equal(visible.items.length, 1);
  assert.equal(visible.items[0].displayState, "displaying");
  assert.deepEqual(visible.items[0].accessibility, { role: "status", ariaLive: "polite", label: "Approved callout displaying", contentType: "text/plain" });
  assert.equal("reviewOwnerId" in visible.items[0], false);
  assert.equal("authorId" in visible.items[0], false);
  assert.equal("reason" in visible.items[0], false);
  env.setTime(12000);
  assert.equal((await env.service.queryDisplay({ session: viewer, channelId })).state, "empty");
  assert.equal((await env.service.getOwn({ session: author, channelId, id: pending.id })).state, "expired");
  assert.equal((await submit(env.service)).id, pending.id);
  assert.equal((await env.service.moderate(reviewRequest)).displayState, "expired");
  assert.equal((await env.service.moderate(reviewRequest)).endsAt, 12000);
});

test("rejection remains private, terminal and visible only to its author", async () => {
  const { service, setTime } = environment();
  const record = await claimed(service);
  await service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: 2, decision: "reject", reason: "off-topic", idempotencyKey: "reject-1" });
  setTime(1_000_000);
  const own = await service.getOwn({ session: author, channelId, id: record.id });
  assert.equal(own.state, "rejected");
  assert.equal(own.reason, "off-topic");
  assert.equal("reviewOwnerId" in own, false);
  assert.equal((await service.queryDisplay({ session: viewer, channelId })).items.length, 0);
  await rejectsCode(service.getOwn({ session: "mock-author-b", channelId, id: record.id }), "NOT_FOUND");
  await rejectsCode(service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: 3, decision: "approve", idempotencyKey: "revise-1" }), "TERMINAL_STATE");
});

test("permissions are channel scoped and clients cannot assign authors or approval", async () => {
  const { service, mocks } = environment();
  await rejectsCode(submit(service, "unauth", { session: "unknown-session" }), "UNAUTHENTICATED");
  await rejectsCode(submit(service, "viewer", { session: viewer }), "FORBIDDEN");
  await rejectsCode(submit(service, "other-channel", { channelId: MOCK_CHANNELS[1] }), "FORBIDDEN");
  await rejectsCode(submit(service, "forged", { authorId: "moderator-a" }), "INVALID_REQUEST");
  await rejectsCode(submit(service, "forged", { approved: true }), "INVALID_REQUEST");
  const own = await submit(service, "moderator-owned", { session: moderator });
  await rejectsCode(service.claimReview({ session: moderator, channelId, id: own.id, expectedRevision: 1, idempotencyKey: "self" }), "SELF_REVIEW");
  assert.equal((await service.listReviewQueue({ session: moderator, channelId })).items.length, 0);
  mocks.setSession(author, { subject: "author-a", expiresAt: 5000, grants: [{ channelId, actions: ["display"] }] });
  await rejectsCode(submit(service, "revoked"), "FORBIDDEN");
});

test("concurrent moderation claims have one owner and revisions prevent stale decisions", async () => {
  const { service } = environment();
  const record = await submit(service);
  const claims = await Promise.allSettled([moderator, "mock-moderator-b"].map((session) => service.claimReview({ session, channelId, id: record.id, expectedRevision: 1, idempotencyKey: "claim" })));
  assert.equal(claims.filter((item) => item.status === "fulfilled").length, 1);
  assert.equal(claims[1].reason.code, "REVIEW_OWNED");
  const queue = await service.listReviewQueue({ session: moderator, channelId });
  assert.equal(queue.items[0].reviewOwnerId, "moderator-a");
  await rejectsCode(service.moderate({ session: "mock-moderator-b", channelId, id: record.id, expectedRevision: 2, decision: "approve", idempotencyKey: "approve-b" }), "REVIEW_OWNED");
  await rejectsCode(service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: 1, decision: "approve", idempotencyKey: "stale" }), "STALE_REVISION");
  const result = await service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: 2, decision: "approve", idempotencyKey: "approve-a" });
  assert.equal(result.revision, 3);
});

test("idempotency is exact, actor scoped, concurrent and does not consume another rate slot", async () => {
  const { service, mocks } = environment({ limits: { submissionLimit: 2 } });
  const [first, second] = await Promise.all([submit(service), submit(service)]);
  assert.equal(first.id, second.id);
  assert.equal(mocks.getScreens().length, 1);
  await rejectsCode(submit(service, "submit-1", { text: "Different text" }), "IDEMPOTENCY_CONFLICT");
  const otherActor = await submit(service, "submit-1", { session: "mock-author-b" });
  assert.notEqual(otherActor.id, first.id);
  await submit(service, "submit-2");
  await rejectsCode(submit(service, "submit-3"), "RATE_LIMITED");
  assert.equal((await service.listOwn({ session: author, channelId })).items.length, 2);
  mocks.setModerationAvailable(false);
  await rejectsCode(submit(service), "DEPENDENCY_UNAVAILABLE");
});

test("successful rate limits reset at the boundary and failures do not reserve quota", async () => {
  const env = environment({ limits: { submissionLimit: 1, reviewLimit: 1 } });
  await rejectsCode(submit(env.service, "policy-denial", { text: "mock-blocked" }), "CONTENT_REJECTED");
  env.mocks.setModerationAvailable(false);
  await rejectsCode(submit(env.service, "unavailable"), "DEPENDENCY_UNAVAILABLE");
  env.mocks.setModerationAvailable(true);
  const record = await submit(env.service, "accepted");
  await rejectsCode(submit(env.service, "too-many"), "RATE_LIMITED");
  const claim = await env.service.claimReview({ session: moderator, channelId, id: record.id, expectedRevision: 1, idempotencyKey: "claim" });
  await rejectsCode(env.service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: claim.revision, decision: "approve", idempotencyKey: "approve" }), "RATE_LIMITED");
  env.setTime(61000);
  await submit(env.service, "after-window");
  await env.service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: claim.revision, decision: "approve", idempotencyKey: "approve" });
});

test("capacity is bounded and existing records and idempotent retries remain intact", async () => {
  const { service } = environment({ limits: { maxRecords: 1 } });
  const first = await submit(service);
  await rejectsCode(submit(service, "second", { session: "mock-author-b" }), "CAPACITY_LIMIT");
  assert.equal((await submit(service)).id, first.id);
  assert.equal((await service.getOwn({ session: author, channelId, id: first.id })).text, MOCK_REQUEST.text);
});

test("pending expiry is current on reads and denies claim at the exact boundary", async () => {
  const env = environment({ limits: { pendingTtlMs: 1000 } });
  const record = await submit(env.service);
  env.setTime(2000);
  assert.equal((await env.service.listOwn({ session: author, channelId })).items[0].state, "expired");
  assert.equal((await env.service.listReviewQueue({ session: moderator, channelId })).state, "empty");
  await rejectsCode(env.service.claimReview({ session: moderator, channelId, id: record.id, expectedRevision: 1, idempotencyKey: "late" }), "EXPIRED");
});

test("approval checks current expiry after asynchronous screening and a failed review does not charge quota", async () => {
  let now = 1000;
  let screenCalls = 0;
  const clock = () => now;
  const mocks = createMockAdapters(clock);
  const moderation = { ...mocks.moderation, async screen(input) {
    screenCalls += 1;
    if (screenCalls === 2) now = 2000;
    return mocks.moderation.screen(input);
  } };
  const service = createCalloutService({ auth: mocks.auth, moderation, clock, channels: MOCK_CHANNELS, limits: { pendingTtlMs: 1000, reviewLimit: 2 } });
  const record = await claimed(service);
  await rejectsCode(service.moderate({ session: moderator, channelId, id: record.id, expectedRevision: 2, decision: "approve", idempotencyKey: "late" }), "EXPIRED");
  const another = await submit(service, "another");
  await service.claimReview({ session: moderator, channelId, id: another.id, expectedRevision: 1, idempotencyKey: "claim-another" });
  assert.equal((await service.getOwn({ session: author, channelId, id: record.id })).state, "expired");
});

test("identity or grant changes during screening never commit a callout", async () => {
  for (const variant of ["subject", "grant", "expiry"]) {
    const clock = () => 1000;
    const mocks = createMockAdapters(clock);
    const moderation = { ...mocks.moderation, async screen(input) {
      const value = { subject: variant === "subject" ? "author-b" : "author-a", expiresAt: variant === "expiry" ? 1000 : 5000, grants: [{ channelId, actions: variant === "grant" ? ["display"] : ["submit", "read-own"] }] };
      mocks.setSession(author, value);
      return mocks.moderation.screen(input);
    } };
    const service = createCalloutService({ auth: mocks.auth, moderation, clock, channels: MOCK_CHANNELS });
    await rejectsCode(submit(service), variant === "expiry" ? "UNAUTHENTICATED" : "FORBIDDEN");
    mocks.setSession(author, { subject: "author-a", expiresAt: 5000, grants: [{ channelId, actions: ["read-own"] }] });
    assert.equal((await service.listOwn({ session: author, channelId })).state, "empty");
  }
});

test("dependency exceptions, malformed responses and missing dependencies fail closed without leaking messages", async () => {
  const env = environment();
  const variants = [
    { auth: { async authenticate() { throw new Error("private-provider-message"); } } },
    { auth: { async authenticate() { return { subject: "x", expiresAt: 5000, grants: [{ channelId, actions: ["*"] }] }; } } },
    { auth: { async authenticate() { return { subject: "x", expiresAt: 5000, grants: [,] }; } } },
    { moderation: { async available() { return "yes"; }, async screen() {} } },
    { moderation: { async available() { return true; }, async screen() { return { allowed: true }; } } },
  ];
  for (const variant of variants) {
    const service = createCalloutService({ auth: env.mocks.auth, moderation: env.mocks.moderation, clock: env.clock, channels: MOCK_CHANNELS, ...variant });
    await assert.rejects(submit(service), (error) => error.code === "DEPENDENCY_UNAVAILABLE" && !error.message.includes("private-provider-message"));
  }
  await rejectsCode(createCalloutService().queryDisplay({ session: viewer, channelId }), "DEPENDENCY_UNAVAILABLE");
  env.mocks.setAuthAvailable(false);
  await rejectsCode(env.service.queryDisplay({ session: viewer, channelId }), "DEPENDENCY_UNAVAILABLE");
});

test("adapter accessors and response accessors cannot leak provider details", async () => {
  const env = environment();
  for (const name of ["authenticate", "available", "screen"]) {
    const adapter = name === "authenticate" ? { ...env.mocks.auth } : { ...env.mocks.moderation };
    Object.defineProperty(adapter, name, { get() { throw new Error("synthetic-private-adapter-detail"); } });
    const service = createCalloutService({ auth: name === "authenticate" ? adapter : env.mocks.auth, moderation: name === "authenticate" ? env.mocks.moderation : adapter, clock: env.clock, channels: MOCK_CHANNELS });
    await assert.rejects(service.queryDisplay({ session: viewer, channelId }), (error) => error instanceof CalloutError && error.code === "DEPENDENCY_UNAVAILABLE" && !error.message.includes("synthetic-private-adapter-detail"));
  }
  let reads = 0;
  const principal = { expiresAt: 5000, grants: [{ channelId, actions: ["display"] }] };
  Object.defineProperty(principal, "subject", { enumerable: true, get() { reads += 1; return "viewer"; } });
  const service = createCalloutService({ auth: { async authenticate() { return principal; } }, moderation: env.mocks.moderation, clock: env.clock, channels: MOCK_CHANNELS });
  await rejectsCode(service.queryDisplay({ session: viewer, channelId }), "DEPENDENCY_UNAVAILABLE");
  assert.equal(reads, 0);
});

test("accessor, nonenumerable, symbol and unknown fields are rejected without invoking getters", async () => {
  const { service } = environment();
  let reads = 0;
  for (const kind of ["accessor", "nonenumerable-known", "nonenumerable-unknown", "symbol"]) {
    const input = { ...MOCK_REQUEST };
    if (kind === "accessor") Object.defineProperty(input, "session", { enumerable: true, get() { reads += 1; return author; } });
    if (kind === "nonenumerable-known") Object.defineProperty(input, "session", { enumerable: false, value: author });
    if (kind === "nonenumerable-unknown") Object.defineProperty(input, "approved", { enumerable: false, value: true });
    if (kind === "symbol") input[Symbol("approved")] = true;
    await rejectsCode(service.submit(input), "INVALID_REQUEST");
  }
  assert.equal(reads, 0);
});

test("text is canonical, bounded, single-line and never interpreted as markup", async () => {
  const { service } = environment({ limits: { submissionLimit: 10 } });
  const record = await submit(service, "unicode", { text: "  e\u0301 Beam 🦀  " });
  assert.equal(record.text, "é Beam 🦀");
  for (const text of ["", " ", "<script>alert(1)</script>", "line\nbreak", "\u202eoverride", "a".repeat(281), "🦀".repeat(280), "\ud800"]) {
    await rejectsCode(submit(service, "bad-text", { text }), "INVALID_REQUEST");
  }
  for (const durationMs of [0, 999, 300001, 1.5, NaN]) await rejectsCode(submit(service, "bad-duration", { durationMs }), "INVALID_REQUEST");
});

test("returned objects and input config cannot mutate stored records or authorization", async () => {
  const channels = [...MOCK_CHANNELS];
  const limits = { maxRecords: 2 };
  const env = environment({ channels, limits });
  channels.length = 0;
  limits.maxRecords = 0;
  const record = await submit(env.service);
  record.text = "changed";
  record.provenance.source = "live";
  record.state = "approved";
  record.revision = 999;
  const list = await env.service.listOwn({ session: author, channelId });
  list.items[0].text = "also changed";
  list.items.push({ state: "approved" });
  const own = await env.service.getOwn({ session: author, channelId, id: record.id });
  assert.equal(own.text, MOCK_REQUEST.text);
  assert.equal(own.state, "pending");
  assert.equal(own.revision, 1);
  assert.equal(own.provenance.source, "synthetic-local");
});

test("queries are bounded and deterministically ordered, including tied timestamps", async () => {
  const { service } = environment({ limits: { submissionLimit: 10 } });
  await approved(service, "a");
  await approved(service, "b");
  const displayed = await service.queryDisplay({ session: viewer, channelId, limit: 1 });
  assert.equal(displayed.items.length, 1);
  assert.equal(displayed.items[0].id, "callout-000001");
  const all = await service.queryDisplay({ session: viewer, channelId, limit: 50 });
  assert.deepEqual(all.items.map((item) => item.id), ["callout-000001", "callout-000002"]);
  for (const limit of [0, 51, -1, 1.5, "2"]) await rejectsCode(service.queryDisplay({ session: viewer, channelId, limit }), "INVALID_REQUEST");
});

test("clock failure/rollback and session expiry do not revive or disclose records", async () => {
  const env = environment();
  await approved(env.service);
  env.setTime(11000);
  assert.equal((await env.service.queryDisplay({ session: viewer, channelId })).state, "empty");
  env.setTime(10000);
  await rejectsCode(env.service.queryDisplay({ session: viewer, channelId }), "CLOCK_UNAVAILABLE");
  const broken = environment({ clock: () => NaN });
  await rejectsCode(broken.service.queryDisplay({ session: viewer, channelId }), "CLOCK_UNAVAILABLE");
  const expired = environment();
  expired.mocks.setSession(viewer, { subject: "viewer", expiresAt: 1000, grants: [{ channelId, actions: ["display"] }] });
  await rejectsCode(expired.service.queryDisplay({ session: viewer, channelId }), "UNAUTHENTICATED");
});
