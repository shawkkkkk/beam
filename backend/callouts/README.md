# Local callout service contract

This is a proposed Beam product interface, independently implemented on base
`b8120a8a58b1375ba3b8f0d2a590c49923e18021`. No on-chain callout ABI or original
Pump service implementation is evidenced. This module supplies local function
calls and synthetic mock records; it opens no socket and performs no remote,
wallet, payment, signing, or display action. It does not integrate the website.
The repository's new-code license remains a maintainer question; this lane
does not change or infer licensing from the archived source.

## Interface and trust boundary

`createCalloutService({ auth, moderation, clock, channels, limits })` returns
an in-memory service. All operations are asynchronous. `channels` is an explicit
local list of synthetic channel identifiers, not verified mint or chain data.
Every returned record carries `source: "synthetic-local"` and
`verifiedOnChain: false`. Restarting the process loses its records.

The caller supplies a session identifier. Only the injected trusted
`auth.authenticate(session)` may establish its subject, channel grants and
session expiry. Request fields cannot set an author, reviewer, approval, or
permission. Grants name exact channel/action pairs: `submit`, `read-own`,
`moderate`, or `display`. Unknown sessions, expired sessions, unknown channels,
missing grants and malformed adapter responses are denied. Missing/unavailable
auth, moderation or clock fails closed, including reads; adapter exceptions are
reported with a generic error code and never exposed as provider messages.

The trusted moderation adapter provides `available()` and
`screen({ channelId, authorId, text })`. Screening returns an explicit allow
decision and policy version. Submission screening does not approve display:
every accepted submission begins pending. Approval screens the immutable text
again and rechecks authorization after the asynchronous dependency work.
Identity changes during an operation are denied. Trusted adapter snapshots,
not caller-provided roles, are the authority; adapters must settle their calls.

`clock()` returns integer epoch milliseconds. It must be nondecreasing.
Expiry is computed from the current clock after dependency checks on every
read and write; neither idempotent retries nor stale returned objects can
extend display. Clock failure, rollback or unsafe timestamps deny the operation.

## Proposed operations

- `submit({ session, channelId, text, durationMs, idempotencyKey })` requires
  `submit`. Text is normalized to NFC and trimmed; it must be a single line,
  at most 280 Unicode code points and 1,024 UTF-8 bytes. Angle brackets,
  controls and directional override/isolation characters are rejected.
  No HTML or rendered markup is returned. Duration is 1–300 seconds.
- `claimReview({ session, channelId, id, expectedRevision, idempotencyKey })`
  requires `moderate`. Only a pending, unexpired record can be claimed; the
  author cannot claim their own submission. The first reviewer owns it.
- `moderate({ session, channelId, id, expectedRevision, decision,
  startsAt, reason, idempotencyKey })` requires `moderate` and review ownership.
  The decision is approve or reject. Approval starts at the current time or
  an explicit time within the next hour; it lasts the submitted duration.
  Rejection uses a fixed reason code, not unbounded moderator HTML/text.
  Terminal decisions cannot be changed by another request.
- `getOwn({ session, channelId, id })` and
  `listOwn({ session, channelId, limit })` require `read-own` and expose only
  that subject's records. Reviewer identity is omitted from author views.
- `listReviewQueue({ session, channelId, limit })` requires `moderate` and
  lists only pending, unexpired records for that channel, with review ownership.
- `queryDisplay({ session, channelId, limit })` requires `display`. It returns
  only approved records whose window contains the current time. It never
  returns pending/rejected/expired text or private moderation metadata.

Every mutation requires a bounded idempotency key, scoped to actor, channel and
operation. Retrying the same canonical request returns a fresh current-state
snapshot without a second write, new ID, rate charge or display extension.
Reusing its key with different input is a conflict. Authorization and adapter
availability are checked even on retries. Expected revisions reject competing
updates; mutations are serialized locally so concurrent review claims have one
owner. This is a single-process boundary, not a distributed persistence claim.

Defaults bound accepted submissions to 3 per actor/channel/minute and review
actions to 20 per reviewer/channel/minute. Rejected, unavailable, stale or expired
requests do not consume quota, and idempotent successful retries are free.
Limits and timestamps are validated. A maximum of 200 records bounds records
and successful-mutation idempotency entries; a separate cap of three rate
buckets per record-capacity slot bounds active successful-action histories;
capacity failure preserves existing records. There is no paid priority,
automatic publication, deletion, reviewer transfer or eviction/replay policy.

## Display and accessibility data

Pending submissions expire after 15 minutes. Approval/rejection is unavailable
at that boundary. Approved records are `scheduled` before `startsAt`,
`displaying` on `[startsAt, endsAt)`, and `expired` at `endsAt`. Rejected records
remain rejected for the author's history and are never display eligible.
Stored moderation decisions and current display states are distinct.

Query results are deterministic: display orders by start time then stable ID;
review queue by creation time then ID; own history by newest creation then ID.
Limits are integers from 1 to 50. Returned records and arrays are detached
copies. Views include plain-text status labels and `role: "status"` /
`ariaLive: "polite"` data; consumers must render text as text, not HTML.
Empty queries have a clear status label. Service errors have stable codes
and plain-text messages, suitable for a separate UI's unavailable/error state.

## Local acceptance

Before release, local tests must exercise the complete submit → claim →
approve → scheduled → displaying → expired workflow, rejection, author-only
history, channel permissions, self-review denial, competing moderator claims,
stale revisions, idempotent retries/conflicts and rate/capacity boundaries.
Tests must also cover unavailable/malformed dependencies, changed auth during
screening, expiry while awaiting a dependency, returned-object mutation,
clock rollback, malicious/oversized text and deterministic bounded queries.
Fixtures and adapters must be unmistakably synthetic and require no secrets.

Run from the repository root:

```sh
node --test backend/callouts/service.test.mjs
python3 scripts/check_fork.py
git diff --check
```

The code targets Node 22 or newer with built-in modules only. Actual tested
runtime and results are recorded with the change; no hosted CI, real provider,
live display, original-service parity or whole-product completion is implied.

The implemented local boundary passes 17 tests on the session's verified
Node 22.23.4 runtime. `check_fork.py` verifies all 380 archived blobs, 62 retained
byte-identical files and the four existing declared contract differences;
the staged whitespace check also passes. An independent charity-lane peer ran
the same suite and additional adapter-getter probes. Production authentication,
moderation policy, persistence, a transport and the UI remain separate work.
