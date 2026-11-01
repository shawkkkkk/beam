import test from 'node:test';
import assert from 'node:assert/strict';
import { createLivestreamService, createMockMediaProvider } from './livestream.mjs';

const OWNER = ['stream:start', 'stream:stop', 'stream:watch'];
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
function setup(options = {}) {
  const calls = { start: [], stop: [], watch: [] };
  const principals = new Map([
    ['owner', { subject: 'creator-a', permissions: OWNER }],
    ['other', { subject: 'creator-b', permissions: OWNER }],
    ['viewer', { subject: 'viewer-a', permissions: ['stream:watch'] }],
    ['moderator', { subject: 'moderator-a', permissions: ['stream:moderate', 'stream:watch'] }],
    ['denied', { subject: 'creator-a', permissions: [] }],
  ]);
  let time = 0;
  const provider = { kind: 'local-mock',
    async start(input) { calls.start.push(input); return { sessionId: 'mock-session' }; },
    async stop(input) { calls.stop.push(input); },
    async watch(input) { calls.watch.push(input); return { ready: true }; },
  };
  Object.assign(provider, options.provider);
  const service = createLivestreamService({
    authorize: async actor => principals.get(actor),
    resolveCreator: async () => ({ creatorId: 'creator-a', source: 'local-fixture' }),
    provider, clock: () => time,
    ...options,
    provider: options.provider === null ? null : provider,
  });
  return { service, calls, principals, advance: amount => { time += amount; } };
}
const command = (requestId = 'request-a', actor = 'owner', streamId = 'stream-a') => ({ streamId, actor, requestId });
const watch = (service, actor = 'viewer', streamId = 'stream-a') => service.watch({ streamId, actor });

test('only a permitted bound creator can start or stop; a viewer can watch', async () => {
  const { service, calls } = setup();
  assert.equal((await service.start(command('a', 'denied'))).code, 'permission_denied');
  assert.equal((await service.start(command('b', 'viewer'))).code, 'permission_denied');
  assert.equal((await service.start(command('c', 'other'))).code, 'creator_mismatch');
  assert.equal(calls.start.length, 0);
  const started = await service.start(command('d'));
  assert.equal(started.stream.state, 'live');
  assert.equal(started.stream.creatorId, 'creator-a');
  assert.equal((await service.stop(command('e', 'other'))).code, 'creator_mismatch');
  assert.equal((await watch(service)).stream.playback.reason, 'local_mock_only');
  assert.equal((await watch(service, 'denied')).stream, null);
});

test('the bundled provider is a local lifecycle mock with no playable URL', async () => {
  const service = createLivestreamService({ authorize: async () => ({ subject: 'creator-a', permissions: OWNER }),
    resolveCreator: async () => ({ creatorId: 'creator-a', source: 'local-fixture' }), provider: createMockMediaProvider() });
  await service.start(command());
  const result = await service.watch({ streamId: 'stream-a', actor: 'owner' });
  assert.equal(result.localOnly, true);
  assert.deepEqual(result.stream.playback, { available: false, reason: 'local_mock_only' });
  assert.equal(JSON.stringify(result).includes('sessionId'), false);
  assert.equal('url' in result.stream.playback, false);
});

test('missing authorization, provider and creator provenance fail closed', async () => {
  assert.equal((await createLivestreamService().start(command())).code, 'authorization_unavailable');
  const missing = setup({ provider: null });
  assert.equal((await missing.service.start(command())).code, 'provider_unavailable');
  for (const provider of [{ kind: 'live-provider' }, { kind: 'local-mock', start: null }]) {
    const item = setup({ provider });
    assert.equal((await item.service.start(command())).code, 'provider_unavailable');
  }
  for (const binding of [null, { creatorId: 'creator-a', source: 'unverified-chain' }, { creatorId: 123, source: 'local-fixture' }]) {
    const item = setup({ resolveCreator: async () => binding });
    assert.equal((await item.service.start(command())).code, 'creator_unavailable');
    assert.equal(item.calls.start.length, 0);
  }
});

test('malformed grants and requests are denied without coercion', async () => {
  for (const grant of [null, {}, { subject: 123, permissions: OWNER }, { subject: 'creator-a', permissions: 'stream:start' }]) {
    const { service, calls } = setup({ authorize: async () => grant });
    assert.equal((await service.start(command())).code, 'permission_denied');
    assert.equal(calls.start.length, 0);
  }
  const { service } = setup();
  const hostile = { toString() { throw new Error('do not coerce'); } };
  for (const input of [null, { ...command(), streamId: hostile }, { ...command(), actor: 123 }, { ...command(), requestId: '' }]) {
    assert.equal((await service.start(input)).code, 'invalid_request');
  }
});

