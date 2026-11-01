import { array, authorize, record, requireThat, syntheticId, mockId, snapshot } from './primitives.mjs';

const registries = new WeakSet();
const defaults = [
  { charityId: 'synthetic:charity:alpha', recipientId: 'synthetic:recipient:alpha',
    providerId: 'mock:donation-provider:v1', label: 'Synthetic charity Alpha (fixture)' },
  { charityId: 'synthetic:charity:beta', recipientId: 'synthetic:recipient:beta',
    providerId: 'mock:donation-provider:v1', label: 'Synthetic charity Beta (fixture)' },
];

export function createSyntheticRegistry(entries = defaults) {
  array(entries, 16);
  requireThat(entries.length > 0, 'INVALID_REGISTRY_SIZE');
  const records = new Map();
  for (const entry of entries) {
    record(entry, ['charityId', 'recipientId', 'providerId', 'label']);
    syntheticId(entry.charityId); syntheticId(entry.recipientId); mockId(entry.providerId);
    requireThat(typeof entry.label === 'string' && entry.label.startsWith('Synthetic ') && entry.label.endsWith(' (fixture)')
      && entry.label.length <= 96, 'INVALID_FIXTURE_LABEL');
    requireThat(!records.has(entry.charityId), 'DUPLICATE_CHARITY');
    records.set(entry.charityId, snapshot({ ...entry, verification: {
      status: 'mock-verified', kind: 'synthetic-fixture-only', issuer: 'mock:registry:v1', revision: 1,
    } }));
  }
  const registry = Object.freeze({
    list: () => snapshot([...records.values()]),
    get: id => records.get(id) ?? null,
    setStatus(input) {
      record(input, ['principal', 'charityId', 'status']);
      authorize(input.principal, 'registry:manage');
      requireThat(['mock-verified', 'unverified', 'revoked'].includes(input.status), 'INVALID_REGISTRY_STATUS');
      const old = records.get(input.charityId);
      requireThat(old, 'CHARITY_NOT_FOUND');
      requireThat(old.verification.revision < 1024, 'REGISTRY_REVISION_LIMIT');
      const next = snapshot({ ...old, verification: { ...old.verification,
        status: input.status, revision: old.verification.revision + 1 } });
      records.set(input.charityId, next);
      return next;
    },
  });
  registries.add(registry);
  return registry;
}
export const isSyntheticRegistry = value => registries.has(value);
