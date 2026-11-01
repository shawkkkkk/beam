import test from 'node:test';
import assert from 'node:assert/strict';
import { setImmediate as tick } from 'node:timers/promises';
import { CONSENT_VERSION, createCharityService, createLocalPrincipal, createMockDonationProvider,
  createSyntheticRegistry } from './service.mjs';

const alice = createLocalPrincipal({ subject: 'mock:alice', permissions: ['donation:submit', 'donation:read'] });
const bob = createLocalPrincipal({ subject: 'mock:bob', permissions: ['donation:submit', 'donation:read'] });
const reader = createLocalPrincipal({ subject: 'mock:reader', permissions: ['donation:read'] });
const admin = createLocalPrincipal({ subject: 'mock:operator', permissions: ['registry:manage'] });
const code = value => error => error.code === value;
function fixture(plan = {}, options = {}) {
  const provider = createMockDonationProvider(plan), registry = createSyntheticRegistry();
  const service = createCharityService({ enabled: true, provider, registry, clock: () => 1_000, ...options });
  return { provider, registry, service };
}
const prepare = (service, changes = {}) => service.prepare({ principal: alice, idempotencyKey: 'donation-1',
  charityId: 'synthetic:charity:alpha', amountBaseUnits: '9007199254740993', attribution: { mode: 'anonymous' }, ...changes });
const consent = donation => ({ accepted: true, version: CONSENT_VERSION, termsDigest: donation.termsDigest });
const submit = (service, donation, changes = {}) => service.submit({ principal: alice, donationId: donation.donationId,
  consent: consent(donation), ...changes });
const reconcile = (service, donation) => service.reconcile({ principal: alice, donationId: donation.donationId });

test('disabled, absent and spoofed providers never expose a live capability or invoke injected code', () => {
  const provider = createMockDonationProvider();
  const disabled = createCharityService({ provider });
  assert.equal(disabled.capabilities().reason, 'disabled');
  assert.equal(disabled.capabilities().liveDonations, false);
  assert.throws(() => prepare(disabled), code('DONATIONS_UNAVAILABLE'));
  const missing = createCharityService({ enabled: true });
  assert.equal(missing.capabilities().reason, 'missing-local-mock-provider');
  assert.throws(() => prepare(missing), code('DONATIONS_UNAVAILABLE'));
  let invoked = 0;
  const counterfeit = { capability: 'local-mock-only', get id() { invoked++; throw new Error('must not run'); },
    submit() { invoked++; }, lookup() { invoked++; } };
  for (const other of [counterfeit, Object.create(provider), new Proxy(provider, { get() { invoked++; } })]) {
    const service = createCharityService({ enabled: true, provider: other });
    assert.equal(service.capabilities().reason, 'unsupported-provider');
    assert.throws(() => prepare(service), code('DONATIONS_UNAVAILABLE'));
  }
  assert.equal(invoked, 0); assert.equal(provider.inspect().submitCalls, 0);
});

test('fixture authorization rejects JSON identities, insufficient permissions and another owner', async () => {
  const { service, provider } = fixture();
  assert.throws(() => prepare(service, { principal: { ...alice } }), code('UNAUTHENTICATED'));
  assert.throws(() => prepare(service, { principal: reader }), code('FORBIDDEN'));
  const donation = prepare(service);
  assert.throws(() => service.receipt({ principal: bob, donationId: donation.donationId }), code('DONATION_NOT_FOUND'));
  await assert.rejects(submit(service, donation, { principal: bob }), code('DONATION_NOT_FOUND'));
  assert.equal(provider.inspect().submitCalls, 0);
});

test('amounts remain exact canonical strings through the provider ledger, including above Number precision and u64 max', async () => {
  const { service, provider } = fixture();
  const amounts = ['1', '9007199254740993', '18446744073709551615'];
  for (const [index, value] of amounts.entries()) {
    const donation = prepare(service, { idempotencyKey: `amount-${index}`, amountBaseUnits: value });
    const result = await submit(service, donation);
    assert.equal(result.terms.amountBaseUnits, value);
    assert.equal(result.settlement.amountBaseUnits, value);
    assert.equal(result.currency, 'MOCK-UNIT'); assert.equal(result.moneyMoved, false);
  }
  assert.equal(provider.inspect().totalBaseUnits, amounts.reduce((n, a) => n + BigInt(a), 0n).toString());
  assert.equal(provider.inspect().effectCount, 3);
});

