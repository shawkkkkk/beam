# Local charity registry and donation adapter contract

Scope and base: `backend/charity/**` on Beam
`b8120a8a58b1375ba3b8f0d2a590c49923e18021`. This document defines the proposed
interface and acceptance behavior before implementation. It is an off-chain
local simulation contract, not an observed Anchor/donation-relay ABI, Donate.gg
integration, deployed service, authorization of a real charity, or a completed
charity feature. No contract, fee policy, wallet, signing, HTTP or transfer path
is part of this lane. Beam's new-code licensing remains a maintainer decision;
this change does not adopt the archive's license for unrelated new files.

## Data source and capability boundary

The only catalog source is a bounded in-process synthetic fixture registry.
Every record has a `synthetic:` charity/recipient ID, fixture-only display name,
explicit `mock-verified` provenance, fixture issuer and immutable revision.
Those labels establish synthetic test provenance only. They do not establish
real-world charity status, recipient control, provider access or permission.
Registry changes require a local fixture operator principal and create a new
immutable revision. Unverified/revoked records cannot receive new submissions.

The service is disabled by default. Enabling it permits only the factory-created
local mock provider; an arbitrary injected object, a provider-name string, a
missing adapter or a spoofed `kind: mock` does not enable it. The provider factory
is immutable and internally branded. Its outcomes are bounded plain fixture
data, never executable hooks. It has no network, credentials, signing or wallet
capability. Live donation capability is always unavailable.

## Proposed module interface

- `createLocalPrincipal({ subject, permissions })` creates an immutable branded
  **fixture principal**. The trusted caller passes it separately from request
  data; JSON objects claiming the same fields are not principals. This is local
  authorization testing, not production authentication.
- `createSyntheticRegistry()` exposes immutable fixture records and a local
  operator-only status revision operation. Donors cannot administer the registry.
- `createMockDonationProvider({ submitPlan, lookupPlan, delayTicks })` simulates
  asynchronous settlement with a private idempotency ledger and inspectable
  immutable test snapshots. No caller-supplied function is an outcome.
- `createCharityService({ enabled = false, registry, provider, clock })` exposes
  `capabilities()`, `listCharities()`, `prepare()`, `submit()`, `reconcile()` and
  owner-authorized `receipt()`/`history()` reads. There is no server or route.

`prepare({ principal, idempotencyKey, charityId, amountBaseUnits, attribution })`
requires donation permission and creates an immutable `awaiting_consent`
receipt containing the exact proposed terms and their digest. Repeating a key
for the same principal and identical terms returns the same donation; changing
any amount, charity, provider/recipient binding, registry revision or attribution
under that key conflicts. Different principals have separate key namespaces.

Amounts are canonical positive decimal **strings**, parsed with `BigInt`, with
a chosen per-donation bound of `2^64-1`. Numbers, fractional/exponential text,
leading zeroes, signs, whitespace and zero reject. The unit is `MOCK-UNIT`;
this is a proposed local schema limit, not an assertion about an on-chain ABI.
No floating-point conversion or fee deduction occurs.

Attribution is either `{ mode: anonymous }` or `{ mode: named, label }`.
Named attribution requires an explicit bounded label; anonymous mode forbids a
label. Terms bind the principal, attribution, exact amount, fixture currency,
registry revision, charity, provider and recipient. Receipt responses do not
publish the owner identity as attribution. All reads remain owner-authorized;
there is no public donor-search or publication endpoint.

`submit({ principal, donationId, consent })` requires the owner, donation
permission and exact consent `{ accepted: true, version, termsDigest }`. Consent
is bound to the complete prepared terms; a generic accepted flag is insufficient.
The service rechecks current recipient/provenance and provider binding before
each new submission. A revoked/revised recipient cannot silently redirect a
retry. Accepted consent is recorded with the local clock in immutable receipts.

## Asynchronous attempts and immutable receipts

Only one provider operation may be in flight per donation. Concurrent duplicate
submissions share that operation and result. Receipts are append-only immutable
snapshots with sequence, mock mode, exact terms binding, attempt counts and
attribution. The initial receipt and earlier attempt snapshots never change.
`atMs` is the operation-start timestamp from a trusted local clock, not an
external settlement timestamp; pending and result snapshots share that stamp.
Recipient binding is rechecked at the provider handoff after queueing. Revocation
after a handoff does not imply cancellation of an already accepted attempt.

The mock provider supports settled, definitely-not-accepted transient failure,
rejection, unavailable and unknown outcomes. Unknown can occur before acceptance
or after a private mock ledger entry was created. A settled receipt is accepted
only when its provider, recipient, amount, currency, terms digest and idempotency
binding all match. A malformed/mismatched settlement becomes unresolved; it is
not reported as settled and does not permit a blind resubmission.

An unknown outcome requires `reconcile({ principal, donationId })` before another
submission. Reconciliation only looks up the same provider/key/binding; it does
not submit. Unknown/unavailable lookup remains unresolved. A definitive mock
`not_found` permits an explicit later retry; a matching ledger entry produces
`mock_settled`. Revocation cannot prevent reading/reconciling a prior outcome,
but still prevents a new submission. The mock ledger enforces its own key and
fingerprint idempotency, including lost-response recovery.

Configured hard bounds cover fixture records, input lengths, donation count,
attempts and per-donation receipt history. Exhausted limits preserve uncertainty
and existing receipts; they do not infer success/failure. Process restart,
distributed concurrency, durable persistence and real provider finality are
not solved by this in-memory module. They are blockers to a live integration.

## Local acceptance

Use the official Node 22 runtime and no downloaded dependencies:

```sh
node --test backend/charity/*.test.mjs
python3 scripts/check_fork.py
git diff --check
```

Tests must exercise the implemented async service and private mock ledger:

- Default disabled, absent provider and spoofed/live provider rejection, with
  zero provider calls or external actions.
- Synthetic registry provenance, administrator denial, recipient revocation and
  exact provider/recipient binding; no real charity fixtures.
- Canonical amounts including maximum and unsafe-number/overflow rejection;
  consent/version/digest and named/anonymous attribution boundaries.
- Owner and permission denial, forged fixture principal rejection and immutable
  request/receipt snapshots.
- Concurrent duplicates, conflicting key reuse, transient retry, rejection,
  unknown before/after mock acceptance, repeated unresolved lookup and eventual
  reconciliation without duplicate ledger effects.
- Mismatched provider receipts, input/storage/attempt limits, and no blind retry
  after an uncertain outcome. Counter/snapshot assertions must verify actual
  mock calls/effects, not merely expected return objects.

The draft PR will report actual results and remaining boundaries after an
independent peer review. This lane does not activate Donate.gg, archived relay
instructions, creator-fee routing, charity disbursement or any live service.
