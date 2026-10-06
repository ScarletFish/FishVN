import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { after, before, test } from 'node:test';
import { startTestServer } from './helper.js';

let server;
let base = '';
let fixture = '';
let library = '';

// 目录读取要真实磁盘，夹具自己造
before(async () => {
  fixture = mkdtempSync(join(tmpdir(), 'fishvn-fs-'));
  library = join(fixture, 'library');
  mkdirSync(library, { recursive: true });
  server = await startTestServer();
  base = server.base;
});

after(async () => {
  await server.stop();
  rmSync(fixture, { recursive: true, force: true });
});

/** 每个用例独占一个目录，避免 dir_path 唯一约束让用例互相干扰 */
let seq = 0;
function makeDir(files) {
  const dir = join(library, `case-${++seq}`);
  for (const [rel, content] of Object.entries(files)) {
    const full = join(dir, rel);
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
  return dir;
}

async function send(path, method, body) {
  const res = await fetch(`${base}${path}`, {
    method,
    headers: { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  return { status: res.status, body: text ? JSON.parse(text) : null };
}

async function newGame(name) {
  const res = await send('/api/games', 'POST', { name });
  assert.equal(res.status, 201);
  return res.body.game;
}

const addInstallation = (body) => send('/api/installations', 'POST', body);

test('建安装时读目录填入候选入口与大小', async () => {
  const game = await newGame('多候选游戏');
  const dir = makeDir({
    'game.exe': 'aaaa',
    'setup.exe': 'bb',
    'readme.txt': 'cccccccc',
    'data/nested.exe': 'd',
  });

  const res = await addInstallation({ game_id: game.id, dir_path: dir });
  assert.equal(res.status, 201);

  const inst = res.body.installation;
  assert.equal(inst.game_id, game.id);
  // 层数浅的在前：两个顶层 exe，再是嵌套的
  assert.deepEqual(inst.exes, ['game.exe', 'setup.exe', 'data/nested.exe']);
  assert.equal(inst.size_bytes, 4 + 2 + 8 + 1);
  assert.equal(inst.entry_path, null, '候选不止一个时入口留空');
});

test('候选唯一时自动填入口', async () => {
  const game = await newGame('单候选游戏');
  const res = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'only.bat': 'e' }) });
  assert.equal(res.status, 201);
  assert.deepEqual(res.body.installation.exes, ['only.bat']);
  assert.equal(res.body.installation.entry_path, 'only.bat');
});

test('没有可执行文件时入口留空等用户指认', async () => {
  const game = await newGame('无入口游戏');
  const res = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'readme.txt': 'ffff' }) });
  assert.equal(res.status, 201);
  assert.deepEqual(res.body.installation.exes, []);
  assert.equal(res.body.installation.entry_path, null);
});

test('目录不存在返回 400', async () => {
  const game = await newGame('路径错误');
  const res = await addInstallation({ game_id: game.id, dir_path: join(library, '不存在') });
  assert.equal(res.status, 400);
  assert.match(res.body.error, /不存在/);
});

test('路径是文件而非目录返回 400', async () => {
  const game = await newGame('传了文件');
  const file = makeDir({ 'x.txt': 'x' }) + '/x.txt';
  const res = await addInstallation({ game_id: game.id, dir_path: file });
  assert.equal(res.status, 400);
  assert.match(res.body.error, /不是目录/);
});

test('缺 game_id 或 dir_path 返回 400', async () => {
  const game = await newGame('参数缺失');
  const dir = makeDir({ 'a.exe': 'a' });
  assert.equal((await addInstallation({ dir_path: dir })).status, 400);
  assert.equal((await addInstallation({ game_id: game.id })).status, 400);
  assert.equal((await addInstallation({ game_id: game.id, dir_path: '   ' })).status, 400);
});

test('作品不存在返回 404', async () => {
  const res = await addInstallation({
    game_id: '00000000-0000-0000-0000-000000000000',
    dir_path: makeDir({ 'a.exe': 'a' }),
  });
  assert.equal(res.status, 404);
});

test('同一目录重复建安装返回 409', async () => {
  const dir = makeDir({ 'a.exe': 'a' });
  const first = await newGame('占位者');
  const second = await newGame('撞号者');
  assert.equal((await addInstallation({ game_id: first.id, dir_path: dir })).status, 201);

  const res = await addInstallation({ game_id: second.id, dir_path: dir });
  assert.equal(res.status, 409);
  assert.match(res.body.error, /已有安装记录/);
});

test('PATCH 改入口与改归属作品', async () => {
  const game = await newGame('原属');
  const other = await newGame('新属');
  const created = await addInstallation({
    game_id: game.id,
    dir_path: makeDir({ 'game.exe': 'a', 'setup.exe': 'b', 'data/nested.exe': 'c' }),
  });
  const id = created.body.installation.id;

  const entry = await send(`/api/installations/${id}`, 'PATCH', { entry_path: 'data/nested.exe' });
  assert.equal(entry.status, 200);
  assert.equal(entry.body.installation.entry_path, 'data/nested.exe');

  const moved = await send(`/api/installations/${id}`, 'PATCH', { game_id: other.id });
  assert.equal(moved.status, 200);
  assert.equal(moved.body.installation.game_id, other.id);
});

