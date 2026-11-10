import { expect, test } from 'vitest';

import { backfillIndicator } from './backfill-indicator.js';
import { createRun } from './backfill-status.js';

const run = (patch = {}) => ({ ...createRun(), runId: 3, ...patch });
const at = (patch = {}) => backfillIndicator(run(patch), { current: 'home', seenRun: 0 });

test('没有跑过任务时没有圆点，提示仍是导航名', () => {
  const { dot, tip } = backfillIndicator(createRun(), { current: 'home', seenRun: 0 });
  expect(dot).toBeNull();
  expect(tip).toBe('补全');
});

test('在补全页上不显示圆点', () => {
  const { dot } = backfillIndicator(run({ running: true, total: 62 }), {
    current: 'backfill',
    seenRun: 0,
  });
  expect(dot).toBeNull();
});

test('在跑时亮运行点，提示带进度', () => {
  const { dot, tip } = at({ running: true, done: 3, total: 62 });
  expect(dot).toBe('running');
  expect(tip).toBe('正在补全 3/62');
});

test('图片阶段换成张数进度', () => {
  const { tip } = at({
    running: true,
    done: 62,
    total: 62,
    stage: 'images',
    imagesDone: 4,
    imagesTotal: 42,
  });
  expect(tip).toBe('正在下载图片 4/42');
});

test('暂停与限流优先于进度文案', () => {
  expect(at({ running: true, paused: true, done: 3, total: 62 }).tip).toBe('已暂停');
  expect(at({ running: true, rateLimited: true, done: 3, total: 62 }).tip).toBe('限流等待中');
});

test('完结未读时亮完成点，提示是汇总', () => {
  const report = { total: 5, fetched: 4, unmatched: 1, failed: 0, cancelled: false };
  const { dot, tip } = at({ finished: report });
  expect(dot).toBe('done');
  expect(tip).toBe('成功 4/5 部，1 部未匹配');
});

test('进过补全页后完成点消失', () => {
  const report = { total: 5, fetched: 5, unmatched: 0, failed: 0, cancelled: false };
  const { dot } = backfillIndicator(run({ finished: report }), { current: 'home', seenRun: 3 });
  expect(dot).toBeNull();
});

test('取消的完结不留点', () => {
  const report = { total: 5, fetched: 2, unmatched: 1, failed: 0, cancelled: true };
  const { dot } = at({ finished: report });
  expect(dot).toBeNull();
});

test('上一轮的完成点不会盖住新一轮的运行点', () => {
  const report = { total: 5, fetched: 5, unmatched: 0, failed: 0, cancelled: false };
  const { dot, tip } = at({ running: true, done: 1, total: 8, finished: report });
  expect(dot).toBe('running');
  expect(tip).toBe('正在补全 1/8');
});
