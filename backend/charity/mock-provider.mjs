import { setImmediate as immediate } from 'node:timers/promises';
import { amount, array, digest, record, requireThat, snapshot, syntheticId, text } from './primitives.mjs';

const providers = new WeakSet();
export const MOCK_PROVIDER_ID = 'mock:donation-provider:v1';
const submitKinds = ['settled', 'transient', 'rejected', 'unavailable', 'unknown-before-commit',
  'unknown-after-commit', 'throw-before-commit', 'throw-after-commit', 'mismatch'];
const lookupKinds = ['auto', 'unknown', 'unavailable', 'mismatch'];

export function createMockDonationProvider(options = {}) {
  record(options, ['submitPlan', 'lookupPlan', 'delayTicks'], []);
  const submitPlan = options.submitPlan ?? ['settled'];
  const lookupPlan = options.lookupPlan ?? ['auto'];
  for (const [plan, kinds] of [[submitPlan, submitKinds], [lookupPlan, lookupKinds]]) {
    array(plan, 32);
    requireThat(plan.every(x => kinds.includes(x)), 'INVALID_MOCK_PLAN');
  }
  const submits = [...submitPlan], lookups = [...lookupPlan];
  const ticks = options.delayTicks ?? 1;
  requireThat(Number.isInteger(ticks) && ticks >= 0 && ticks <= 8, 'INVALID_MOCK_DELAY');
  const ledger = new Map(); let submitCalls = 0; let lookupCalls = 0;
  const wait = async () => { for (let i = 0; i < ticks; i++) await immediate(); };
  function admit(request) {
    record(request, ['key', 'providerId', 'charityId', 'recipientId', 'amountBaseUnits', 'currency', 'termsDigest']);
    text(request.key, /^[0-9a-f]{64}$/, 64, 'INVALID_MOCK_BINDING');
    text(request.termsDigest, /^[0-9a-f]{64}$/, 64, 'INVALID_MOCK_BINDING');
    requireThat(request.providerId === MOCK_PROVIDER_ID && request.currency === 'MOCK-UNIT', 'INVALID_MOCK_BINDING');
    syntheticId(request.charityId); syntheticId(request.recipientId); amount(request.amountBaseUnits);
  }
  function committed(request) {
    const old = ledger.get(request.key);
    if (old) {
      requireThat(old.requestDigest === digest(request), 'MOCK_IDEMPOTENCY_CONFLICT');
      return old;
    }
    requireThat(ledger.size < 128, 'MOCK_LEDGER_FULL');
    const receipt = snapshot({ settlementId: `mock:settlement:${ledger.size + 1}`, ...request,
      requestDigest: digest(request), simulation: true, moneyMoved: false });
    ledger.set(request.key, receipt);
    return receipt;
  }
  const provider = Object.freeze({
    id: MOCK_PROVIDER_ID,
    capability: 'local-mock-only',
    async submit(input) {
      admit(input);
      const request = snapshot(input);
      submitCalls++;
      await wait();
      const old = ledger.get(request.key);
      if (old) return { outcome: 'settled', settlement: committed(request) };
      const kind = submits.shift() ?? 'settled';
      if (kind === 'transient' || kind === 'rejected' || kind === 'unavailable') return { outcome: kind, accepted: false };
      if (kind === 'throw-before-commit') throw new Error('synthetic lost response before acceptance');
      if (kind === 'unknown-before-commit') return { outcome: 'unknown' };
      const settlement = committed(request);
      if (kind === 'throw-after-commit') throw new Error('synthetic lost response after acceptance');
      if (kind === 'unknown-after-commit') return { outcome: 'unknown' };
      if (kind === 'mismatch') return { outcome: 'settled', settlement: { ...settlement, recipientId: 'synthetic:recipient:wrong' } };
      return { outcome: 'settled', settlement };
    },
    async lookup(input) {
      admit(input);
      const request = snapshot(input);
      lookupCalls++;
      await wait();
      const kind = lookups.shift() ?? 'auto';
      if (kind === 'unknown' || kind === 'unavailable') return { outcome: kind };
      const settlement = ledger.get(request.key);
      if (!settlement) return { outcome: 'not_found', definitive: true };
      if (kind === 'mismatch') return { outcome: 'settled', settlement: { ...settlement, amountBaseUnits: '999' } };
      return { outcome: 'settled', settlement };
    },
    inspect: () => snapshot({ submitCalls, lookupCalls, settlements: [...ledger.values()],
      effectCount: ledger.size, totalBaseUnits: [...ledger.values()].reduce((n, r) => n + BigInt(r.amountBaseUnits), 0n).toString() }),
  });
  providers.add(provider);
  return provider;
}
export const isLocalMockProvider = value => providers.has(value);
