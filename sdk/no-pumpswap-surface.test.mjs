import test from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

// Split so this file does not itself contain the id it forbids.
const PUMPSWAP_PROGRAM_ID = 'pAMMBay' + '6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA';

function filesUnder(dir, out = []) {
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'target') continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) filesUnder(path, out);
    else out.push(path);
  }
  return out;
}

test('SDK and preview sources do not embed the PumpSwap program id', () => {
  const hits = [];
  for (const root of ['sdk', 'web/src']) {
    for (const file of filesUnder(root)) {
      if (file.endsWith('no-pumpswap-surface.test.mjs')) continue;
      const text = readFileSync(file, 'utf8');
      if (text.includes(PUMPSWAP_PROGRAM_ID)) hits.push(file);
    }
  }
  assert.deepEqual(hits, []);
});
