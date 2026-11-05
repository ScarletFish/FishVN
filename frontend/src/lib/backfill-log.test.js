import { expect, test } from 'vitest';

import { mergeProgress } from './backfill-log.js';

let seq = 0;

const row = (phase, extra = {}) => ({
  seq: ++seq,
  event: { phase, game_id: 'g1', run_id: 1, ...extra },
});

test('同一部作品连续的立绘进度折叠成一行，取最后一条', () => {
  const logs = [
    row('resolve', { detail: '命中 3 条' }),
    row('images', { done: 0, total: 8 }),
    row('images', { done: 1, total: 8 }),
    row('images', { done: 4, total: 8 }),
    row('images', { done: 8, total: 8 }),
    row('backfill', { detail: '写入 11 个字段' }),
  ];
  const out = mergeProgress(logs);
  expect(out.map((r) => r.event.phase)).toEqual(['resolve', 'images', 'backfill']);
  expect(out[1].event.done).toBe(8);
  expect(out[1].event.total).toBe(8);
});

test('不同作品的立绘进度不合并', () => {
  const logs = [
    row('images', { done: 1, total: 8 }),
    row('images', { done: 2, total: 8, game_id: 'g2' }),
    row('images', { done: 3, total: 8 }),
  ];
  expect(mergeProgress(logs)).toHaveLength(3);
});

test('被其它阶段隔开的立绘进度不合并，重跑时不会把两轮并成一条', () => {
  const logs = [
    row('images', { done: 8, total: 8 }),
    row('resolve', { detail: '命中 1 条' }),
    row('images', { done: 1, total: 8 }),
  ];
  expect(mergeProgress(logs)).toHaveLength(3);
});

test('带错误的立绘进度单独成行，不被后续数字覆盖', () => {
  const logs = [
    row('images', { done: 8, total: 8 }),
    row('images', { done: 8, total: 8, error: '2 张角色立绘下载失败' }),
  ];
  const out = mergeProgress(logs);
  expect(out).toHaveLength(2);
  expect(out[1].event.error).toBe('2 张角色立绘下载失败');
});

test('其它阶段重复出现时保持原样', () => {
  const logs = [row('meta'), row('meta'), row('cover'), row('cover')];
  expect(mergeProgress(logs)).toHaveLength(4);
});

test('空日志返回空数组', () => {
  expect(mergeProgress([])).toEqual([]);
});