test('provider and auth exceptions, private session handles and extras are redacted', async () => {
  const secret = 'SYNTHETIC_DO_NOT_EXPOSE';
  const auth = setup({ authorize: async () => { throw new Error(secret); } });
  assert.equal(JSON.stringify(await auth.service.start(command())).includes(secret), false);
  const failed = setup({ provider: { start: async () => { throw new Error(secret); } } });
  const failure = await failed.service.start(command());
  assert.equal(failure.stream.state, 'failed');
  assert.equal(JSON.stringify(failure).includes(secret), false);
  const success = setup({ provider: { start: async () => ({ sessionId: 'private-session', streamKey: secret }),
    watch: async () => ({ ready: true, token: secret, url: `https://example.invalid/${secret}` }) } });
  await success.service.start(command());
  const publicResult = JSON.stringify(await watch(success.service));
  assert.equal(publicResult.includes(secret), false);
  assert.equal(publicResult.includes('private-session'), false);
  assert.equal(publicResult.includes('https:'), false);
});

test('a changed creator binding cannot authorize a new owner or stop the original', async () => {
  let creatorId = 'creator-a';
  const { service, calls } = setup({ resolveCreator: async () => ({ creatorId, source: 'local-fixture' }) });
  await service.start(command());
  creatorId = 'creator-b';
  assert.equal((await service.stop(command('b', 'other'))).code, 'creator_unavailable');
  assert.equal((await service.start(command('c', 'other'))).code, 'creator_unavailable');
  assert.equal(calls.start.length, 1);
  assert.equal(calls.stop.length, 0);
});

test('authorized duplicate starts share one provider call and replay current state', async () => {
  const entered = deferred(), finish = deferred();
  let count = 0;
  const { service } = setup({ provider: { async start() { count += 1; entered.resolve(); return finish.promise; } } });
  const first = service.start(command());
  await entered.promise;
  const second = service.start(command());
  finish.resolve({ sessionId: 'one-session' });
  const [a, b] = await Promise.all([first, second]);
  assert.equal(a.ok, true); assert.equal(b.ok, true); assert.equal(count, 1);
  assert.equal([a.replayed, b.replayed].filter(Boolean).length, 1);
  await service.stop(command('stop'));
  const replay = await service.start(command());
  assert.equal(replay.replayed, true);
  assert.equal(replay.stream.state, 'ended');
  assert.equal(replay.stream.playback.available, false);
  assert.equal(count, 1);
  assert.equal((await service.start(command('new'))).code, 'superseded');
});

test('idempotency does not bypass auth and conflicting reuse cannot perform another action', async () => {
  const { service, principals, calls } = setup();
  await service.start(command());
  assert.equal((await service.stop(command())).code, 'idempotency_conflict');
  assert.equal(calls.stop.length, 0);
  principals.set('owner', { subject: 'creator-a', permissions: [] });
  assert.equal((await service.start(command())).code, 'permission_denied');
  assert.equal(calls.start.length, 1);
});

for (const decision of ['stop', 'ban']) {
  test(`${decision} during deferred provider start cannot resurrect and cleans the late session`, async () => {
    const entered = deferred(), finish = deferred();
    const stopped = [];
    const { service } = setup({ provider: {
      async start({ signal }) { entered.resolve(signal); return finish.promise; },
      async stop(value) { stopped.push(value.sessionId); },
    } });
    const pending = service.start(command());
    const signal = await entered.promise;
    const closed = decision === 'stop' ? await service.stop(command('stop'))
      : await service.moderate({ ...command('ban', 'moderator'), decision: 'ban' });
    assert.equal(closed.stream.state, decision === 'stop' ? 'ended' : 'banned');
    assert.equal(signal.aborted, true);
    finish.resolve({ sessionId: 'late-session' });
    const late = await pending;
    assert.equal(late.code, 'superseded');
    assert.equal(late.stream.state, decision === 'stop' ? 'ended' : 'banned');
    assert.deepEqual(stopped, ['late-session']);
    assert.equal((await watch(service)).stream.playback.available, false);
  });
}

