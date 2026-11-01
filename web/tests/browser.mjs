import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createServer } from "node:net";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function freePort() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}
async function retry(fn) {
  for (let attempt = 0; attempt < 80; attempt++) {
    try {
      return await fn();
    } catch {
      await sleep(100);
    }
  }
  throw new Error("Preview or Chromium did not start");
}
async function stop(child) {
  if (child.exitCode !== null) return;
  child.kill("SIGTERM");
  await Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    sleep(2000),
  ]);
  if (child.exitCode === null) child.kill("SIGKILL");
}

const previewPort = await freePort();
const chromePort = await freePort();
const profile = await mkdtemp(join(tmpdir(), "beam-browser-"));
const preview = spawn(
  "./node_modules/.bin/vite",
  [
    "preview",
    "--host",
    "127.0.0.1",
    "--port",
    String(previewPort),
    "--strictPort",
  ],
  { stdio: "ignore" },
);
const chrome = spawn(
  "chromium",
  [
    "--headless=new",
    "--no-sandbox",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    `--remote-debugging-port=${chromePort}`,
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdio: "ignore" },
);
let ws;
try {
  await retry(async () => {
    if (!(await fetch(`http://127.0.0.1:${previewPort}/`)).ok)
      throw new Error("preview");
  });
  const page = await retry(async () => {
    const pages = await (
      await fetch(`http://127.0.0.1:${chromePort}/json/list`)
    ).json();
    const target = pages.find((item) => item.type === "page");
    if (!target) throw new Error("page");
    return target;
  });
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    ws.addEventListener("open", resolve, { once: true });
    ws.addEventListener("error", reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  const remoteRequests = [];
  ws.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (
      message.method === "Network.requestWillBeSent" &&
      !message.params.request.url.startsWith(
        `http://127.0.0.1:${previewPort}/`,
      ) &&
      !message.params.request.url.startsWith("data:")
    )
      remoteRequests.push(message.params.request.url);
    if (message.id && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id);
      pending.delete(message.id);
      message.error
        ? reject(new Error(message.error.message))
        : resolve(message.result);
    }
  });
  function send(method, params = {}) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      ws.send(JSON.stringify({ id, method, params }));
    });
  }
  async function evaluate(expression) {
    const result = await send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  }
  async function until(expression) {
    return retry(async () => {
      const value = await evaluate(expression);
      if (!value) throw new Error("waiting");
      return value;
    });
  }
  await send("Network.enable");
  await send("Page.enable");
  await send("Page.navigate", { url: `http://127.0.0.1:${previewPort}/` });
  await until("!!document.querySelector('.site-shell')");
  const initialCopy = await evaluate("document.body.innerText");
  assert.match(initialCopy, /sample market data/i);
  assert.match(initialCopy, /Raydium AMM v4/i);
  assert.match(
    initialCopy,
    /0\.42% community \/ 0\.05% treasury \/ 0\.03% founder/,
  );
  assert.doesNotMatch(initialCopy, /CPMM|durable liquidity|lasting liquidity/i);

  // A malformed watchlist must not prevent React from rendering on reload.
  await evaluate(
    "localStorage.setItem('beam-watchlist', '{'); localStorage.setItem('beam-local-drafts', '{')",
  );
  await send("Page.reload", { ignoreCache: true });
  await until("!!document.querySelector('.site-shell')");
  assert.equal(
    await evaluate("document.querySelectorAll('.token-card').length"),
    8,
  );

  await evaluate(
    "[...document.querySelectorAll('button')].find(b => b.textContent.includes('Create token')).click()",
  );
  await until("!!document.querySelector('[role=dialog] form')");
  assert.equal(
    await evaluate(
      "document.querySelector('[role=dialog]').contains(document.activeElement)",
    ),
    true,
  );
  // The last focusable item wraps to the first, including at mobile width.
  await send("Emulation.setDeviceMetricsOverride", {
    width: 375,
    height: 667,
    deviceScaleFactor: 1,
    mobile: true,
  });
  assert.equal(
    await evaluate(
      "document.querySelector('.modal-panel').getBoundingClientRect().right <= innerWidth",
    ),
    true,
  );
  assert.equal(
    await evaluate(
      `(() => { const panel = document.querySelector('.modal-panel'); const items = [...panel.querySelectorAll("button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled])")]; items.at(-1).focus(); document.dispatchEvent(new KeyboardEvent('keydown', {key:'Tab', bubbles:true, cancelable:true})); return document.activeElement === items[0]; })()`,
    ),
    true,
  );

  const maliciousCallout = '<img src=x onerror="window.__beamXss=1">';
  await evaluate(
    `(() => { const put = (selector, value) => { const input = document.querySelector(selector); const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', {bubbles:true})); }; put('input[placeholder="e.g. Cosmic Cat"]', 'First Draft'); put('input[placeholder="e.g. COSMIC"]', 'FIRST'); const area = document.querySelector('textarea[placeholder="Optional community callout"]'); Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(area, ${JSON.stringify(maliciousCallout)}); area.dispatchEvent(new Event('input', {bubbles:true})); })()`,
  );
  await until(
    "document.querySelector('input[placeholder=\"e.g. Cosmic Cat\"]').value === 'First Draft'",
  );
  await evaluate(
    "document.querySelector('form.create-form').requestSubmit(); document.querySelector('form.create-form')?.requestSubmit()",
  );
  await until("!!document.querySelector('.detail-heading')");
  assert.equal(
    await evaluate(
      `(() => { const region = document.querySelector('.amm-v4-status[role="region"]'); const heading = document.getElementById(region?.getAttribute('aria-labelledby')); return !!region && heading?.textContent === 'Curve and AMM v4 status' && region.contains(heading) && region.getBoundingClientRect().right <= innerWidth; })()`,
    ),
    true,
    "the lifecycle status must be a labelled region that fits mobile width",
  );
  const draftLifecycle = await evaluate(
    "document.querySelector('.amm-v4-status').textContent",
  );
  assert.match(draftLifecycle, /Local draft — no token or bonding curve/);
  assert.match(draftLifecycle, /Migration in this build: disabled/);
  assert.match(draftLifecycle, /Verified AMM v4 liquidity: unavailable/);
  assert.equal(
    await evaluate(
      "document.querySelector('.amm-v4-status a, .amm-v4-status button') === null",
    ),
    true,
  );
  const planText = await evaluate(
    "document.querySelector('.draft-options')?.textContent",
  );
  assert.match(planText, /Local plans only — all actions unavailable/);
  assert.match(
    planText,
    /Not submitted for moderation or published as a callout/,
  );
  assert.equal(planText.includes(maliciousCallout), true);
  assert.equal(
    await evaluate(
      "document.querySelector('.draft-options img') === null && document.querySelector('.draft-options a[href]') === null && window.__beamXss === undefined",
    ),
    true,
  );
  assert.equal(
    await evaluate(
      "JSON.parse(localStorage.getItem('beam-local-drafts')).length",
    ),
    1,
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await until("!document.querySelector('[role=dialog]')");
  assert.equal(
    await evaluate(
      "document.activeElement?.textContent.includes('Create token')",
    ),
    true,
  );
  await evaluate(
    "document.querySelector('button[aria-label=\"View Nova Club\"]').click()",
  );
  await until("!!document.querySelector('.amm-v4-status')");
  const sampleLifecycle = await evaluate(
    "document.querySelector('.amm-v4-status').textContent",
  );
  assert.match(
    sampleLifecycle,
    /Sample preview — curve and AMM v4 state unavailable/,
  );
  assert.match(sampleLifecycle, /Verified AMM v4 liquidity: unavailable/);
  assert.doesNotMatch(
    sampleLifecycle,
    /pool address|market cap|price|SOL reserves/i,
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await until("!document.querySelector('[role=dialog]')");
  assert.equal(
    await evaluate(
      "document.querySelector('.token-card .card-open')?.getAttribute('aria-label')",
    ),
    "View First Draft",
  );
  await evaluate(
    "document.querySelector('button[aria-label=\"Add First Draft to watchlist\"]').click()",
  );
  await until(
    "JSON.parse(localStorage.getItem('beam-watchlist')).length === 1",
  );
  await send("Page.reload", { ignoreCache: true });
  await until("!!document.querySelector('.site-shell')");
  assert.equal(
    await evaluate(
      "document.querySelector('.token-card .card-open')?.getAttribute('aria-label')",
    ),
    "View First Draft",
  );
  assert.equal(
    await evaluate(
      "!!document.querySelector('button[aria-label=\"Remove First Draft from watchlist\"]')",
    ),
    true,
  );
  await evaluate(
    "[...document.querySelectorAll('button')].find(b => b.textContent.includes('Create token')).click()",
  );
  await until("!!document.querySelector('[role=dialog] form')");
  await evaluate(
    `(() => { const put = (selector, value) => { const input = document.querySelector(selector); const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', {bubbles:true})); }; put('input[placeholder="e.g. Cosmic Cat"]', 'Second Draft'); put('input[placeholder="e.g. COSMIC"]', 'SECOND'); })()`,
  );
  await evaluate("document.querySelector('form.create-form').requestSubmit()");
  await until(
    "JSON.parse(localStorage.getItem('beam-local-drafts')).length === 2",
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await until("!document.querySelector('[role=dialog]')");
  assert.deepEqual(
    await evaluate(
      "[...document.querySelectorAll('.token-card .card-open')].slice(0,2).map(b=>b.getAttribute('aria-label'))",
    ),
    ["View Second Draft", "View First Draft"],
  );
  await evaluate("document.querySelector('.wallet-button').click()");
  await until("!!document.querySelector('[role=dialog]')");
  assert.match(
    await evaluate("document.querySelector('[role=dialog]').innerText"),
    /cannot connect, sign, or submit/i,
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await until("!document.querySelector('[role=dialog]')");
  await send("Page.addScriptToEvaluateOnNewDocument", {
    source:
      "Object.defineProperty(window, 'localStorage', { configurable: true, get() { throw new Error('blocked storage'); } });",
  });
  await send("Page.reload", { ignoreCache: true });
  await until(
    "document.querySelector('.preview-banner')?.innerText.includes('Browser storage is unavailable')",
  );
  assert.deepEqual(
    remoteRequests,
    [],
    "preview must not make wallet or RPC requests",
  );
  console.log(
    "Chromium preview: malformed storage, draft order/persistence, repeat submit, focus/mobile, watchlist, no remote requests passed",
  );
} finally {
  ws?.close();
  await Promise.all([stop(chrome), stop(preview)]);
  await rm(profile, { recursive: true, force: true });
}