test('noncanonical, numeric, zero, excessive and accessor-bearing inputs fail before a provider effect', () => {
  const { service, provider } = fixture();
  for (const amountBaseUnits of [0, 1, 1n, NaN, Infinity, '0', '00', '01', '+1', '-1', '1.0', '1e3', ' 1', '1 ',
    '18446744073709551616', '1'.repeat(100)]) assert.throws(() => prepare(service, { amountBaseUnits }));
  let reads = 0;
  const input = { principal: alice, idempotencyKey: 'getter', amountBaseUnits: '1', attribution: { mode: 'anonymous' } };
  Object.defineProperty(input, 'charityId', { enumerable: true, get() { reads++; return 'synthetic:charity:alpha'; } });
  assert.throws(() => service.prepare(input), code('INVALID_FIELDS'));
  const hidden = { principal: alice, idempotencyKey: 'hidden', amountBaseUnits: '1', attribution: { mode: 'anonymous' } };
  Object.defineProperty(hidden, 'charityId', { value: 'synthetic:charity:alpha', enumerable: false });
  assert.throws(() => service.prepare(hidden), code('INVALID_FIELDS'));
  assert.equal(reads, 0); assert.equal(provider.inspect().effectCount, 0);
  assert.throws(() => prepare(service, { charityId: 'x'.repeat(1000) }), code('INVALID_SYNTHETIC_ID'));
});

test('consent binds exact terms and attribution; anonymous receipts contain no owner identity', async () => {
  const { service, provider } = fixture();
  const original = prepare(service);
  for (const value of [{ accepted: false, version: CONSENT_VERSION, termsDigest: original.termsDigest },
    { accepted: true, version: 'other', termsDigest: original.termsDigest },
    { accepted: true, version: CONSENT_VERSION, termsDigest: '0'.repeat(64) },
    { accepted: true }]) await assert.rejects(submit(service, original, { consent: value }));
  const otherOwner = prepare(service, { principal: bob });
  assert.notEqual(otherOwner.termsDigest, original.termsDigest);
  await assert.rejects(service.submit({ principal: bob, donationId: otherOwner.donationId, consent: consent(original) }), code('CONSENT_REQUIRED'));
  assert.equal(provider.inspect().submitCalls, 0);
  const result = await submit(service, original);
  assert.equal(result.state, 'mock_settled');
  assert.deepEqual(result.terms.attribution, { mode: 'anonymous' });
  assert.equal(JSON.stringify(result).includes(alice.subject), false);
  assert.equal(original.consent, null); assert.equal(original.state, 'awaiting_consent');
  assert.equal(result.consent.acceptedAtMs, 1000);
});

test('named attribution requires explicit bounded consented text and snapshots caller data', async () => {
  const { service } = fixture();
  for (const value of [{ mode: 'anonymous', label: 'hidden name' }, { mode: 'named' }, { mode: 'named', label: '' },
    { mode: 'named', label: ' leading' }, { mode: 'named', label: 'x\n' }, { mode: 'named', label: 'x'.repeat(65) }])
    assert.throws(() => prepare(service, { attribution: value }));
  const attribution = { mode: 'named', label: 'Synthetic donor Alpha' };
  const donation = prepare(service, { attribution }); attribution.label = 'Changed later';
  const result = await submit(service, donation);
  assert.equal(result.terms.attribution.label, 'Synthetic donor Alpha');
  assert.throws(() => { result.terms.attribution.label = 'tamper'; }, TypeError);
  assert.throws(() => { result.settlement.amountBaseUnits = '2'; }, TypeError);
});

test('idempotency keys bind all prepared terms and are scoped to the authenticated fixture owner', () => {
  const { service, provider } = fixture();
  const first = prepare(service); assert.strictEqual(prepare(service), first);
  for (const change of [{ amountBaseUnits: '2' }, { charityId: 'synthetic:charity:beta' },
    { attribution: { mode: 'named', label: 'Explicit synthetic donor' } }])
    assert.throws(() => prepare(service, change), code('IDEMPOTENCY_CONFLICT'));
  assert.notEqual(prepare(service, { principal: bob }).donationId, first.donationId);
  assert.equal(provider.inspect().submitCalls, 0);
});

