const ID = /^[A-Za-z0-9][A-Za-z0-9:_-]{0,63}$/;
const TERMINAL = new Set(['ended', 'banned', 'failed']);
const CAPABILITY = { start: 'stream:start', stop: 'stream:stop', watch: 'stream:watch', moderate: 'stream:moderate' };
const DEFAULTS = { maxStreams: 64, maxPendingPerStream: 8, maxOperations: 16, maxEvents: 16,
  maxLifetimeMs: 3_600_000, terminalRetentionMs: 60_000 };
const validId = value => typeof value === 'string' && ID.test(value);

function admittedProvider(provider) {
  try {
    if (provider?.kind !== 'local-mock') return null;
    const methods = {};
    for (const key of ['start', 'stop', 'watch']) {
      const method = provider[key];
      if (typeof method !== 'function') return null;
      methods[key] = method.bind(provider);
    }
    return methods;
  } catch { return null; }
}

/** Local-only policy controller. It contains no network or player integration. */
export function createLivestreamService({ authorize, resolveCreator, provider, clock = Date.now, limits = {} } = {}) {
  const policy = { ...DEFAULTS, ...limits };
  const maxima = { maxStreams: 1024, maxPendingPerStream: 64, maxOperations: 128, maxEvents: 128,
    maxLifetimeMs: 86_400_000, terminalRetentionMs: 86_400_000 };
  for (const [key, value] of Object.entries(policy)) {
    if (!Object.hasOwn(maxima, key) || !Number.isSafeInteger(value) || value < 1 || value > maxima[key]) throw new TypeError('Invalid local retention limit');
  }
  if (typeof clock !== 'function') throw new TypeError('A local clock function is required');
  const media = admittedProvider(provider);
  const records = new Map();
  let lastTime = 0;
  let clockAvailable = true;

  function now() {
    try {
      const value = clock();
      if (!Number.isSafeInteger(value) || value < 0 || value > Number.MAX_SAFE_INTEGER - 86_400_000) throw new TypeError('Invalid local clock');
      lastTime = Math.max(lastTime, value);
      clockAvailable = true;
    } catch {
      // Closing existing work must still settle safely when an injected clock
      // fails. New starts are denied; no error text or invalid time is stored.
      clockAvailable = false;
    }
    return lastTime;
  }
  function event(record, type) {
    const at = now();
    if (!clockAvailable && ['starting', 'live'].includes(record.state)) record.clockFailed = true;
    record.events.push({ type, at });
    if (record.events.length > policy.maxEvents) record.events.shift();
  }
  function cleanup(record, sessionId) {
    if (record.cleanupPromise) return record.cleanupPromise;
    record.sessionId = sessionId;
    record.cleanup = 'pending';
    record.pending += 1;
    record.cleanupPromise = Promise.resolve().then(() => {
      if (!media) throw new Error('Unavailable local adapter');
      return media.stop({ streamId: record.id, sessionId });
    }).then(() => {
      record.sessionId = null;
      record.cleanup = 'done';
      event(record, 'cleanup_done');
      return true;
    }, () => {
      record.cleanup = 'failed';
      event(record, 'cleanup_failed');
      return false;
    }).finally(() => { record.pending -= 1; });
    return record.cleanupPromise;
  }
  function close(record, state) {
    if (record.state === 'expired') return Promise.resolve(true);
    if (record.state !== 'banned' && (!TERMINAL.has(record.state) || state === 'banned')) {
      record.epoch += 1;
      record.abort?.abort();
      record.state = state;
      record.terminalAt ??= now();
      event(record, state);
    }
    return record.sessionId ? cleanup(record, record.sessionId) : Promise.resolve(true);
  }
  function prune() {
    const time = now();
    for (const [id, record] of records) {
      if (record.state === 'expired') continue;
      if (!clockAvailable && ['starting', 'live'].includes(record.state)) {
        record.clockFailed = true;
        void close(record, 'failed');
      }
      if (!TERMINAL.has(record.state) && time - record.createdAt >= policy.maxLifetimeMs) void close(record, 'ended');
      if (TERMINAL.has(record.state) && record.pending === 0 && time - record.terminalAt >= policy.terminalRetentionMs) {
        // Bounded safety tombstones prevent ID reuse after metadata deletion.
        records.set(id, { id, state: 'expired' });
      }
    }
  }
  function reserve(id) {
    prune();
    let record = records.get(id);
    if (record?.state === 'expired') return { error: 'stream_expired' };
    if (!record) {
      if (records.size >= policy.maxStreams) return { error: 'capacity_exceeded' };
      record = { id, creatorId: null, state: 'idle', epoch: 0, createdAt: now(), terminalAt: null,
        pending: 0, requests: 0, sessionId: null, cleanup: 'not_needed', cleanupPromise: null,
        abort: null, clockFailed: false, operations: new Map(), events: [] };
      records.set(id, record);
    }
    if (record.requests >= policy.maxPendingPerStream) return { error: 'capacity_exceeded' };
    record.pending += 1;
    record.requests += 1;
    return { record, epoch: record.epoch };
  }
  function release(record) {
    record.pending -= 1;
    record.requests -= 1;
    if (record.state === 'idle' && record.pending === 0 && record.events.length === 0 && record.operations.size === 0) records.delete(record.id);
    prune();
  }
  function view(record, playbackReason) {
    const current = records.get(record.id);
    if (current?.state === 'expired') return null;
    const reasons = { idle: 'not_started', starting: 'starting', live: 'local_mock_only', ended: 'ended', banned: 'banned', failed: 'provider_unavailable' };
    return { id: record.id, creatorId: record.creatorId, state: record.state, localOnly: true, clockAvailable,
      source: 'local-fixture', cleanup: record.cleanup,
      playback: { available: false, reason: record.state === 'live' ? playbackReason ?? reasons[record.state] : reasons[record.state] },
      createdAt: record.createdAt, terminalAt: record.terminalAt,
      retentionExpiresAt: record.terminalAt === null ? null : record.terminalAt + policy.terminalRetentionMs,
      events: record.events.map(item => ({ ...item })) };
  }
  async function response(record, outcome, replayed = false, action) {
    prune();
    if (action === 'start' && record.clockFailed) {
      await close(record, 'failed');
      outcome = { ok: false, code: 'clock_unavailable' };
    } else if (action === 'start' && outcome.ok && !replayed && record.state !== 'live') {
      outcome = { ok: false, code: 'superseded' };
    }
    return { ok: outcome.ok, code: outcome.code, replayed, localOnly: true,
      stream: record ? view(record, outcome.playbackReason) : null };
  }
  const rejected = code => ({ ok: false, code, replayed: false, localOnly: true, stream: null });

  async function identity(actor, action, streamId) {
    if (typeof authorize !== 'function') return { error: 'authorization_unavailable' };
    let grant;
    try { grant = await authorize(actor, { action, streamId }); } catch { return { error: 'authorization_unavailable' }; }
    if (!grant || !validId(grant.subject) || !Array.isArray(grant.permissions) || grant.permissions.length > 32 ||
        !grant.permissions.every(item => typeof item === 'string' && item.length <= 64)) return { error: 'permission_denied' };
    if (!grant.permissions.includes(CAPABILITY[action])) return { error: 'permission_denied' };
    return { subject: grant.subject };
  }
  async function bind(record) {
    if (typeof resolveCreator !== 'function') return false;
    let binding;
    try { binding = await resolveCreator(record.id); } catch { return false; }
    if (!binding || binding.source !== 'local-fixture' || !validId(binding.creatorId)) return false;
    if (record.creatorId !== null && record.creatorId !== binding.creatorId) return false;
    record.creatorId ??= binding.creatorId;
    return true;
  }
  async function operation(record, requestId, action, subject, work) {
    const existing = record.operations.get(requestId);
    if (existing) {
      if (existing.action !== action || existing.subject !== subject) return { outcome: { ok: false, code: 'idempotency_conflict' } };
      return { outcome: await existing.promise, replayed: true };
    }
    if (record.operations.size >= policy.maxOperations) {
      const completed = [...record.operations].find(([, item]) => item.done);
      if (!completed) return { outcome: { ok: false, code: 'capacity_exceeded' } };
      record.operations.delete(completed[0]);
    }
    const entry = { action, subject, done: false, promise: null };
    entry.promise = Promise.resolve().then(work).finally(() => { entry.done = true; });
    record.operations.set(requestId, entry);
    return { outcome: await entry.promise, replayed: false };
  }
  async function run(input, action, work, command = true) {
    let request;
    try {
      // Copy all caller-owned primitive fields once, before the first await.
      request = { streamId: input?.streamId, actor: input?.actor, requestId: input?.requestId, decision: input?.decision };
    } catch { return rejected('invalid_request'); }
    if (!validId(request.streamId) || !validId(request.actor) || (command && !validId(request.requestId)) ||
        (action === 'moderate' && !['ban', 'stop', 'ended'].includes(request.decision))) return rejected('invalid_request');
    const reservation = reserve(request.streamId);
    if (reservation.error) return rejected(reservation.error);
    const { record, epoch } = reservation;
    let completed;
    try {
      if (action === 'start' && !clockAvailable) return rejected('clock_unavailable');
      const grant = await identity(request.actor, action, request.streamId);
      if (grant.error) return rejected(grant.error);
      if (!await bind(record)) return rejected('creator_unavailable');
      if ((action === 'start' || action === 'stop') && grant.subject !== record.creatorId) return rejected('creator_mismatch');
      prune();
      if (action === 'start' && !clockAvailable) return rejected('clock_unavailable');
      const result = command
        ? await operation(record, request.requestId, action === 'moderate' ? `moderate:${request.decision}` : action, grant.subject, () => work(record, epoch, request))
        : { outcome: await work(record, epoch, request), replayed: false };
      completed = result;
    } catch {
      if (action === 'start' && record.state === 'starting') void close(record, 'failed');
      return rejected('local_operation_failed');
    } finally { release(record); }
    // All release/retention work happens before the public state snapshot.
    return response(record, completed.outcome, completed.replayed, action);
  }

  function maintenanceStats() {
    const retained = [...records.values()].filter(record => record.state !== 'expired');
    return { identities: records.size, retainedRecords: retained.length,
      pendingRequests: retained.reduce((sum, record) => sum + record.requests, 0),
      eventRecords: retained.reduce((sum, record) => sum + record.events.length, 0),
      idempotencyEntries: retained.reduce((sum, record) => sum + record.operations.size, 0) };
  }
  return Object.freeze({
    start(input) {
      return run(input, 'start', async (record, epoch) => {
        if (record.epoch !== epoch || TERMINAL.has(record.state)) return { ok: false, code: 'superseded' };
        if (record.state !== 'idle') return { ok: false, code: 'stream_busy' };
        if (!media) return { ok: false, code: 'provider_unavailable' };
        record.state = 'starting';
        record.abort = new AbortController();
        event(record, 'starting');
        if (!clockAvailable || record.clockFailed) {
          void close(record, 'failed');
          return { ok: false, code: 'clock_unavailable' };
        }
        let result;
        try { result = await media.start({ streamId: record.id, creatorId: record.creatorId, signal: record.abort.signal }); }
        catch {
          if (record.epoch !== epoch || record.state !== 'starting') return { ok: false, code: 'superseded' };
          void close(record, 'failed');
          return { ok: false, code: 'provider_unavailable' };
        }
        prune();
        if (!result || !validId(result.sessionId)) {
          if (record.epoch === epoch && record.state === 'starting') void close(record, 'failed');
          return { ok: false, code: 'provider_protocol_error' };
        }
        record.sessionId = result.sessionId;
        if (!clockAvailable || record.clockFailed) {
          void close(record, 'failed');
          await cleanup(record, result.sessionId);
          return { ok: false, code: 'clock_unavailable' };
        }
        if (record.epoch !== epoch || record.state !== 'starting') {
          await cleanup(record, result.sessionId);
          return { ok: false, code: 'superseded' };
        }
        record.state = 'live';
        event(record, 'live');
        return { ok: true, code: 'applied' };
      });
    },
    stop(input) {
      return run(input, 'stop', async record => ({ ok: await close(record, 'ended'), code: record.cleanup === 'failed' ? 'provider_stop_failed' : 'applied' }));
    },
    moderate(input) {
      return run(input, 'moderate', async (record, _epoch, request) => ({ ok: await close(record, request.decision === 'ban' ? 'banned' : 'ended'),
        code: record.cleanup === 'failed' ? 'provider_stop_failed' : 'applied' }));
    },
    watch(input) {
      return run(input, 'watch', async record => {
        if (record.state !== 'live') return { ok: true, code: 'observed' };
        const epoch = record.epoch;
        const sessionId = record.sessionId;
        let result;
        try { result = await media.watch({ streamId: record.id, sessionId }); } catch {
          if (record.epoch !== epoch || record.state !== 'live') return { ok: true, code: 'observed' };
          return { ok: false, code: 'provider_unavailable', playbackReason: 'provider_unavailable' };
        }
        prune();
        if (record.epoch !== epoch || record.state !== 'live') return { ok: true, code: 'observed' };
        if (!result || typeof result.ready !== 'boolean') return { ok: false, code: 'provider_protocol_error', playbackReason: 'provider_unavailable' };
        return { ok: true, code: 'observed', playbackReason: result.ready ? 'local_mock_only' : 'provider_not_ready' };
      }, false);
    },
    async sweep() {
      prune();
      await Promise.all([...records.values()].map(record => record.cleanupPromise).filter(Boolean));
      prune();
      return maintenanceStats();
    },
    maintenanceStats,
  });
}

/** Bundled mock acknowledges lifecycle only; it never supplies playback. */
export function createMockMediaProvider() {
  let sequence = 0;
  return Object.freeze({ kind: 'local-mock',
    async start() { sequence += 1; return { sessionId: `mock-session-${sequence}` }; },
    async stop() {},
    async watch() { return { ready: true }; },
  });
}
