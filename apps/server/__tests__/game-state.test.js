import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { STATUSES } from '../../../dist/server/types.js';
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

async function send(path, method, body) {
  const res = await fetch(`${base}${path}`, {
    method,
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  return { status: res.status, body: text ? JSON.parse(text) : null };
}

const post = (body) => send('/api/games', 'POST', body);
const getState = (id) => send(`/api/games/${id}/state`, 'GET');
const patchState = (id, body) => send(`/api/games/${id}/state`, 'PATCH', body);
const patchGame = (id, body) => send(`/api/games/${id}`, 'PATCH', body);
const delGame = (id) => send(`/api/games/${id}`, 'DELETE');

async function newGame(name) {
  const { status, body } = await post({ name });
  assert.equal(status, 201);
  return body.game;
}

test('建条目时自动带一条 wishlist 状态', async () => {
  const game = await newGame('默认状态');
  const { status, body } = await getState(game.id);
  assert.equal(status, 200);
  assert.equal(body.state.status, 'wishlist');
  assert.equal(body.state.game_id, game.id);
  assert.equal(body.state.user_rating, null);
  assert.equal(body.state.notes, null);
  assert.equal(body.state.last_played_at, null);
  assert.equal(body.state.play_count, 0);
});

test('带 vndb_id 建条目时状态同样是 wishlist', async () => {
  const game = await newGame('已绑定');
  await patchGame(game.id, { vndb_id: 'v20001' });
  const { body } = await getState(game.id);
  assert.equal(body.state.status, 'wishlist');
});

test('四个状态值都能写入', async () => {
  const game = await newGame('状态流转');
  for (const value of STATUSES) {
    const { status, body } = await patchState(game.id, { status: value });
    assert.equal(status, 200, value);
    assert.equal(body.state.status, value);
  }
});

test('表外的状态值返回 400', async () => {
  const game = await newGame('非法状态');
  for (const value of ['on_hold', 'wish', 'WISHLIST', 'playing ', '', 1, null]) {
    const res = await patchState(game.id, { status: value });
    assert.equal(res.status, 400, `status=${JSON.stringify(value)}`);
  }
  const { body } = await getState(game.id);
  assert.equal(body.state.status, 'wishlist', '非法写入不应改变状态');
});

test('PATCH 改 user_rating 与 notes', async () => {
  const game = await newGame('评分笔记');
  const { status, body } = await patchState(game.id, { user_rating: 8.5, notes: '通关了' });
  assert.equal(status, 200);
  assert.equal(body.state.user_rating, 8.5);
  assert.equal(body.state.notes, '通关了');

  const cleared = await patchState(game.id, { user_rating: null, notes: null });
  assert.equal(cleared.body.state.user_rating, null);
  assert.equal(cleared.body.state.notes, null);
});

test('user_rating 与 notes 类型错返回 400', async () => {
  const game = await newGame('类型错');
  assert.equal((await patchState(game.id, { user_rating: '8' })).status, 400);
  assert.equal((await patchState(game.id, { user_rating: true })).status, 400);
  assert.equal((await patchState(game.id, { notes: 42 })).status, 400);
});

test('PATCH 状态空 body 返回 400', async () => {
  const game = await newGame('空改状态');
  assert.equal((await patchState(game.id, {})).status, 400);
  assert.equal((await patchState(game.id, { status: undefined })).status, 400);
});

test('读改不存在的作品状态返回 404', async () => {
  const ghost = '00000000-0000-0000-0000-000000000000';
  assert.equal((await getState(ghost)).status, 404);
  assert.equal((await patchState(ghost, { status: 'playing' })).status, 404);
});

test('删作品时状态随外键级联清除', async () => {
  const game = await newGame('随主删除');
  assert.equal((await delGame(game.id)).status, 204);
  assert.equal((await getState(game.id)).status, 404);
});