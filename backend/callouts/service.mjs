/** Proposed local Beam product boundary; no transport or external actions. */

export const PROVENANCE = Object.freeze({
  source: "synthetic-local",
  verifiedOnChain: false,
  externalActionsEnabled: false,
});

const DEFAULTS = Object.freeze({
  maxRecords: 200,
  submissionLimit: 3,
  reviewLimit: 20,
  rateWindowMs: 60_000,
  pendingTtlMs: 15 * 60_000,
});
const ACTIONS = new Set(["submit", "read-own", "moderate", "display"]);
const REASONS = new Set(["off-topic", "abuse", "duplicate", "other"]);
const MAX_SCHEDULE_MS = 60 * 60_000;
const MAX_DURATION_MS = 300_000;
const MESSAGES = Object.freeze({
  INVALID_REQUEST: "Check the callout request and try again.",
  UNAUTHENTICATED: "Sign in to continue.",
  FORBIDDEN: "This account cannot perform that callout action.",
  NOT_FOUND: "That callout is unavailable.",
  DEPENDENCY_UNAVAILABLE: "Callouts are temporarily unavailable.",
  CLOCK_UNAVAILABLE: "Callouts are temporarily unavailable.",
  CONTENT_REJECTED: "This text cannot be submitted for a callout.",
  IDEMPOTENCY_CONFLICT: "Use a new request key for a different action.",
  RATE_LIMITED: "Too many callout requests. Try again later.",
  CAPACITY_LIMIT: "Callouts are at local capacity. Try again later.",
  REVIEW_OWNED: "Another moderator owns this review.",
  SELF_REVIEW: "A different moderator must review this callout.",
  STALE_REVISION: "This callout changed. Refresh it before continuing.",
  TERMINAL_STATE: "This callout already has a moderation decision.",
  EXPIRED: "This callout has expired.",
});

export class CalloutError extends Error {
  constructor(code) {
    super(MESSAGES[code] ?? MESSAGES.INVALID_REQUEST);
    this.name = "CalloutError";
    this.code = code;
  }

  toJSON() {
    return { code: this.code, message: this.message, role: "status", ariaLive: "polite" };
  }
}

const fail = (code) => { throw new CalloutError(code); };
const requireValue = (condition, code = "INVALID_REQUEST") => {
  if (!condition) fail(code);
};
const integer = (value, min, max) => Number.isSafeInteger(value) && value >= min && value <= max;
const identifier = (value) => typeof value === "string" && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(value);
const keyValid = (value) => typeof value === "string" && /^[A-Za-z0-9][A-Za-z0-9_-]{0,79}$/.test(value);

function plainRecord(value, allowed) {
  requireValue(value !== null && typeof value === "object" && !Array.isArray(value));
  const prototype = Object.getPrototypeOf(value);
  requireValue(prototype === Object.prototype || prototype === null);
  for (const key of Reflect.ownKeys(value)) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key);
    requireValue(typeof key === "string" && allowed.includes(key) && descriptor?.enumerable && "value" in descriptor);
  }
}

function request(value, extra) {
  plainRecord(value, ["session", "channelId", ...extra]);
  requireValue(typeof value.session === "string" && value.session.length > 0 && value.session.length <= 128);
  requireValue(identifier(value.channelId));
  return { ...value };
}