test('PATCH 把归属改成不存在的作品返回 404', async () => {
  const game = await newGame('待改归属');
  const created = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const res = await send(`/api/installations/${created.body.installation.id}`, 'PATCH', {
    game_id: '00000000-0000-0000-0000-000000000000',
  });
  assert.equal(res.status, 404);
});

test('PATCH 字段类型错或为空 body 返回 400', async () => {
  const game = await newGame('字段校验');
  const created = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const id = created.body.installation.id;
  assert.equal((await send(`/api/installations/${id}`, 'PATCH', {})).status, 400);
  assert.equal((await send(`/api/installations/${id}`, 'PATCH', { entry_path: '' })).status, 400);
  assert.equal((await send(`/api/installations/${id}`, 'PATCH', { game_id: 7 })).status, 400);
});

test('PATCH 不存在的安装返回 404', async () => {
  const res = await send('/api/installations/00000000-0000-0000-0000-000000000000', 'PATCH', {
    entry_path: 'x.exe',
  });
  assert.equal(res.status, 404);
});

test('GET 可按 game_id 过滤', async () => {
  const mine = await newGame('过滤甲');
  const theirs = await newGame('过滤乙');
  const one = await addInstallation({ game_id: mine.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const two = await addInstallation({ game_id: theirs.id, dir_path: makeDir({ 'b.exe': 'b' }) });

  const filtered = await send(`/api/installations?game_id=${mine.id}`, 'GET');
  assert.equal(filtered.status, 200);
  const ids = filtered.body.installations.map((i) => i.id);
  assert.deepEqual(ids, [one.body.installation.id]);
  assert.equal(ids.includes(two.body.installation.id), false);
});

test('GET 无过滤时列出全部', async () => {
  const res = await send('/api/installations', 'GET');
  assert.equal(res.status, 200);
  assert.ok(res.body.installations.length >= 2);
});

test('GET 找不到时返回空数组', async () => {
  const res = await send('/api/installations?game_id=00000000-0000-0000-0000-000000000000', 'GET');
  assert.equal(res.status, 200);
  assert.deepEqual(res.body.installations, []);
});

test('DELETE 删记录不删磁盘文件，可重新登记', async () => {
  const game = await newGame('待删安装');
  const dir = makeDir({ 'only.exe': 'a' });
  const created = await addInstallation({ game_id: game.id, dir_path: dir });
  const id = created.body.installation.id;

  assert.equal((await send(`/api/installations/${id}`, 'DELETE')).status, 204);
  assert.equal((await send(`/api/installations/${id}`, 'DELETE')).status, 404);

  // 目录仍在，可以重新登记
  assert.equal((await addInstallation({ game_id: game.id, dir_path: dir })).status, 201);
});

test('默认安装只接受属于该作品的安装', async () => {
  const game = await newGame('默认安装甲');
  const other = await newGame('默认安装乙');
  const mine = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const foreign = await addInstallation({ game_id: other.id, dir_path: makeDir({ 'b.exe': 'b' }) });

  const wrongOwner = await send(`/api/games/${game.id}/state`, 'PATCH', {
    default_installation_id: foreign.body.installation.id,
  });
  assert.equal(wrongOwner.status, 400);
  assert.match(wrongOwner.body.error, /不属于这部作品/);

  const missing = await send(`/api/games/${game.id}/state`, 'PATCH', {
    default_installation_id: '00000000-0000-0000-0000-000000000000',
  });
  assert.equal(missing.status, 404);

  const ok = await send(`/api/games/${game.id}/state`, 'PATCH', {
    default_installation_id: mine.body.installation.id,
  });
  assert.equal(ok.status, 200);
  assert.equal(ok.body.state.default_installation_id, mine.body.installation.id);
});

test('删掉默认安装后状态里的引用置空', async () => {
  const game = await newGame('级联置空');
  const created = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const id = created.body.installation.id;

  const set = await send(`/api/games/${game.id}/state`, 'PATCH', { default_installation_id: id });
  assert.equal(set.body.state.default_installation_id, id);

  assert.equal((await send(`/api/installations/${id}`, 'DELETE')).status, 204);

  const afterDelete = await send(`/api/games/${game.id}/state`, 'GET');
  assert.equal(afterDelete.body.state.default_installation_id, null);
});

test('删作品时它的安装退回待认领而非消失', async () => {
  const game = await newGame('作品将删');
  const created = await addInstallation({ game_id: game.id, dir_path: makeDir({ 'a.exe': 'a' }) });
  const id = created.body.installation.id;

  assert.equal((await send(`/api/games/${game.id}`, 'DELETE')).status, 204);

  const still = await send('/api/installations', 'GET');
  const row = still.body.installations.find((i) => i.id === id);
  assert.ok(row, '安装记录应仍在');
  assert.equal(row.game_id, null, '应退回待认领');
});