test('stop also cancels a start still awaiting authorization', async () => {
  const grant = deferred(), entered = deferred();
  const { service, calls } = setup({ authorize: async (actor, { action }) => {
    if (action === 'start') { entered.resolve(); return grant.promise; }
    return { subject: 'creator-a', permissions: OWNER };
  } });
  const start = service.start(command());
  await entered.promise;
  await service.stop(command('stop'));
  grant.resolve({ subject: 'creator-a', permissions: OWNER });
  assert.equal((await start).code, 'superseded');
  assert.equal(calls.start.length, 0);
});

test('a late watch response cannot restore availability after ban', async () => {
  const entered = deferred(), finish = deferred();
  const { service } = setup({ provider: { async watch() { entered.resolve(); return finish.promise; } } });
  await service.start(command());
  const watching = watch(service);
  await entered.promise;
  await service.moderate({ ...command('ban', 'moderator'), decision: 'ban' });
  finish.resolve({ ready: true, url: 'https://example.invalid/private' });
  const result = await watching;
  assert.equal(result.stream.state, 'banned');
  assert.deepEqual(result.stream.playback, { available: false, reason: 'banned' });
});

test('a late stop completion cannot downgrade a newer ban', async () => {
  const entered = deferred(), finish = deferred();
  let stops = 0;
  const { service } = setup({ provider: { async stop() { stops += 1; entered.resolve(); await finish.promise; } } });
  await service.start(command());
  const stopping = service.stop(command('stop'));
  await entered.promise;
  const banning = service.moderate({ ...command('ban', 'moderator'), decision: 'ban' });
  // Watch does not need to wait for provider cleanup to observe local policy.
  assert.equal((await watch(service)).stream.state, 'banned');
  finish.resolve();
  assert.equal((await stopping).stream.state, 'banned');
  assert.equal((await banning).stream.state, 'banned');
  assert.equal(stops, 1);
});

test('stop failure stays terminal, reports cleanup failure and does not retry secretly', async () => {
  let calls = 0;
  const { service } = setup({ provider: { async stop() { calls += 1; throw new Error('SYNTHETIC_PRIVATE'); } } });
  await service.start(command());
  const result = await service.stop(command('stop'));
  assert.equal(result.ok, false);
  assert.equal(result.code, 'provider_stop_failed');
  assert.equal(result.stream.state, 'ended');
  assert.equal(result.stream.cleanup, 'failed');
  assert.equal(JSON.stringify(result).includes('SYNTHETIC_PRIVATE'), false);
  await service.stop(command('other-stop'));
  assert.equal(calls, 1);
});

test('missing or malformed provider responses fail closed and never expose playback', async () => {
  for (const result of [undefined, {}, { sessionId: 123 }]) {
    const { service } = setup({ provider: { start: async () => result } });
    assert.equal((await service.start(command())).code, 'provider_protocol_error');
    assert.equal((await watch(service)).stream.state, 'failed');
  }
  for (const result of [{ ready: false }, {}, undefined]) {
    const { service } = setup({ provider: { watch: async () => result } });
    await service.start(command());
    const viewed = await watch(service);
    assert.equal(viewed.stream.playback.available, false);
    assert.equal(viewed.code, result?.ready === false ? 'observed' : 'provider_protocol_error');
  }
  const failure = setup({ provider: { watch: async () => { throw new Error('SYNTHETIC_PRIVATE'); } } });
  await failure.service.start(command());
  assert.equal((await watch(failure.service)).code, 'provider_unavailable');
});

test('only moderators can ban or mark ended, and ended cannot undo a ban', async () => {
  const { service } = setup();
  assert.equal((await service.moderate({ ...command('ban'), decision: 'ban' })).code, 'permission_denied');
  assert.equal((await service.moderate({ ...command('bad', 'moderator'), decision: 'unban' })).code, 'invalid_request');
  await service.start(command());
  assert.equal((await service.moderate({ ...command('end', 'moderator'), decision: 'ended' })).stream.state, 'ended');
  await service.moderate({ ...command('ban', 'moderator'), decision: 'ban' });
  assert.equal((await service.moderate({ ...command('end-2', 'moderator'), decision: 'ended' })).stream.state, 'banned');
});

