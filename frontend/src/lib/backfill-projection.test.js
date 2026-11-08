import { expect, test } from 'vitest';
import {
  appendEvent,
  isItemDone,
  mergeLogRows,
  needsReload,
  patchItem,
  projectEvent,
} from './backfill-projection.js';

function status() {
  return {
    done: 0,
    total: 3,
    running: true,
    items: [
      { game_id: 'a', state: 'pending' },
      { game_id: 'b', state: 'ok' },
    ],
  };
}

test('进度事件直接投影到状态', () => {
  const next = projectEvent(status(), { phase: 'backfill', done: 2, total: 3 });
  expect(next.done).toBe(2);
  expect(next.total).toBe(3);
  expect(next.running).toBe(true);
});

test('当前条目转为 active，其他条目保持原位', () => {
  const next = projectEvent(status(), {
    phase: 'backfill',
    game_id: 'a',
    done: 1,
    total: 3,
  });
  expect(next.items[0].state).toBe('active');
  expect(next.items[1]).toEqual({ game_id: 'b', state: 'ok' });
});

test('图片阶段的张数进度不覆盖条目进度', () => {
  const base = status();
  const next = projectEvent(base, { phase: 'images', done: 4, total: 10 });
  expect(next.done).toBe(base.done);
  expect(next.total).toBe(base.total);
  expect(next.stage).toBe('images');
  expect(next.images_done).toBe(4);
  expect(next.images_total).toBe(10);
  expect(projectEvent(next, { phase: 'images', done: 4, total: 10 })).toBe(next);
});

test('无变化的事件保持原对象引用', () => {
  const base = status();
  expect(projectEvent(base, { phase: 'paused' })).toBe(base);
  expect(projectEvent(base, null)).toBe(base);
});

test('只有终态与错误才需要重拉权威状态', () => {
  for (const phase of ['finished', 'failed', 'unmatched', 'resolve', 'rate-limited', 'paused']) {
    expect(needsReload({ phase }), phase).toBe(true);
  }
  for (const phase of ['backfill', 'meta', 'characters', 'images', 'cover']) {
    expect(needsReload({ phase }), phase).toBe(false);
  }
  expect(needsReload({ phase: 'meta', error: '超时' })).toBe(true);
  expect(needsReload(null)).toBe(false);
});

test('只有单条完成的事件需要回取该条结果', () => {
  expect(isItemDone({ phase: 'backfill', game_id: 'a', done: 1 })).toBe(true);
  expect(isItemDone({ phase: 'backfill', done: 3, total: 3 })).toBe(false);
  expect(isItemDone({ phase: 'images', game_id: 'a' })).toBe(false);
  expect(isItemDone({ phase: 'finished' })).toBe(false);
  expect(isItemDone(null)).toBe(false);
});

test('单条回取只替换该行，其余条目与引用保持', () => {
  const base = status();
  const next = patchItem(base, { game_id: 'a', state: 'ok', filled: ['简介'], characters: 4 });

  expect(next).not.toBe(base);
  expect(base.items[0]).toEqual({ game_id: 'a', state: 'pending' });
  expect(next.items[0].state).toBe('ok');
  expect(next.items[1]).toBe(base.items[1]);
  expect(next.done).toBe(base.done);
});

test('回取的条目不在这批目标里时状态不动', () => {
  const base = status();
  expect(patchItem(base, { game_id: 'zzz', state: 'ok' })).toBe(base);
  expect(patchItem(base, null)).toBe(base);
  expect(patchItem(null, { game_id: 'a' })).toBe(null);
});

test('带序号的推送事件直接成为日志行', () => {
  const ev = { phase: 'images', seq: 7, done: 3, total: 5 };
  expect(appendEvent([], ev)).toEqual([
    { seq: 7, name: 'vndb-backfill', event: ev },
  ]);
});

test('没有序号的事件不写日志', () => {
  const logs = [{ seq: 1, event: { phase: 'meta' } }];
  expect(appendEvent(logs, { phase: 'meta' })).toBe(logs);
  expect(appendEvent(logs, null)).toBe(logs);
});

test('历史补读与实时事件按序号去重并保持有序', () => {
  const live = { seq: 3, event: { phase: 'images' } };
  const history = [
    { seq: 3, event: { phase: 'images' } },
    { seq: 2, event: { phase: 'meta' } },
  ];
  const rows = mergeLogRows([live], history);
  expect(rows.map((row) => row.seq)).toEqual([2, 3]);
  expect(rows).toHaveLength(2);
});

test('缺序号的行被忽略', () => {
  expect(mergeLogRows([], [{ event: {} }, { seq: 2, event: {} }]).map((r) => r.seq)).toEqual([2]);
});

test('日志只保留序号最大的 400 条', () => {
  const rows = Array.from({ length: 405 }, (_, i) => ({
    seq: i + 1,
    event: { phase: 'meta' },
  }));
  const kept = mergeLogRows([], rows);
  expect(kept).toHaveLength(400);
  expect(kept[0].seq).toBe(6);
  expect(kept.at(-1).seq).toBe(405);
});
