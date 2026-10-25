// @vitest-environment jsdom
import { expect, test } from 'vitest';

import {
  PHASE_I18N,
  createDockState,
  reduce,
  seedFromStatus,
  markFinished,
  dockView,
  currentIndex,
} from './dock-state.js';

const ev = (phase, extra = {}) => ({ phase, run_id: 1, ...extra });

test('新一轮 run_id 变化即重置后端字段与错误，无需手动清标记', () => {
  let s = createDockState();
  s = reduce(s, ev('backfill', { done: 5, total: 62 }));
  s = reduce(s, { phase: 'failed', run_id: 1, error: '「A」失败' });
  expect(s.done).toBe(5);
  expect(s.errors).toHaveLength(1);

  // 第二轮启动（run_id 变为 2）
  s = reduce(s, ev('meta', { run_id: 2 }));
  expect(s.runId).toBe(2);
  expect(s.done).toBe(0);
  expect(s.total).toBe(0);
  expect(s.errors).toHaveLength(0);
  expect(s.running).toBe(true);
});

test('阶段标记被跟踪，且不污染全局 done/total（仅 backfill 更新）', () => {
  let s = createDockState();
  s = reduce(s, ev('backfill', { done: 1, total: 10 }));
  s = reduce(s, ev('meta'));
  expect(s.phase).toBe('meta');
  expect(PHASE_I18N.meta).toBe('mylist.backfillPhaseMeta');
  expect(s.done).toBe(1); // meta 不带全局累计

  // images 带的是「单游戏内图片进度」，不应覆盖全局 games-done
  s = reduce(s, ev('images', { done: 3, total: 8 }));
  expect(s.phase).toBe('images');
  expect(s.done).toBe(1);
  expect(s.total).toBe(10);
});

test('failed / unmatched 的报错文本进 errors 列表', () => {
  let s = createDockState();
  s = reduce(s, { phase: 'unmatched', run_id: 1, error: '「B」未匹配' });
  s = reduce(s, { phase: 'failed', run_id: 1, error: '「C」异常' });
  expect(s.errors.map((e) => e.message)).toEqual(['「B」未匹配', '「C」异常']);
  expect(s.errorCount).toBe(2);
});

test('限流只置位不进列表：面板有独立徽标，同一句会刷满列表', () => {
  let s = createDockState();
  s = reduce(s, { phase: 'rate', run_id: 1, error: '限流冻结' });
  s = reduce(s, { phase: 'rate-limited', run_id: 1, error: '限流冻结' });
  expect(s.errors).toHaveLength(0);
  expect(s.rateLimited).toBe(true);
});

test('同文案报错合并计数，不再堆重复行', () => {
  let s = createDockState();
  for (let i = 0; i < 5; i++) {
    s = reduce(s, { phase: 'failed', run_id: 1, error: '「A」网络错误' });
  }
  s = reduce(s, { phase: 'failed', run_id: 1, error: '「B」网络错误' });
  expect(s.errors).toHaveLength(2);
  expect(s.errors[0]).toMatchObject({ message: '「A」网络错误', times: 5 });
  expect(s.errorCount).toBe(6);
});

test('错误列表封顶 20 条，超出的只记数', () => {
  let s = createDockState();
  for (let i = 0; i < 23; i++) {
    s = reduce(s, { phase: 'failed', run_id: 1, error: `「${i}」失败` });
  }
  expect(s.errors).toHaveLength(20);
  expect(s.overflow).toBe(3);
  expect(s.errorCount).toBe(23);
});

test('currentIndex 取 done+1，封顶 total', () => {
  const s = { ...createDockState(), done: 4, total: 10 };
  expect(currentIndex(s)).toBe(5);
  const done = { ...createDockState(), done: 10, total: 10 };
  expect(currentIndex(done)).toBe(10);
});

test('运行中按 run_id 折叠为手柄，展开即恢复', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 2, total: 62 }));
  const ui = { collapsedRun: 0, closedRun: 0, hidden: false };
  expect(dockView(s, ui).form).toBe('panel');

  ui.collapsedRun = s.runId; // 用户对这轮点了收起
  expect(dockView(s, ui).form).toBe('handle');

  ui.collapsedRun = 0; // 点手柄展开
  expect(dockView(s, ui).form).toBe('panel');
});

test('完结汇总默认显示，关掉该轮 run_id 才隐藏', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 62, total: 62 }));
  s = markFinished(s, { total: 62, fetched: 60, unmatched: 1, failed: 1, cancelled: false });
  const ui = { collapsedRun: 0, closedRun: 0, hidden: false };
  expect(dockView(s, ui).form).toBe('panel');
  expect(dockView(s, ui).variant).toBe('summary');

  ui.closedRun = s.runId;
  expect(dockView(s, ui).form).toBe('hidden');
});

test('markFinished 保留运行中收集的 errors 供汇总查看', () => {
  let s = createDockState();
  s = reduce(s, { phase: 'failed', run_id: 1, error: '「X」失败' });
  s = reduce(s, ev('backfill', { done: 62, total: 62 }));
  s = markFinished(s, { total: 62, fetched: 61, unmatched: 0, failed: 1, cancelled: false });
  expect(s.errors).toHaveLength(1); // 没有被 seedFromStatus 清掉
  expect(s.finished.failed).toBe(1);
});

test('详情页 hidden=true 时任何形态都隐藏', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 1, total: 62 }));
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: true }).form).toBe('hidden');
});

test('取消中显示 cancelling 变体且按钮可禁用', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 1, total: 62 }));
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: false }, true)).toEqual({
    form: 'panel',
    variant: 'cancelling',
  });
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
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: false }).variant).toBe('summary');
});

test('暂停先显示 pausing，收到后端 paused 事件才算真停', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 3, total: 62 }));
  s = { ...s, paused: true, pausedIdle: false };
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: false }).variant).toBe('pausing');

  s = reduce(s, ev('paused'));
  expect(s.pausedIdle).toBe(true);
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: false }).variant).toBe('paused');
});

test('暂停期间有进度推进说明还在收尾，回到 pausing', () => {
  let s = reduce(createDockState(), ev('backfill', { done: 3, total: 62 }));
  s = reduce({ ...s, paused: true, pausedIdle: false }, ev('paused'));
  s = reduce(s, ev('backfill', { done: 4, total: 62 }));
  expect(s.pausedIdle).toBe(false);
});

test('seedFromStatus 按后端 paused_idle 决定是否还在收尾', () => {
  const s = seedFromStatus(createDockState(), {
    running: true,
    paused: true,
    paused_idle: false,
    run_id: 3,
    done: 2,
    total: 9,
  });
  expect(s.pausedIdle).toBe(false);
  expect(dockView(s, { collapsedRun: 0, closedRun: 0, hidden: false }).variant).toBe('pausing');
});