test('concurrent duplicate operations share one async submission and immutable result', async () => {
  const { service, provider } = fixture({ delayTicks: 8 }); const donation = prepare(service);
  const pending = Array.from({ length: 12 }, () => submit(service, donation));
  await tick();
  assert.equal(service.receipt({ principal: alice, donationId: donation.donationId }).state, 'pending_submission');
  const results = await Promise.all(pending);
  assert.ok(results.every(result => result === results[0]));
  assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 1);
  assert.strictEqual(await submit(service, donation), results[0]);
  assert.equal(provider.inspect().submitCalls, 1);
});

test('definitive transient and unavailable submissions retry without duplicating effects', async () => {
  for (const outcome of ['transient', 'unavailable']) {
    const { service, provider } = fixture({ submitPlan: [outcome, 'settled'] }); const donation = prepare(service);
    const failed = await submit(service, donation);
    assert.equal(failed.state, 'retryable'); assert.equal(provider.inspect().effectCount, 0);
    const result = await submit(service, donation);
    assert.equal(result.state, 'mock_settled'); assert.equal(provider.inspect().submitCalls, 2);
    assert.equal(provider.inspect().effectCount, 1); assert.equal(failed.state, 'retryable');
    const history = service.history({ principal: alice, donationId: donation.donationId });
    assert.deepEqual(history.map(r => r.sequence), [1, 2, 3, 4, 5]);
    assert.throws(() => history.push(result), TypeError);
  }
});

test('unknown before acceptance requires lookup and an explicit later retry', async () => {
  for (const outcome of ['unknown-before-commit', 'throw-before-commit']) {
    const { service, provider } = fixture({ submitPlan: [outcome, 'settled'] }); const donation = prepare(service);
    assert.equal((await submit(service, donation)).state, 'unknown');
    await assert.rejects(submit(service, donation), code('RECONCILIATION_REQUIRED'));
    assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 0);
    assert.equal((await reconcile(service, donation)).state, 'retryable');
    assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().lookupCalls, 1);
    assert.equal((await submit(service, donation)).state, 'mock_settled');
    assert.equal(provider.inspect().effectCount, 1);
  }
});

test('lost responses after acceptance and repeated unknown lookup preserve one mock settlement', async () => {
  for (const outcome of ['unknown-after-commit', 'throw-after-commit']) {
    const { service, provider } = fixture({ submitPlan: [outcome], lookupPlan: ['unknown', 'unavailable', 'auto'], delayTicks: 2 });
    const donation = prepare(service);
    assert.equal((await submit(service, donation)).state, 'unknown'); assert.equal(provider.inspect().effectCount, 1);
    await assert.rejects(submit(service, donation), code('RECONCILIATION_REQUIRED'));
    const group = await Promise.all(Array.from({ length: 8 }, () => reconcile(service, donation)));
    assert.ok(group.every(r => r === group[0])); assert.equal(group[0].state, 'unknown');
    assert.equal(provider.inspect().lookupCalls, 1);
    assert.equal((await reconcile(service, donation)).state, 'unknown');
    assert.equal((await reconcile(service, donation)).state, 'mock_settled');
    assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 1);
  }
});

test('mismatched mock receipts stay unresolved and reconciliation recovers the actual bound record', async () => {
  const { service, provider } = fixture({ submitPlan: ['mismatch'], lookupPlan: ['mismatch', 'auto'] }); const donation = prepare(service);
  const mismatch = await submit(service, donation);
  assert.equal(mismatch.state, 'unknown'); assert.equal(mismatch.reason, 'provider-receipt-mismatch');
  assert.equal(mismatch.settlement, null); assert.equal(provider.inspect().effectCount, 1);
  await assert.rejects(submit(service, donation), code('RECONCILIATION_REQUIRED'));
  assert.equal((await reconcile(service, donation)).state, 'unknown');
  assert.equal((await reconcile(service, donation)).state, 'mock_settled');
  assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 1);
});

