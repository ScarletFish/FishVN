import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';

// DATA_DIR 在模块加载时求值，必须先设好再动态 import，否则会写到真实 data/
const dataDir = mkdtempSync(join(tmpdir(), 'fishvn-health-'));
process.env.FISHVN_DATA_DIR = dataDir;
const { startServer } = await import('../../../dist/server/server.js');
const { closeDb } = await import('../../../dist/server/db.js');

let base = '';
let running;

before(async () => {
  running = await startServer(0, '127.0.0.1');
  base = `http://127.0.0.1:${running.port}`;
});

after(async () => {
  await running.close();
  closeDb();
  rmSync(dataDir, { recursive: true, force: true });
});

test('GET /api/health 返回 ok', async () => {
  const res = await fetch(`${base}/api/health`);
  assert.equal(res.status, 200);
  assert.deepEqual(await res.json(), { ok: true });
});

test('未命中的 API 路径返回 404 JSON', async () => {
  const res = await fetch(`${base}/api/nope`);
  assert.equal(res.status, 404);
  assert.deepEqual(await res.json(), { error: 'not found' });
});