test('retention expires active sessions, purges terminal metadata and blocks reuse', async () => {
  const { service, advance, calls } = setup({ limits: { maxLifetimeMs: 10, terminalRetentionMs: 5, maxStreams: 1 } });
  await service.start(command());
  advance(10);
  await service.sweep();
  const ended = await watch(service);
  assert.equal(ended.stream.state, 'ended');
  assert.equal(calls.stop.length, 1);
  advance(5);
  await service.sweep();
  assert.equal((await watch(service)).code, 'stream_expired');
  assert.deepEqual(service.maintenanceStats(), { identities: 1, retainedRecords: 0, pendingRequests: 0, eventRecords: 0, idempotencyEntries: 0 });
  assert.equal((await service.start(command('again'))).code, 'stream_expired');
  assert.equal((await service.start(command('new', 'owner', 'stream-b'))).code, 'capacity_exceeded');
});

test('a deferred start remains canceled across lifetime and terminal-retention expiry', async () => {
  const entered = deferred(), finish = deferred();
  const stopped = [];
  const { service, advance } = setup({ limits: { maxLifetimeMs: 10, terminalRetentionMs: 5 },
    provider: { async start() { entered.resolve(); return finish.promise; }, async stop(value) { stopped.push(value.sessionId); } } });
  const pending = service.start(command());
  await entered.promise;
  advance(20);
  await service.sweep();
  assert.equal((await watch(service)).stream.state, 'ended');
  advance(6); // The observed terminal transition started retention at time20.
  finish.resolve({ sessionId: 'late-expired-session' });
  assert.equal((await pending).code, 'superseded');
  await service.sweep();
  assert.deepEqual(stopped, ['late-expired-session']);
  assert.equal((await service.start(command('again'))).code, 'stream_expired');
});

test('history and idempotency caches stay bounded and snapshots cannot mutate them', async () => {
  const { service } = setup({ limits: { maxEvents: 2, maxOperations: 2 } });
  await service.start(command());
  for (let index = 0; index < 20; index += 1) await service.stop(command(`stop-${index}`));
  const result = await watch(service);
  assert.ok(result.stream.events.length <= 2);
  assert.ok(service.maintenanceStats().idempotencyEntries <= 2);
  result.stream.events.push({ type: 'forged', at: 0 });
  result.stream.state = 'live';
  const fresh = await watch(service);
  assert.equal(fresh.stream.state, 'ended');
  assert.equal(fresh.stream.events.some(event => event.type === 'forged'), false);
});

test('pending authorization is bounded and rejected identities do not consume permanent slots', async () => {
  const grant = deferred(), entered = deferred();
  const { service } = setup({ limits: { maxPendingPerStream: 1, maxStreams: 1 },
    authorize: async () => { entered.resolve(); return grant.promise; } });
  const first = service.start(command());
  await entered.promise;
  assert.equal((await service.start(command('other'))).code, 'capacity_exceeded');
  grant.resolve(null);
  assert.equal((await first).code, 'permission_denied');
  assert.equal(service.maintenanceStats().identities, 0);
});

test('invalid limits reject rather than silently creating unbounded storage', () => {
  for (const limits of [{ maxStreams: 0 }, { maxEvents: Infinity }, { unknown: 1 }, { toString: 1 }]) {
    assert.throws(() => createLivestreamService({ limits }), /Invalid local retention limit/);
  }
});

test('command fields are snapshotted before asynchronous authorization', async () => {
  const entered = deferred(), grant = deferred();
  const { service, calls } = setup({ authorize: async (actor, { action }) => {
    if (action === 'moderate') { entered.resolve(); return grant.promise; }
    return { subject: actor === 'owner' ? 'creator-a' : 'viewer-a', permissions: OWNER };
  } });
  await service.start(command());
  const mutable = { ...command('mod-original', 'moderator'), decision: 'stop' };
  const pending = service.moderate(mutable);
  await entered.promise;
  Object.assign(mutable, { streamId: 'stream-b', actor: 'other', requestId: 'mod-changed', decision: 'ban' });
  grant.resolve({ subject: 'moderator-a', permissions: ['stream:moderate'] });
  const result = await pending;
  assert.equal(result.stream.id, 'stream-a');
  assert.equal(result.stream.state, 'ended');
  const replay = await service.moderate({ ...command('mod-original', 'moderator'), decision: 'stop' });
  assert.equal(replay.replayed, true);
  assert.equal(calls.stop.length, 1);
  assert.equal(service.maintenanceStats().identities, 1);
});