test('registry administration is separate and revisions never silently redirect retries', async () => {
  const { service, provider, registry } = fixture({ submitPlan: ['unknown-before-commit'] }); const donation = prepare(service);
  const old = registry.get('synthetic:charity:alpha');
  assert.throws(() => registry.setStatus({ principal: alice, charityId: old.charityId, status: 'revoked' }), code('FORBIDDEN'));
  await submit(service, donation);
  registry.setStatus({ principal: admin, charityId: old.charityId, status: 'revoked' });
  assert.equal(old.verification.status, 'mock-verified');
  assert.equal((await reconcile(service, donation)).state, 'retryable');
  await assert.rejects(submit(service, donation), code('RECIPIENT_UNAVAILABLE'));
  registry.setStatus({ principal: admin, charityId: old.charityId, status: 'mock-verified' });
  await assert.rejects(submit(service, donation), code('RECIPIENT_CHANGED'));
  assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 0);
});

test('a revoked recipient does not prevent resolving an already accepted mock payment', async () => {
  const { service, registry, provider } = fixture({ submitPlan: ['unknown-after-commit'] }); const donation = prepare(service);
  await submit(service, donation);
  registry.setStatus({ principal: admin, charityId: donation.terms.charityId, status: 'revoked' });
  const result = await reconcile(service, donation);
  assert.equal(result.state, 'mock_settled'); assert.equal(provider.inspect().submitCalls, 1);
});

test('revocation while an operation is queued blocks handoff without changing receipts or consuming an attempt', async () => {
  const { service, registry, provider } = fixture(); const donation = prepare(service);
  const operation = submit(service, donation);
  registry.setStatus({ principal: admin, charityId: donation.terms.charityId, status: 'revoked' });
  await assert.rejects(operation, code('RECIPIENT_UNAVAILABLE'));
  const unchanged = service.receipt({ principal: alice, donationId: donation.donationId });
  assert.strictEqual(unchanged, donation); assert.equal(unchanged.submissions, 0);
  assert.equal(provider.inspect().submitCalls, 0); assert.equal(provider.inspect().effectCount, 0);
});

test('synthetic registry provenance and provider/recipient binding reject alternate routes', () => {
  assert.throws(() => createSyntheticRegistry([{ charityId: 'real-charity', recipientId: 'real-wallet',
    providerId: 'Donate.gg', label: 'Real charity' }]));
  const registry = createSyntheticRegistry([{ charityId: 'synthetic:charity:alpha', recipientId: 'synthetic:recipient:alpha',
    providerId: 'mock:another-provider', label: 'Synthetic other provider (fixture)' }]);
  const provider = createMockDonationProvider(), service = createCharityService({ enabled: true, registry, provider });
  assert.throws(() => prepare(service), code('PROVIDER_RECIPIENT_MISMATCH'));
  const normal = fixture();
  assert.throws(() => prepare(normal.service, { recipientId: 'synthetic:recipient:wrong' }), code('INVALID_FIELDS'));
  assert.ok(normal.service.listCharities().entries.every(e => e.verification.kind === 'synthetic-fixture-only'));
  assert.equal(provider.inspect().submitCalls, 0);
});

test('bounded attempts and storage preserve unresolved truth instead of inferring settlement', async () => {
  const limited = fixture({}, { maxDonations: 1 }); prepare(limited.service);
  assert.throws(() => prepare(limited.service, { idempotencyKey: 'second' }), code('DONATION_LIMIT'));
  const { service, provider } = fixture({ submitPlan: ['unknown-before-commit', 'unknown-before-commit', 'unknown-after-commit'],
    lookupPlan: ['unknown', 'auto', 'auto', 'unknown'] });
  const donation = prepare(service);
  await submit(service, donation); await reconcile(service, donation); await reconcile(service, donation);
  await submit(service, donation); await reconcile(service, donation);
  await submit(service, donation); const unresolved = await reconcile(service, donation);
  assert.equal(unresolved.state, 'unknown');
  await assert.rejects(reconcile(service, donation), code('LOOKUP_LIMIT'));
  await assert.rejects(submit(service, donation), code('RECONCILIATION_REQUIRED'));
  assert.equal(service.history({ principal: alice, donationId: donation.donationId }).length, 15);
  assert.equal(provider.inspect().submitCalls, 3); assert.equal(provider.inspect().lookupCalls, 4);
  assert.equal(provider.inspect().effectCount, 1);
  const transient = fixture({ submitPlan: ['transient'] }, { maxSubmissions: 1 }); const d = prepare(transient.service);
  await submit(transient.service, d); await assert.rejects(submit(transient.service, d), code('SUBMISSION_LIMIT'));
  assert.equal(transient.provider.inspect().submitCalls, 1);
});

