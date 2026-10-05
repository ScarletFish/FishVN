import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { startTestServer } from './helper.js';

let server;
let base = '';

before(async () => {
  server = await startTestServer();
  base = server.base;
});

after(async () => {
  await server.stop();
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