for (const fails of [false, true]) {
test(`clock failure during ${fails ? 'rejected' : 'successful'} cleanup is redacted without pending leaks`, async () => {
  const entered = deferred(), finish = deferred();
  let broken = false;
  const { service } = setup({ clock: () => {
    if (broken) throw new Error('SYNTHETIC_PRIVATE_CLOCK');
    return 5;
  }, provider: { async stop() { entered.resolve(); await finish.promise; if (fails) throw new Error('SYNTHETIC_PRIVATE_PROVIDER'); } } });
  await service.start(command());
  const stopping = service.stop(command('stop'));
  await entered.promise;
  broken = true;
  finish.resolve();
  const result = await stopping;
  assert.equal(result.stream.state, 'ended');
  assert.equal(result.stream.cleanup, fails ? 'failed' : 'done');
  assert.equal(result.stream.clockAvailable, false);
  assert.equal(JSON.stringify(result).includes('SYNTHETIC_PRIVATE_CLOCK'), false);
  await service.sweep();
  assert.equal(service.maintenanceStats().pendingRequests, 0);
  assert.equal((await service.start(command('new', 'owner', 'stream-b'))).code, 'clock_unavailable');
  await new Promise(resolve => setImmediate(resolve)); // Surface an unhandled cleanup rejection if one exists.
});
}

test('a failed clock after deferred start never publishes a live session', async () => {
  const entered = deferred(), finish = deferred();
  let broken = false;
  const stops = [];
  const { service } = setup({ clock: () => broken ? NaN : 1,
    provider: { async start() { entered.resolve(); return finish.promise; }, async stop(value) { stops.push(value.sessionId); } } });
  const pending = service.start(command());
  await entered.promise;
  broken = true;
  finish.resolve({ sessionId: 'clock-orphan' });
  const result = await pending;
  assert.equal(result.code, 'clock_unavailable');
  assert.equal(result.stream.state, 'failed');
  assert.deepEqual(stops, ['clock-orphan']);
  assert.equal(service.maintenanceStats().pendingRequests, 0);
});

test('a provider rejection arriving after ban preserves the ban', async () => {
  const entered = deferred(), finish = deferred();
  const { service } = setup({ provider: { async start() { entered.resolve(); return finish.promise; } } });
  const pending = service.start(command());
  await entered.promise;
  await service.moderate({ ...command('ban', 'moderator'), decision: 'ban' });
  finish.reject(new Error('SYNTHETIC_PRIVATE_PROVIDER'));
  const result = await pending;
  assert.equal(result.code, 'superseded');
  assert.equal(result.stream.state, 'banned');
  assert.equal(JSON.stringify(result).includes('SYNTHETIC_PRIVATE_PROVIDER'), false);
});

test('five valid clock reads followed by permanent failure cannot publish live', async () => {
  let reads = 0, starts = 0, stops = 0;
  const { service } = setup({ clock: () => {
    reads += 1;
    if (reads > 5) throw new Error('SYNTHETIC_LATE_CLOCK');
    return 1;
  }, provider: {
    async start() { starts += 1; return { sessionId: 'late-clock-session' }; },
    async stop() { stops += 1; },
  } });
  const result = await service.start(command());
  assert.equal(result.ok, false);
  assert.equal(result.code, 'clock_unavailable');
  assert.equal(result.stream.state, 'failed');
  assert.equal(result.stream.cleanup, 'done');
  assert.equal(result.stream.clockAvailable, false);
  assert.equal(starts, 1);
  assert.equal(stops, 1);
  assert.equal(service.maintenanceStats().pendingRequests, 0);
});

test('clock-fault boundaries across start settlement never return an observed bad-clock live state', async () => {
  for (let validReads = 0; validReads <= 12; validReads += 1) {
    let reads = 0, faultObserved = false, starts = 0, stops = 0;
    const { service } = setup({ clock: () => {
      reads += 1;
      if (reads > validReads) { faultObserved = true; throw new Error('SYNTHETIC_CLOCK'); }
      return 1;
    }, provider: {
      async start() { starts += 1; return { sessionId: 'boundary-session' }; },
      async stop() { stops += 1; },
    } });
    const result = await service.start(command());
    if (faultObserved) {
      assert.equal(result.ok, false, `validReads=${validReads}`);
      assert.notEqual(result.stream?.state, 'live', `validReads=${validReads}`);
      assert.equal(stops, starts, `started resources must be cleaned at validReads=${validReads}`);
    } else {
      assert.equal(result.ok, true);
      assert.equal(result.stream.state, 'live');
      assert.equal(stops, 0);
    }
    assert.equal(service.maintenanceStats().pendingRequests, 0);
  }
});