test('provider ledger independently enforces idempotency and immutable mock effects', async () => {
  const provider = createMockDonationProvider({ delayTicks: 2 });
  const request = Object.freeze({ key: '1'.repeat(64), providerId: provider.id, charityId: 'synthetic:charity:alpha',
    recipientId: 'synthetic:recipient:alpha', amountBaseUnits: '9', currency: 'MOCK-UNIT', termsDigest: '2'.repeat(64) });
  const [a, b] = await Promise.all([provider.submit(request), provider.submit(request)]);
  assert.strictEqual(a.settlement, b.settlement); assert.equal(provider.inspect().effectCount, 1);
  await assert.rejects(provider.submit({ ...request, amountBaseUnits: '10' }), code('MOCK_IDEMPOTENCY_CONFLICT'));
  assert.equal(provider.inspect().effectCount, 1);
  assert.throws(() => { provider.inspect().settlements[0].amountBaseUnits = '0'; }, TypeError);
});

test('the exported mock provider snapshots admitted submit and lookup inputs before awaiting', async () => {
  const provider = createMockDonationProvider({ delayTicks: 2 });
  const original = { key: '3'.repeat(64), providerId: provider.id, charityId: 'synthetic:charity:alpha',
    recipientId: 'synthetic:recipient:alpha', amountBaseUnits: '9', currency: 'MOCK-UNIT', termsDigest: '4'.repeat(64) };
  const mutable = { ...original };
  const pending = provider.submit(mutable);
  mutable.amountBaseUnits = '99'; mutable.key = '5'.repeat(64);
  const result = await pending;
  assert.equal(result.settlement.amountBaseUnits, '9'); assert.equal(result.settlement.key, original.key);
  const lookup = { ...original };
  const resolving = provider.lookup(lookup);
  lookup.key = '6'.repeat(64); lookup.amountBaseUnits = '999';
  assert.equal((await resolving).settlement.amountBaseUnits, '9');
  assert.equal(provider.inspect().effectCount, 1); assert.equal(provider.inspect().totalBaseUnits, '9');
});

test('invalid configuration and clock failures stop before dispatch; operation timestamps cannot fail after acceptance', async () => {
  for (const options of [{ maxDonations: 0 }, { maxDonations: 65 }, { maxSubmissions: 4 }, { maxLookups: 5 }, { clock: 'clock' }])
    assert.throws(() => createCharityService(options));
  assert.throws(() => createMockDonationProvider({ submitPlan: [() => {}] }), code('INVALID_MOCK_PLAN'));
  let hooks = 0;
  const hiddenPlan = [];
  Object.defineProperty(hiddenPlan, '0', { get() { hooks++; return 'settled'; }, enumerable: true });
  assert.throws(() => createMockDonationProvider({ submitPlan: hiddenPlan }), code('INVALID_ARRAY'));
  assert.throws(() => createMockDonationProvider({ submitPlan: Array(2) }), code('INVALID_ARRAY'));
  assert.equal(hooks, 0);
  let invalid = false;
  const f = fixture({}, { clock: () => invalid ? NaN : 10 }); const donation = prepare(f.service);
  invalid = true; await assert.rejects(submit(f.service, donation), code('INVALID_CLOCK'));
  assert.equal(f.provider.inspect().submitCalls, 0);
  assert.equal(f.service.receipt({ principal: alice, donationId: donation.donationId }).state, 'awaiting_consent');
  invalid = false; assert.equal((await submit(f.service, donation)).state, 'mock_settled');
  let reads = 0;
  const once = fixture({}, { clock: () => { reads++; if (reads > 2) throw new Error('clock unavailable'); return 10; } });
  const d = prepare(once.service); assert.equal((await submit(once.service, d)).state, 'mock_settled'); assert.equal(reads, 2);
});

test('definitive rejection is terminal and never consumes a second submission', async () => {
  const { service, provider } = fixture({ submitPlan: ['rejected', 'settled'] }); const donation = prepare(service);
  assert.equal((await submit(service, donation)).state, 'rejected');
  await assert.rejects(submit(service, donation), code('DONATION_REJECTED'));
  assert.equal(provider.inspect().submitCalls, 1); assert.equal(provider.inspect().effectCount, 0);
});
