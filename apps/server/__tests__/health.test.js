import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { startServer } from '../../../dist/server/server.js';

let base = '';
let running;

before(async () => {
  running = await startServer(0, '127.0.0.1');
  base = `http://127.0.0.1:${running.port}`;
});

after(async () => {
  await running.close();
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