function textValue(value) {
  requireValue(typeof value === "string" && value.length <= 2048);
  const normalized = value.normalize("NFC");
  requireValue(!/[\u0000-\u001f\u007f-\u009f<>\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u.test(normalized));
  const text = normalized.trim();
  const characters = [...text];
  requireValue(characters.length > 0 && characters.length <= 280 && Buffer.byteLength(text, "utf8") <= 1024);
  requireValue(characters.every((character) => {
    const point = character.codePointAt(0);
    return point < 0xd800 || point > 0xdfff;
  }));
  return text;
}

function mutation(value, extra) {
  const args = request(value, ["idempotencyKey", ...extra]);
  requireValue(keyValid(args.idempotencyKey));
  return args;
}

function recordMutation(value, extra) {
  const args = mutation(value, ["id", "expectedRevision", ...extra]);
  requireValue(identifier(args.id) && integer(args.expectedRevision, 1, Number.MAX_SAFE_INTEGER));
  return args;
}

function query(value, extra = []) {
  const args = request(value, ["limit", ...extra]);
  const limit = args.limit ?? 20;
  requireValue(integer(limit, 1, 50));
  return { ...args, limit };
}

function currentState(record, now) {
  if (record.moderationState === "rejected") return { state: "rejected", displayState: "rejected" };
  if (record.moderationState === "pending") {
    return now >= record.pendingExpiresAt
      ? { state: "expired", displayState: "expired" }
      : { state: "pending", displayState: "awaiting-review" };
  }
  if (now >= record.endsAt) return { state: "expired", displayState: "expired" };
  return { state: "approved", displayState: now < record.startsAt ? "scheduled" : "displaying" };
}

const LABELS = Object.freeze({
  "awaiting-review": "Callout awaiting moderation",
  scheduled: "Approved callout scheduled",
  displaying: "Approved callout displaying",
  rejected: "Callout rejected",
  expired: "Callout expired",
});

function view(record, now, kind = "own") {
  const state = currentState(record, now);
  const result = {
    id: record.id,
    channelId: record.channelId,
    text: record.text,
    ...state,
    startsAt: record.startsAt ?? null,
    endsAt: record.endsAt ?? null,
    provenance: { ...PROVENANCE },
    accessibility: { role: "status", ariaLive: "polite", label: LABELS[state.displayState], contentType: "text/plain" },
  };
  if (kind !== "display") Object.assign(result, {
    revision: record.revision,
    durationMs: record.durationMs,
    createdAt: record.createdAt,
    pendingExpiresAt: record.pendingExpiresAt,
    reason: record.reason ?? null,
    policyVersion: record.policyVersion,
  });
  if (kind === "review") Object.assign(result, { authorId: record.authorId, reviewOwnerId: record.reviewOwnerId });
  return result;
}

function listView(items, now, kind) {
  return {
    state: items.length ? "ready" : "empty",
    items: items.map((item) => view(item, now, kind)),
    provenance: { ...PROVENANCE },
    accessibility: {
      role: "status", ariaLive: "polite",
      label: items.length ? `${items.length} callout${items.length === 1 ? "" : "s"} available` : "No callouts available",
    },
  };
}

export function createCalloutService({ auth, moderation, clock, channels = [], limits = {} } = {}) {
  plainRecord(limits, Object.keys(DEFAULTS));
  const config = { ...DEFAULTS, ...limits };
  requireValue(integer(config.maxRecords, 1, 1000));
  requireValue(integer(config.submissionLimit, 1, 100) && integer(config.reviewLimit, 1, 100));
  requireValue(integer(config.rateWindowMs, 1, 86_400_000) && integer(config.pendingTtlMs, 1000, 3_600_000));
  requireValue(Array.isArray(channels) && channels.length <= 100 && Array.from(channels).every(identifier));
  const registered = new Set(channels);
  requireValue(registered.size === channels.length);
  const records = new Map();
  const idempotency = new Map();
  const rates = new Map();
  let sequence = 0;
  let lastNow = -1;
  let pending = Promise.resolve();

  function nowValue() {
    let now;
    try { now = clock(); } catch { fail("CLOCK_UNAVAILABLE"); }
    const margin = Math.max(config.pendingTtlMs, MAX_SCHEDULE_MS + MAX_DURATION_MS);
    requireValue(integer(now, 0, 8_640_000_000_000_000 - margin) && now >= lastNow, "CLOCK_UNAVAILABLE");
    lastNow = now;
    return now;
  }

  async function authorize(session, channelId, action) {
    let available;
    let principal;
    try {
      const authenticate = auth?.authenticate;
      const availability = moderation?.available;
      const screening = moderation?.screen;
      requireValue(typeof authenticate === "function" && typeof availability === "function" && typeof screening === "function" && typeof clock === "function", "DEPENDENCY_UNAVAILABLE");
      available = await Reflect.apply(availability, moderation, []);
      if (available === true) principal = await Reflect.apply(authenticate, auth, [session]);
    } catch { fail("DEPENDENCY_UNAVAILABLE"); }
    requireValue(available === true, "DEPENDENCY_UNAVAILABLE");
    requireValue(principal !== null && principal !== undefined, "UNAUTHENTICATED");
    let subject;
    let expiresAt;
    let grants;
    try {
      plainRecord(principal, ["subject", "expiresAt", "grants"]);
      requireValue(identifier(principal.subject) && Number.isSafeInteger(principal.expiresAt) && Array.isArray(principal.grants) && principal.grants.length <= 100);
      subject = principal.subject;
      expiresAt = principal.expiresAt;
      grants = Array.from(principal.grants).map((grant) => {
        plainRecord(grant, ["channelId", "actions"]);
        requireValue(identifier(grant.channelId) && Array.isArray(grant.actions) && grant.actions.length <= 4 && Array.from(grant.actions).every((name) => ACTIONS.has(name)));
        return { channelId: grant.channelId, actions: [...grant.actions] };
      });
    } catch { fail("DEPENDENCY_UNAVAILABLE"); }
    const now = nowValue();
    requireValue(expiresAt > now, "UNAUTHENTICATED");
    requireValue(registered.has(channelId), "NOT_FOUND");
    requireValue(grants.some((grant) => grant.channelId === channelId && grant.actions.includes(action)), "FORBIDDEN");
    return { subject, now };
  }

  async function screen(channelId, authorId, text) {
    let result;
    try { result = await moderation.screen(Object.freeze({ channelId, authorId, text })); }
    catch { fail("DEPENDENCY_UNAVAILABLE"); }
    let allowed;
    let policyVersion;
    try {
      plainRecord(result, ["allowed", "policyVersion"]);
      requireValue(typeof result.allowed === "boolean" && identifier(result.policyVersion));
      allowed = result.allowed;
      policyVersion = result.policyVersion;
    } catch { fail("DEPENDENCY_UNAVAILABLE"); }
    requireValue(allowed, "CONTENT_REJECTED");
    return policyVersion;
  }

  function serial(operation) {
    const next = pending.then(operation, operation);
    pending = next.then(() => undefined, () => undefined);
    return next;
  }

  function idemKey(args, actor, operation) {
    return JSON.stringify([actor, args.channelId, operation, args.idempotencyKey]);
  }

  function replay(args, actor, operation, fingerprint) {
    const saved = idempotency.get(idemKey(args, actor, operation));
    if (!saved) return undefined;
    requireValue(saved.fingerprint === fingerprint, "IDEMPOTENCY_CONFLICT");
    return records.get(saved.id);
  }

  function remember(args, actor, operation, fingerprint, record) {
    idempotency.set(idemKey(args, actor, operation), { fingerprint, id: record.id });
  }

  function rate(actor, channelId, group, now, commit = true) {
    const key = JSON.stringify([actor, channelId, group]);
    const times = (rates.get(key) ?? []).filter((time) => now - time < config.rateWindowMs);
    const maximum = group === "submit" ? config.submissionLimit : config.reviewLimit;
    requireValue(times.length < maximum, "RATE_LIMITED");
    if (commit) {
      for (const [other, history] of rates) {
        const active = history.filter((time) => now - time < config.rateWindowMs);
        if (active.length) rates.set(other, active);
        else rates.delete(other);
      }
      requireValue(rates.has(key) || rates.size < config.maxRecords * 3, "CAPACITY_LIMIT");
      rates.set(key, [...times, now]);
    }
  }

  function find(args) {
    const record = records.get(args.id);
    requireValue(record && record.channelId === args.channelId, "NOT_FOUND");
    return record;
  }

  function assertPending(record, revision, now) {
    requireValue(record.moderationState === "pending", "TERMINAL_STATE");
    requireValue(now < record.pendingExpiresAt, "EXPIRED");
    requireValue(record.revision === revision, "STALE_REVISION");
  }

  async function submit(value) {
    const args = mutation(value, ["text", "durationMs"]);
    args.text = textValue(args.text);
    args.durationMs ??= 30_000;
    requireValue(integer(args.durationMs, 1000, MAX_DURATION_MS));
    const fingerprint = JSON.stringify([args.text, args.durationMs]);
    return serial(async () => {
      const initial = await authorize(args.session, args.channelId, "submit");
      const saved = replay(args, initial.subject, "submit", fingerprint);
      if (saved) return view(saved, initial.now);
      requireValue(records.size < config.maxRecords, "CAPACITY_LIMIT");
      rate(initial.subject, args.channelId, "submit", initial.now, false);
      const policyVersion = await screen(args.channelId, initial.subject, args.text);
      const current = await authorize(args.session, args.channelId, "submit");
      requireValue(current.subject === initial.subject, "FORBIDDEN");
      rate(current.subject, args.channelId, "submit", current.now);
      const record = {
        id: `callout-${String(++sequence).padStart(6, "0")}`,
        channelId: args.channelId, authorId: current.subject,
        text: args.text, durationMs: args.durationMs,
        createdAt: current.now, pendingExpiresAt: current.now + config.pendingTtlMs,
        moderationState: "pending", reviewOwnerId: null, revision: 1, policyVersion,
      };
      records.set(record.id, record);
      remember(args, current.subject, "submit", fingerprint, record);
      return view(record, current.now);
    });
  }

  async function claimReview(value) {
    const args = recordMutation(value, []);
    const fingerprint = JSON.stringify([args.id, args.expectedRevision]);
    return serial(async () => {
      const current = await authorize(args.session, args.channelId, "moderate");
      const saved = replay(args, current.subject, "claim", fingerprint);
      if (saved) return view(saved, current.now, "review");
      const record = find(args);
      requireValue(record.authorId !== current.subject, "SELF_REVIEW");
      requireValue(record.reviewOwnerId === null || record.reviewOwnerId === current.subject, "REVIEW_OWNED");
      assertPending(record, args.expectedRevision, current.now);
      requireValue(record.reviewOwnerId === null, "STALE_REVISION");
      rate(current.subject, args.channelId, "review", current.now);
      record.reviewOwnerId = current.subject;
      record.revision += 1;
      remember(args, current.subject, "claim", fingerprint, record);
      return view(record, current.now, "review");
    });
  }

  async function moderate(value) {
    const args = recordMutation(value, ["decision", "startsAt", "reason"]);
    requireValue(args.decision === "approve" || args.decision === "reject");
    if (args.decision === "approve") {
      requireValue(args.reason === undefined && (args.startsAt === undefined || integer(args.startsAt, 0, 8_640_000_000_000_000)));
    } else requireValue(REASONS.has(args.reason) && args.startsAt === undefined);
    const fingerprint = JSON.stringify([args.id, args.expectedRevision, args.decision, args.startsAt ?? null, args.reason ?? null]);
    return serial(async () => {
      const initial = await authorize(args.session, args.channelId, "moderate");
      const saved = replay(args, initial.subject, "moderate", fingerprint);
      if (saved) return view(saved, initial.now, "review");
      const record = find(args);
      requireValue(record.authorId !== initial.subject, "SELF_REVIEW");
      requireValue(record.reviewOwnerId === initial.subject, "REVIEW_OWNED");
      assertPending(record, args.expectedRevision, initial.now);
      rate(initial.subject, args.channelId, "review", initial.now, false);
      let policyVersion = record.policyVersion;
      if (args.decision === "approve") policyVersion = await screen(args.channelId, record.authorId, record.text);
      const current = await authorize(args.session, args.channelId, "moderate");
      requireValue(current.subject === initial.subject, "FORBIDDEN");
      assertPending(record, args.expectedRevision, current.now);
      const startsAt = args.startsAt ?? current.now;
      if (args.decision === "approve") requireValue(startsAt >= current.now && startsAt <= current.now + MAX_SCHEDULE_MS);
      rate(current.subject, args.channelId, "review", current.now);
      if (args.decision === "approve") {
        record.startsAt = startsAt;
        record.endsAt = startsAt + record.durationMs;
        record.moderationState = "approved";
        record.policyVersion = policyVersion;
      } else {
        record.moderationState = "rejected";
        record.reason = args.reason;
      }
      record.revision += 1;
      remember(args, current.subject, "moderate", fingerprint, record);
      return view(record, current.now, "review");
    });
  }

  async function getOwn(value) {
    const args = request(value, ["id"]);
    requireValue(identifier(args.id));
    const current = await authorize(args.session, args.channelId, "read-own");
    const record = find(args);
    requireValue(record.authorId === current.subject, "NOT_FOUND");
    return view(record, current.now);
  }

  async function listOwn(value) {
    const args = query(value);
    const current = await authorize(args.session, args.channelId, "read-own");
    const items = [...records.values()].filter((record) => record.channelId === args.channelId && record.authorId === current.subject)
      .sort((a, b) => b.createdAt - a.createdAt || a.id.localeCompare(b.id)).slice(0, args.limit);
    return listView(items, current.now, "own");
  }

  async function listReviewQueue(value) {
    const args = query(value);
    const current = await authorize(args.session, args.channelId, "moderate");
    const items = [...records.values()].filter((record) => record.channelId === args.channelId && record.authorId !== current.subject && record.moderationState === "pending" && current.now < record.pendingExpiresAt)
      .sort((a, b) => a.createdAt - b.createdAt || a.id.localeCompare(b.id)).slice(0, args.limit);
    return listView(items, current.now, "review");
  }

  async function queryDisplay(value) {
    const args = query(value);
    const current = await authorize(args.session, args.channelId, "display");
    const items = [...records.values()].filter((record) => record.channelId === args.channelId && record.moderationState === "approved" && current.now >= record.startsAt && current.now < record.endsAt)
      .sort((a, b) => a.startsAt - b.startsAt || a.id.localeCompare(b.id)).slice(0, args.limit);
    return listView(items, current.now, "display");
  }

  return Object.freeze({ submit, claimReview, moderate, getOwn, listOwn, listReviewQueue, queryDisplay });
}
