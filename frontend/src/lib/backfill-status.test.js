import { expect, test } from 'vitest';

import { createRun, reduce, seedFromStatus } from './backfill-status.js';

const ev = (phase, extra = {}) => ({ phase, run_id: 1, ...extra });

test('新一轮 run_id 变化即重置上一轮的进度与标志', () => {
  let s = createRun();
  s = reduce(s, ev('backfill', { done: 5, total: 62 }));
  s = reduce(s, ev('rate', { error: '限流冻结' }));
  s = reduce(s, ev('paused'));
  expect(s.done).toBe(5);
  expect(s.rateLimited).toBe(true);
  expect(s.paused).toBe(true);

  s = reduce(s, ev('backfill', { run_id: 2, done: 0, total: 30 }));
  expect(s.runId).toBe(2);
  expect(s.done).toBe(0);
  expect(s.total).toBe(30);
  expect(s.rateLimited).toBe(false);
  expect(s.paused).toBe(false);
  expect(s.running).toBe(true);
});

test('首个事件是阶段事件也算在跑，不要求先见到 backfill', () => {
  const s = reduce(createRun(), ev('meta'));
  expect(s.running).toBe(true);
});

test('finished 只关运行标志，已取到的汇总仍留着', () => {
  const report = { total: 1, fetched: 1, unmatched: 0, failed: 0, cancelled: false };
  let s = reduce(createRun(), ev('backfill', { done: 1, total: 1 }));
  s = { ...s, finished: report };
  s = reduce(s, ev('finished'));
  expect(s.running).toBe(false);
  expect(s.finished).toEqual(report);
});

test('只有 backfill 更新全局 done/total，图片进度不覆盖', () => {
  let s = createRun();
  s = reduce(s, ev('backfill', { done: 1, total: 10 }));
  s = reduce(s, ev('images', { done: 3, total: 8 }));
  expect(s.done).toBe(1);
  expect(s.total).toBe(10);
  s = reduce(s, ev('backfill', { done: 2, total: 10 }));
  expect(s.done).toBe(2);
});

test('paused 置暂停态，下一部开始即回到运行中', () => {
  let s = createRun();
  s = reduce(s, ev('backfill', { done: 3, total: 62 }));
  s = reduce(s, ev('paused'));
  expect(s.paused).toBe(true);
  expect(s.running).toBe(true);

  s = reduce(s, ev('backfill', { done: 4, total: 62 }));
  expect(s.paused).toBe(false);
});

test('限流只置标志位', () => {
  let s = createRun();
  s = reduce(s, ev('rate', { error: '限流冻结' }));
  expect(s.rateLimited).toBe(true);
  s = reduce(s, ev('rate-limited', { error: '限流冻结' }));
  expect(s.rateLimited).toBe(true);
});

test('无事件或无 phase 时原样返回', () => {
  const s = createRun();
  expect(reduce(s, null)).toBe(s);
  expect(reduce(s, { run_id: 1 })).toBe(s);
});

test('run_id 为 0 的单条重试不影响批量运行状态', () => {
  let s = reduce(createRun(), ev('backfill', { done: 7, total: 20 }));
  const before = s;
  s = reduce(s, ev('images', { run_id: 0, done: 1, total: 4 }));
  expect(s).toBe(before);
  expect(s.running).toBe(true);
  expect(s.done).toBe(7);
});

test('图片阶段单独记张数进度，收尾时清掉阶段', () => {
  let s = reduce(createRun(), ev('backfill', { done: 10, total: 10 }));
  s = reduce(s, ev('images', { done: 3, total: 42 }));
  expect(s.stage).toBe('images');
  expect(s.imagesDone).toBe(3);
  expect(s.imagesTotal).toBe(42);

  s = reduce(s, ev('finished'));
  expect(s.stage).toBe('');
});

test('seedFromStatus 接上在途任务：进度、暂停、限流一并回填', () => {
  const s = seedFromStatus(createRun(), {
    running: true,
    run_id: 3,
    paused_idle: true,
    rate_limited: true,
    done: 2,
    total: 9,
  });
  expect(s.runId).toBe(3);
  expect(s.running).toBe(true);
  expect(s.paused).toBe(true);
  expect(s.rateLimited).toBe(true);
  expect(s.done).toBe(2);
  expect(s.total).toBe(9);
  expect(s.finished).toBeNull();
});

test('seedFromStatus 回填图片阶段进度', () => {
  const s = seedFromStatus(createRun(), {
    running: true,
    run_id: 4,
    done: 20,
    total: 20,
    stage: 'images',
    images_done: 7,
    images_total: 30,
  });
  expect(s.stage).toBe('images');
  expect(s.imagesDone).toBe(7);
  expect(s.imagesTotal).toBe(30);
});

test('seedFromStatus 用快照回填完结汇总，不丢 run_id', () => {
  const status = {
    running: false,
    run_id: 7,
    finished: { total: 3, fetched: 3, unmatched: 0, failed: 0, cancelled: false },
  };
  const s = seedFromStatus(createRun(), status);
  expect(s.runId).toBe(7);
  expect(s.running).toBe(false);
  expect(s.finished).toEqual(status.finished);
});

test('seedFromStatus 收到「未运行、无汇总」时只清运行标志', () => {
  const s = seedFromStatus(
    { ...createRun(), runId: 3, running: true, done: 2, finished: { total: 1 } },
    { running: false, run_id: 3, finished: null },
  );
  expect(s.running).toBe(false);
  expect(s.finished).toBeNull();
});
