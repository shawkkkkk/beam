# Local livestream permissions and moderation contract

This is a proposed Beam backend interface, independently implemented from the
product requirements in `docs/TASKS.md` and `docs/FEATURES.md`. It is not an
observed Pump service, media API, on-chain ABI, deployed service, or player.
Base: `b8120a8a58b1375ba3b8f0d2a590c49923e18021`.

## Interface and data sources

`createLivestreamService({ authorize, resolveCreator, provider, clock, limits })`
returns asynchronous `start`, `stop`, `watch`, `moderate`, and `sweep` methods.
Commands use `{ streamId, actor, requestId }`; `watch` omits `requestId` and
moderation adds `decision: 'ban' | 'stop' | 'ended'`. Identifiers are bounded
opaque fixture strings. There is no HTTP listener or account creation.
Command primitive fields are copied before any asynchronous authorization, so
caller mutation cannot change the authorized target, decision or retry key.

The injected local fixture catalog `resolveCreator(streamId)` returns
`{ creatorId, source: 'local-fixture' }`. The first authorized operation binds
the stream to that creator; later catalog disagreement fails closed while the
lifecycle record is retained. Unused watch-only or denied idle reservations may
be discarded; they do not establish a persistent creator binding. A caller
cannot choose a creator through command data. This catalog does not establish
chain ownership or finality; connecting the indexer is future integration work.

The trusted injected mock `authorize(actor, { action, streamId })` returns
`{ subject, permissions }`. Start/stop require the respective `stream:start` or
`stream:stop` permission and the bound creator identity. Watch requires
`stream:watch`. Moderation requires `stream:moderate` and can stop or ban another
creator's stream. Missing, rejected, malformed, or denied authorization fails
closed. No wallet verification or production identity provider is supplied.
Actors are local fixture handles, not credentials, cookies, signatures or keys.

Only an adapter explicitly marked `kind: 'local-mock'` is admitted. Its async
methods are `start({ streamId, creatorId, signal }) -> { sessionId }`,
`stop({ streamId, sessionId })`, and `watch({ streamId, sessionId }) -> { ready }`.
The bundled adapter performs no network, media, disk, wallet or financial action.
Private session handles stay inside the controller. Provider errors and extra
provider fields are never copied into public results or event history.

## State, permissions and availability

The lifecycle is `idle -> starting -> live`, followed by terminal `ended`,
`banned`, or `failed`. Here `live` means only that the local mock acknowledged
start. Every result says `localOnly: true` and playback is always unavailable;
there is no playback URL, stream key, live player or real broadcast claim.
Watch reports the current lifecycle and a reason such as `starting`, `ended`,
`banned`, `provider_unavailable`, `provider_not_ready`, or `local_mock_only`.

Stop and moderation invalidate pending starts before awaiting provider cleanup.
A late successful start is stopped as an orphan and cannot restore a terminal
state. A watch response arriving after stop/ban is discarded in favor of the
current unavailable state. Ban is stronger than stop/ended; later completions
cannot downgrade it. Terminal stream IDs cannot be restarted in this service
instance. These are local policy rules, not claims about a provider's live state.

Request IDs deduplicate authorized command retries. Their scope includes action
and authenticated subject; conflicting reuse is rejected. Cached outcomes are
combined with the current redacted state, so a start retry cannot return a stale
live snapshot after a stop. Different stop requests share one cleanup promise.
Cleanup failure remains visible as `cleanup: 'failed'`; it does not reopen the
stream or imply that an external resource was successfully stopped.

## Retention and bounds

All limits are explicit and bounded: stream identities per instance, concurrent
requests, completed idempotency entries, and event records per stream. Active
records expire after `maxLifetimeMs`; completed terminal metadata is removed
after `terminalRetentionMs` on the next call or explicit `sweep()`.
Minimal expired-ID tombstones are retained within the same identity cap. An
expired ID cannot be reused, including a previously banned ID. Once that cap is
full, new identities fail closed; an instance reset is an explicit local reset,
not a production ban persistence mechanism.

Pending injected promises keep a bounded slot until they settle; their responses
remain canceled after stop/ban/expiry. This local contract has no provider
watchdog, durable storage, webhook authentication, retry worker or cross-process
idempotency. Production integration must supply those facilities and durable
moderation before release. `sweep()` is an internal maintenance method, not an
unauthenticated network endpoint. No timers or live services are launched.
`maintenanceStats()` exposes only bounded aggregate counts for local tests and
host maintenance, with no session handles, identities or event payloads.

Defaults are 64 stream identities, eight pending requests per stream, 16 cached
operations and 16 history entries per stream, one-hour maximum active lifetime,
and one-minute terminal metadata retention. IDs are single-use per instance;
the bounded tombstones are deliberately not evicted to make room for new IDs.
An injected clock fault denies new starts. Cleanup and terminal transitions
still settle using the last valid monotonic time; snapshots explicitly report
`clockAvailable: false`. A clock fault observed during start, live-transition or
final response/release closes the local session and waits for cleanup before a
start result is published. Expiry resumes on a later valid clock observation.

## Local acceptance

Run with Node 22, without installation or dependencies:

```sh
node --test backend/livestream/livestream.test.mjs
python3 scripts/check_fork.py
git diff --check
```

Tests must cover creator binding and capability denial, missing/malformed
adapters, private-field/error redaction, start/stop retries, provider failures,
deferred start followed by stop/ban/expiry, deferred watch followed by ban,
late stop completion after ban, bounded history/cache/capacity and expiry.
Providers are controlled local mocks, including deliberately deferred promises.
No stream, provider account, wallet, transaction, donation, signing or deployment
is enabled. This lane changes no contract, archive, global documentation,
licensing, SDK or website file. Beam's unresolved new-code licensing terms
remain a maintainer decision.

The implementation and tests use only Node built-ins. There are no packages,
install hooks, provider credentials, recordings or playback URLs to configure.
