import assert from 'node:assert/strict';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';

// DATA_DIR 在模块加载时求值，必须先设好再动态 import，否则会写到真实 data/
const dataDir = mkdtempSync(join(tmpdir(), 'fishvn-games-'));
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

async function post(body) {
  const res = await fetch(`${base}/api/games`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() };
}

async function patch(id, body) {
  const res = await fetch(`${base}/api/games/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() };
}

test('只给名字就能建条目，其余字段留空', async () => {
  const { status, body } = await post({ name: '  星空  ' });
  assert.equal(status, 201);
  assert.equal(body.game.name, '星空');
  assert.equal(body.game.vndb_id, null);
  assert.equal(typeof body.game.id, 'string');
  assert.equal(typeof body.game.created_at, 'number');
});

test('多条未匹配条目合法：vndb_id 的 NULL 之间不算重复', async () => {
  const a = await post({ name: 'A' });
  const b = await post({ name: 'B' });
  assert.equal(a.status, 201);
  assert.equal(b.status, 201);
  assert.notEqual(a.body.game.id, b.body.game.id);
});

test('name 缺省、为空、类型错都返回 400', async () => {
  assert.equal((await post({})).status, 400);
  assert.equal((await post({ name: '   ' })).status, 400);
  assert.equal((await post({ name: 42 })).status, 400);
});

test('body 非法 JSON 或不是对象时返回 400', async () => {
  const send = (payload) =>
    fetch(`${base}/api/games`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: payload,
    });

  const broken = await send('{');
  assert.equal(broken.status, 400);
  assert.match((await broken.json()).error, /JSON/);

  for (const payload of ['"abc"', '123', 'null']) {
    const res = await send(payload);
    assert.equal(res.status, 400, `payload=${payload}`);
    assert.match((await res.json()).error, /JSON 对象/);
  }

  // 数组在 JS 里也是 object，会继续走到必填校验
  const arr = await send('[]');
  assert.equal(arr.status, 400);
  assert.match((await arr.json()).error, /name 必填/);
});

test('vndb_id 格式非法返回 400', async () => {
  assert.equal((await post({ name: 'X', vndb_id: '14' })).status, 400);
  assert.equal((await post({ name: 'X', vndb_id: 'v' })).status, 400);
  assert.equal((await post({ name: 'X', vndb_id: 'v14x' })).status, 400);
});

test('建条目时可以带上 vndb_id', async () => {
  const { status, body } = await post({ name: '命中', vndb_id: 'v20001' });
  assert.equal(status, 201);
  assert.equal(body.game.vndb_id, 'v20001');
});

test('GET 列出全部条目，按创建时间倒序', async () => {
  const res = await fetch(`${base}/api/games`);
  assert.equal(res.status, 200);
  const { games } = await res.json();
  assert.ok(Array.isArray(games));
  assert.ok(games.length >= 4, `期望至少 4 条，实际 ${games.length}`);
  for (const key of ['id', 'name', 'vndb_id', 'created_at']) {
    assert.ok(key in games[0], `缺字段 ${key}`);
  }
  for (let i = 1; i < games.length; i++) {
    assert.ok(games[i - 1].created_at >= games[i].created_at, '未按创建时间倒序');
  }
});

test('PATCH 改 name 与绑定 vndb_id', async () => {
  const { body: created } = await post({ name: '原名' });
  const { status, body } = await patch(created.game.id, { name: '新名', vndb_id: 'v20002' });
  assert.equal(status, 200);
  assert.equal(body.game.name, '新名');
  assert.equal(body.game.vndb_id, 'v20002');
});

test('PATCH 绑到已被占用的 vndb_id 返回 409', async () => {
  const a = await post({ name: '占用者', vndb_id: 'v20003' });
  const b = await post({ name: '后来者' });
  assert.equal(a.status, 201);
  assert.equal(b.status, 201);
  const { status, body } = await patch(b.body.game.id, { vndb_id: 'v20003' });
  assert.equal(status, 409);
  assert.match(body.error, /占用/);
});

test('PATCH vndb_id 给 null 是解绑，之后另一条可以绑同一个号', async () => {
  const a = await post({ name: '原持有', vndb_id: 'v20004' });
  const b = await post({ name: '接盘' });
  assert.equal(a.status, 201);
  assert.equal(b.status, 201);
  assert.equal((await patch(a.body.game.id, { vndb_id: null })).status, 200);
  assert.equal((await patch(b.body.game.id, { vndb_id: 'v20004' })).status, 200);
});

test('PATCH 不存在的 id 返回 404', async () => {
  const { status } = await patch('00000000-0000-0000-0000-000000000000', { name: 'x' });
  assert.equal(status, 404);
});

async function del(id) {
  const res = await fetch(`${base}/api/games/${id}`, { method: 'DELETE' });
  const text = await res.text();
  return { status: res.status, body: text ? JSON.parse(text) : null };
}

async function exists(id) {
  const { games } = await fetch(`${base}/api/games`).then((r) => r.json());
  return games.some((g) => g.id === id);
}

test('DELETE 删掉条目，列表里不再出现', async () => {
  const { body: created } = await post({ name: '待删' });
  const id = created.game.id;
  assert.equal(await exists(id), true);

  const { status } = await del(id);
  assert.equal(status, 204);
  assert.equal(await exists(id), false);
});

test('DELETE 同一 id 第二次返回 404', async () => {
  const { body: created } = await post({ name: '删一次' });
  assert.equal((await del(created.game.id)).status, 204);
  assert.equal((await del(created.game.id)).status, 404);
});

test('DELETE 不存在的 id 返回 404', async () => {
  const { status, body } = await del('00000000-0000-0000-0000-000000000000');
  assert.equal(status, 404);
  assert.match(body.error, /不存在/);
});

test('PATCH 空 body 或无可更新字段返回 400', async () => {
  const { body: created } = await post({ name: '空改' });
  assert.equal((await patch(created.game.id, {})).status, 400);
  assert.equal((await patch(created.game.id, { vndb_id: undefined })).status, 400);
});