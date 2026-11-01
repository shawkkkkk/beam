import { createHash } from 'node:crypto';

export class CharityError extends Error {
  constructor(code) { super(code); this.name = 'CharityError'; this.code = code; }
}
export function requireThat(condition, code) { if (!condition) throw new CharityError(code); }
export function record(value, allowed, required = allowed) {
  requireThat(value !== null && typeof value === 'object' && !Array.isArray(value), 'INVALID_RECORD');
  requireThat([Object.prototype, null].includes(Object.getPrototypeOf(value)), 'INVALID_RECORD');
  const descriptors = Object.getOwnPropertyDescriptors(value);
  requireThat(Reflect.ownKeys(descriptors).every(key => typeof key === 'string' && allowed.includes(key)
    && 'value' in descriptors[key] && descriptors[key].enumerable), 'INVALID_FIELDS');
  requireThat(required.every(key => Object.hasOwn(descriptors, key)), 'MISSING_FIELD');
  return value;
}
export function array(value, maximum) {
  requireThat(Array.isArray(value) && Object.getPrototypeOf(value) === Array.prototype
    && value.length <= maximum, 'INVALID_ARRAY');
  const fields = Object.getOwnPropertyDescriptors(value);
  requireThat(Reflect.ownKeys(fields).length === value.length + 1
    && Object.keys(fields).every(key => key === 'length' || (/^(0|[1-9][0-9]*)$/.test(key)
      && Number(key) < value.length && 'value' in fields[key] && fields[key].enumerable)), 'INVALID_ARRAY');
  return value;
}
export function freeze(value) {
  if (value && typeof value === 'object') {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
export const snapshot = value => freeze(structuredClone(value));
export function text(value, pattern, max, code = 'INVALID_TEXT') {
  requireThat(typeof value === 'string' && value.length <= max && pattern.test(value), code);
  return value;
}
export const mockId = value => text(value, /^mock:[a-z0-9:-]+$/, 80, 'INVALID_MOCK_ID');
export const syntheticId = value => text(value, /^synthetic:[a-z0-9:-]+$/, 80, 'INVALID_SYNTHETIC_ID');
export function amount(value) {
  text(value, /^[1-9][0-9]{0,19}$/, 20, 'INVALID_AMOUNT');
  requireThat(BigInt(value) <= 18_446_744_073_709_551_615n, 'AMOUNT_TOO_LARGE');
  return value;
}
export const digest = value => createHash('sha256').update(JSON.stringify(value)).digest('hex');
const principals = new WeakSet();
const permissions = ['donation:submit', 'donation:read', 'registry:manage'];
export function createLocalPrincipal(input) {
  record(input, ['subject', 'permissions']);
  mockId(input.subject);
  array(input.permissions, permissions.length);
  requireThat(input.permissions.every(p => permissions.includes(p))
    && new Set(input.permissions).size === input.permissions.length, 'INVALID_PERMISSIONS');
  const principal = snapshot({ subject: input.subject, permissions: input.permissions, authentication: 'local-fixture-only' });
  principals.add(principal);
  return principal;
}
export function authorize(principal, permission) {
  requireThat(principals.has(principal), 'UNAUTHENTICATED');
  requireThat(principal.permissions.includes(permission), 'FORBIDDEN');
  return principal.subject;
}
export function attribution(value) {
  record(value, ['mode', 'label'], ['mode']);
  requireThat(value.mode === 'anonymous' || value.mode === 'named', 'INVALID_ATTRIBUTION');
  if (value.mode === 'anonymous') {
    requireThat(!Object.hasOwn(value, 'label'), 'ANONYMOUS_LABEL_FORBIDDEN');
    return freeze({ mode: 'anonymous' });
  }
  text(value.label, /^[^\p{Cc}\p{Cf}]+$/u, 64, 'INVALID_ATTRIBUTION_LABEL');
  requireThat(value.label.trim() === value.label, 'INVALID_ATTRIBUTION_LABEL');
  return freeze({ mode: 'named', label: value.label });
}
