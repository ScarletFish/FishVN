import { expect, test } from 'vitest';
import { needsReload, projectEvent } from './backfill-projection.js';

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

test('无变化的事件保持原对象引用', () => {
  const base = status();
  expect(projectEvent(base, { phase: 'images' })).toBe(base);
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
