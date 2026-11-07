import { expect, test } from 'vitest';

import {
  createDockState,
  reduce,
  seedFromStatus,
  markFinished,
  dockVisible,
} from './dock-state.js';

const ev = (phase, extra = {}) => ({ phase, run_id: 1, ...extra });
const ui = (patch = {}) => ({ closedRun: 0, hidden: false, ...patch });

test('新一轮 run_id 变化即重置上一轮的进度与标志', () => {
  let s = createDockState();
  s = reduce(s, ev('backfill', { done: 5, total: 62 }));
  s = reduce(s, ev('rate', { error: '限流冻结' }));
  s = reduce(s, ev('paused'));
  expect(s.done).toBe(5);
  expect(s.rateLimited).toBe(true);
  expect(s.pausedIdle).toBe(true);

  s = reduce(s, ev('backfill', { run_id: 2, done: 0, total: 30 }));
  expect(s.runId).toBe(2);
  expect(s.done).toBe(0);
  expect(s.total).toBe(30);
  expect(s.rateLimited).toBe(false);
  expect(s.pausedIdle).toBe(false);
  expect(s.running).toBe(true);
});

test('首个事件是阶段事件也算在跑，不要求先见到 backfill', () => {
  const s = reduce(createDockState(), ev('meta'));
  expect(s.running).toBe(true);
  expect(dockVisible(s, ui())).toBe(true);
});

test('finished 事件只关运行标志，已取到的汇总仍留着', () => {
  const report = { total: 1, fetched: 1, unmatched: 0, failed: 0, cancelled: false };
  let s = reduce(createDockState(), ev('backfill', { done: 1, total: 1 }));
  s = markFinished(s, report);
  s = reduce(s, ev('finished'));
  expect(s.running).toBe(false);
  expect(s.finished).toEqual(report);
});

test('只有 backfill 更新全局 done/total，图片进度不覆盖', () => {
  let s = createDockState();
  s = reduce(s, ev('backfill', { done: 1, total: 10 }));
  s = reduce(s, ev('images', { done: 3, total: 8 }));
  expect(s.done).toBe(1);
  expect(s.total).toBe(10);
  s = reduce(s, ev('backfill', { done: 2, total: 10 }));
  expect(s.done).toBe(2);
});

test('paused 置暂停态，下一部开始即回到运行中', () => {
  let s = createDockState();
  s = reduce(s, ev('backfill', { done: 3, total: 62 }));
  s = reduce(s, ev('paused'));
  expect(s.pausedIdle).toBe(true);
  expect(s.running).toBe(true);

  s = reduce(s, ev('backfill', { done: 4, total: 62 }));
  expect(s.pausedIdle).toBe(false);
});

test('限流只置标志位', () => {
  let s = createDockState();
  s = reduce(s, ev('rate', { error: '限流冻结' }));
  expect(s.rateLimited).toBe(true);
  s = reduce(s, ev('rate-limited', { error: '限流冻结' }));
  expect(s.rateLimited).toBe(true);
});

test('无事件或无 phase 时原样返回', () => {
  const s = createDockState();
  expect(reduce(s, null)).toBe(s);
  expect(reduce(s, { run_id: 1 })).toBe(s);
});

test('运行中显示，且不受 closedRun 影响', () => {
  const s = reduce(createDockState(), ev('backfill', { done: 2, total: 62 }));
  expect(dockVisible(s, ui())).toBe(true);
  expect(dockVisible(s, ui({ closedRun: s.runId }))).toBe(true);
});

test('没有在途任务时隐藏', () => {
  expect(dockVisible(createDockState(), ui())).toBe(false);
});

test('主动暂停时不显示，下一部开始又显示', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 3, total: 62 }));
  s = reduce(s, ev('paused'));
  expect(s.running).toBe(true);
  expect(dockVisible(s, ui())).toBe(false);

  s = reduce(s, ev('backfill', { done: 4, total: 62 }));
  expect(dockVisible(s, ui())).toBe(true);
});

test('取消的完结不显示汇总', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 20, total: 62 }));
  s = markFinished(s, { total: 62, fetched: 18, unmatched: 1, failed: 1, cancelled: true });
  expect(dockVisible(s, ui())).toBe(false);
  expect(dockVisible(s, ui({ closedRun: s.runId }))).toBe(false);
});

test('完结汇总默认显示，关掉该轮 run_id 才隐藏', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 62, total: 62 }));
  s = markFinished(s, { total: 62, fetched: 60, unmatched: 1, failed: 1, cancelled: false });
  expect(dockVisible(s, ui())).toBe(true);
  expect(dockVisible(s, ui({ closedRun: s.runId }))).toBe(false);
});

test('markFinished 清掉暂停态并带出汇总', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 62, total: 62 }));
  s = reduce(s, ev('paused'));
  s = markFinished(s, { total: 62, fetched: 61, unmatched: 0, failed: 1, cancelled: false });
  expect(s.running).toBe(false);
  expect(s.pausedIdle).toBe(false);
  expect(s.finished.failed).toBe(1);
});

test('hidden 时任何状态都不显示（详情页与补全页）', () => {
  const running = reduce(createDockState(), ev('backfill', { done: 1, total: 62 }));
  expect(dockVisible(running, ui({ hidden: true }))).toBe(false);

  const done = markFinished(running, { total: 62, fetched: 62, unmatched: 0, failed: 0, cancelled: false });
  expect(dockVisible(done, ui({ hidden: true }))).toBe(false);
});

test('seedFromStatus 用快照回填 running / finished，不丢 run_id', () => {
  const status = {
    running: false,
    run_id: 7,
    finished: { total: 3, fetched: 3, unmatched: 0, failed: 0, cancelled: false },
  };
  const s = seedFromStatus(createDockState(), status);
  expect(s.runId).toBe(7);
  expect(s.finished).toEqual(status.finished);
  expect(dockVisible(s, ui())).toBe(true);
});

test('seedFromStatus 接上在途任务：进度、暂停、限流一并回填', () => {
  const s = seedFromStatus(createDockState(), {
    running: true,
    run_id: 3,
    paused_idle: true,
    rate_limited: true,
    done: 2,
    total: 9,
  });
  expect(s.running).toBe(true);
  expect(s.pausedIdle).toBe(true);
  expect(s.rateLimited).toBe(true);
  expect(s.done).toBe(2);
  expect(s.total).toBe(9);
  expect(s.finished).toBeNull();
});

test('seedFromStatus 收到「未运行、无汇总」时只清运行标志', () => {
  const s = seedFromStatus(
    { ...createDockState(), runId: 3, running: true, done: 2 },
    { running: false, run_id: 3, finished: null },
  );
  expect(s.running).toBe(false);
  expect(s.finished).toBeNull();
});
