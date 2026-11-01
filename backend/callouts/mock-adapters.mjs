/** Synthetic local adapters. Not real identity, moderation, or chain providers. */
export const MOCK_CHANNELS = Object.freeze(["mock-token-alpha", "mock-token-beta"]);

export const MOCK_REQUEST = Object.freeze({
  session: "mock-author-a", channelId: MOCK_CHANNELS[0],
  text: "Local Beam preview callout", durationMs: 10_000,
  idempotencyKey: "mock-submit-1",
});

export function createMockAdapters(clock) {
  const principal = (subject, actions, channelId = MOCK_CHANNELS[0]) => ({
    subject, expiresAt: clock() + 86_400_000, grants: [{ channelId, actions }],
  });
  const sessions = new Map([
    ["mock-author-a", principal("author-a", ["submit", "read-own", "display"])],
    ["mock-author-b", principal("author-b", ["submit", "read-own", "display"])],
    ["mock-moderator-a", principal("moderator-a", ["submit", "read-own", "moderate", "display"])],
    ["mock-moderator-b", principal("moderator-b", ["moderate", "display"])],
    ["mock-viewer", principal("viewer", ["display"])],
  ]);
  let authAvailable = true;
  let moderationAvailable = true;
  const screens = [];
  return {
    auth: {
      async authenticate(session) {
        if (!authAvailable) throw new Error("synthetic auth unavailable");
        return sessions.has(session) ? structuredClone(sessions.get(session)) : null;
      },
    },
    moderation: {
      async available() { return moderationAvailable; },
      async screen(record) {
        if (!moderationAvailable) throw new Error("synthetic moderation unavailable");
        if (screens.length < 1000) screens.push(structuredClone(record));
        return { allowed: !record.text.toLowerCase().includes("mock-blocked"), policyVersion: "synthetic-policy-v1" };
      },
    },
    setAuthAvailable(value) { authAvailable = value === true; },
    setModerationAvailable(value) { moderationAvailable = value === true; },
    setSession(session, value) { sessions.set(session, structuredClone(value)); },
    getScreens() { return structuredClone(screens); },
    provenance: { source: "synthetic-local", verifiedOnChain: false },
  };
}
