import { amount, attribution, authorize, digest, freeze, mockId, record, requireThat, snapshot, syntheticId, text } from './primitives.mjs';
import { createSyntheticRegistry, isSyntheticRegistry } from './registry.mjs';
import { isLocalMockProvider } from './mock-provider.mjs';

export { createLocalPrincipal, CharityError } from './primitives.mjs';
export { createSyntheticRegistry } from './registry.mjs';
export { createMockDonationProvider } from './mock-provider.mjs';
export const CONSENT_VERSION = 'beam.charity.consent.v1';
const CURRENCY = 'MOCK-UNIT';

export function createCharityService({ enabled = false, registry = createSyntheticRegistry(), provider = null,
  clock = Date.now, maxDonations = 64, maxSubmissions = 3, maxLookups = 4 } = {}) {
  requireThat(typeof enabled === 'boolean' && typeof clock === 'function', 'INVALID_CONFIGURATION');
  requireThat(isSyntheticRegistry(registry), 'INVALID_REGISTRY');
  for (const [value, cap] of [[maxDonations, 64], [maxSubmissions, 3], [maxLookups, 4]]) {
    requireThat(Number.isInteger(value) && value >= 1 && value <= cap, 'INVALID_LIMIT');
  }
  const donations = new Map(), keys = new Map();
  const capability = () => freeze({ liveDonations: false, simulation: true,
    localMockAvailable: enabled && isLocalMockProvider(provider), reason: !enabled ? 'disabled'
      : provider === null ? 'missing-local-mock-provider' : !isLocalMockProvider(provider) ? 'unsupported-provider' : 'local-mock-only' });
  function available() { requireThat(capability().localMockAvailable, 'DONATIONS_UNAVAILABLE'); }
  function now() {
    const value = clock(); requireThat(Number.isSafeInteger(value) && value >= 0, 'INVALID_CLOCK'); return value;
  }
  function owner(principal, id, permission) {
    const subject = authorize(principal, permission);
    mockId(id);
    const item = donations.get(id);
    requireThat(item && item.subject === subject, 'DONATION_NOT_FOUND');
    return item;
  }
  function recipient(charityId) {
    syntheticId(charityId);
    const entry = registry.get(charityId);
    requireThat(entry && entry.verification.status === 'mock-verified', 'RECIPIENT_UNAVAILABLE');
    requireThat(entry.providerId === provider.id, 'PROVIDER_RECIPIENT_MISMATCH');
    return entry;
  }
  function currentBinding(item) {
    const entry = recipient(item.terms.charityId);
    requireThat(entry.recipientId === item.terms.recipientId && entry.providerId === item.terms.providerId
      && entry.verification.revision === item.terms.registry.revision, 'RECIPIENT_CHANGED');
  }
  function append(item, state, reason, settlement = null, atMs = now()) {
    requireThat(item.history.length < 16, 'RECEIPT_LIMIT');
    item.state = state;
    const receipt = snapshot({ donationId: item.id, sequence: item.history.length + 1, state, reason,
      simulation: true, moneyMoved: false, currency: CURRENCY, terms: item.terms, termsDigest: item.termsDigest,
      consent: item.consent, submissions: item.submissions, lookups: item.lookups, atMs, settlement });
    item.history.push(receipt); return receipt;
  }
  const latest = item => item.history.at(-1);
  function consent(item, value) {
    record(value, ['accepted', 'version', 'termsDigest']);
    requireThat(value.accepted === true && value.version === CONSENT_VERSION
      && value.termsDigest === item.termsDigest, 'CONSENT_REQUIRED');
  }
  function request(item) {
    return snapshot({ key: item.providerKey, providerId: item.terms.providerId, charityId: item.terms.charityId,
      recipientId: item.terms.recipientId, amountBaseUnits: item.terms.amountBaseUnits,
      currency: CURRENCY, termsDigest: item.termsDigest });
  }
  function settled(item, result, atMs) {
    const expected = request(item), value = result.settlement;
    const match = value && typeof value === 'object'
      && Object.entries(expected).every(([key, v]) => value[key] === v)
      && value.requestDigest === digest(expected) && value.simulation === true && value.moneyMoved === false
      && typeof value.settlementId === 'string' && /^mock:settlement:[1-9][0-9]*$/.test(value.settlementId);
    return match ? append(item, 'mock_settled', 'matching-mock-settlement', value, atMs)
      : append(item, 'unknown', 'provider-receipt-mismatch', null, atMs);
  }
  function launch(item, action) {
    if (item.inFlight) return item.inFlight;
    const promise = Promise.resolve().then(action).finally(() => { item.inFlight = null; });
    item.inFlight = promise; return promise;
  }
  return Object.freeze({
    capabilities: capability,
    listCharities: () => snapshot({ source: 'synthetic-fixture-only', capability: capability(), entries: registry.list() }),
    prepare(input) {
      record(input, ['principal', 'idempotencyKey', 'charityId', 'amountBaseUnits', 'attribution']);
      const subject = authorize(input.principal, 'donation:submit'); available();
      const key = text(input.idempotencyKey, /^[A-Za-z0-9._:-]+$/, 80, 'INVALID_IDEMPOTENCY_KEY');
      const entry = recipient(input.charityId);
      const terms = snapshot({ charityId: entry.charityId, recipientId: entry.recipientId, providerId: entry.providerId,
        registry: entry.verification, amountBaseUnits: amount(input.amountBaseUnits), currency: CURRENCY,
        attribution: attribution(input.attribution), consentVersion: CONSENT_VERSION });
      const binding = digest({ subject, terms });
      const scopedKey = JSON.stringify([subject, key]);
      if (keys.has(scopedKey)) {
        const item = donations.get(keys.get(scopedKey));
        requireThat(item.termsDigest === binding, 'IDEMPOTENCY_CONFLICT');
        return latest(item);
      }
      requireThat(donations.size < maxDonations, 'DONATION_LIMIT');
      const id = `mock:donation:${donations.size + 1}`;
      const item = { id, subject, terms, termsDigest: binding, providerKey: digest({ subject, key }),
        consent: null, submissions: 0, lookups: 0, history: [], inFlight: null, state: 'awaiting_consent' };
      const receipt = append(item, 'awaiting_consent', 'explicit-consent-required');
      donations.set(id, item); keys.set(scopedKey, id); return receipt;
    },
    async submit(input) {
      record(input, ['principal', 'donationId', 'consent']);
      const item = owner(input.principal, input.donationId, 'donation:submit');
      consent(item, input.consent);
      if (item.inFlight) return item.inFlight;
      if (item.state === 'mock_settled') return latest(item);
      requireThat(item.state !== 'unknown', 'RECONCILIATION_REQUIRED');
      requireThat(item.state !== 'rejected', 'DONATION_REJECTED'); available();
      currentBinding(item);
      requireThat(item.submissions < maxSubmissions, 'SUBMISSION_LIMIT');
      requireThat(item.history.length + 2 <= 16, 'RECEIPT_LIMIT');
      return launch(item, async () => {
        const atMs = now();
        // Recheck after the queued microtask and trusted clock, before handing
        // the immutable request to the mock provider. No await intervenes.
        currentBinding(item);
        item.consent ??= snapshot({ version: CONSENT_VERSION, termsDigest: item.termsDigest, acceptedAtMs: atMs });
        item.submissions++; append(item, 'pending_submission', 'local-mock-submit', null, atMs);
        let result;
        try { result = await provider.submit(request(item)); }
        catch { return append(item, 'unknown', 'provider-response-unknown', null, atMs); }
        if (result?.outcome === 'settled') return settled(item, result, atMs);
        if (['transient', 'unavailable'].includes(result?.outcome) && result.accepted === false)
          return append(item, 'retryable', result.outcome === 'transient' ? 'definitely-not-accepted' : 'provider-unavailable', null, atMs);
        if (result?.outcome === 'rejected' && result.accepted === false) return append(item, 'rejected', 'provider-rejected', null, atMs);
        return append(item, 'unknown', 'provider-response-unknown', null, atMs);
      });
    },
    async reconcile(input) {
      record(input, ['principal', 'donationId']);
      const item = owner(input.principal, input.donationId, 'donation:read');
      if (item.inFlight) return item.inFlight;
      if (item.state === 'mock_settled') return latest(item);
      requireThat(item.state === 'unknown', 'NOT_UNCERTAIN'); available();
      requireThat(item.lookups < maxLookups, 'LOOKUP_LIMIT');
      requireThat(item.history.length + 2 <= 16, 'RECEIPT_LIMIT');
      return launch(item, async () => {
        const atMs = now();
        item.lookups++; append(item, 'pending_reconciliation', 'local-mock-lookup', null, atMs);
        let result;
        try { result = await provider.lookup(request(item)); }
        catch { return append(item, 'unknown', 'provider-response-unknown', null, atMs); }
        if (result?.outcome === 'settled') return settled(item, result, atMs);
        if (result?.outcome === 'not_found' && result.definitive === true) return append(item, 'retryable', 'definitive-mock-not-found', null, atMs);
        return append(item, 'unknown', result?.outcome === 'unavailable' ? 'lookup-unavailable' : 'lookup-unknown', null, atMs);
      });
    },
    receipt(input) { record(input, ['principal', 'donationId']); return latest(owner(input.principal, input.donationId, 'donation:read')); },
    history(input) { record(input, ['principal', 'donationId']); return snapshot(owner(input.principal, input.donationId, 'donation:read').history); },
  });
}
