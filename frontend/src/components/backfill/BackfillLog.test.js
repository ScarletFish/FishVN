// @vitest-environment jsdom
// 日志最后一条是汇总而非普通事件；文案口径由 backfill-summary 锁，这里断言它确实落到那一行。
import { afterEach, expect, test } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

import BackfillLog from './BackfillLog.svelte';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

const LOGS = [
  { seq: 1, event: { phase: 'resolve', run_id: 7, ts: 1700000000000, game_id: 'g1', detail: '命中 2 条' } },
  { seq: 2, event: { phase: 'paused', run_id: 7, ts: 1700000000500, game_id: null } },
  { seq: 3, event: { phase: 'finished', run_id: 7, ts: 1700000001000, game_id: null } },
];

const STATUS = {
  run_id: 7,
  items: [{ game_id: 'g1', name: '某作' }],
  finished: { total: 5, fetched: 4, unmatched: 1, failed: 0, cancelled: false },
};

function render() {
  const target = document.createElement('div');
  document.body.append(target);
  app = mount(BackfillLog, { target, props: { logs: LOGS, status: STATUS } });
  flushSync();
  return [...document.querySelectorAll('.bf-logrow')];
}

test('完结行标为「汇总」，正文是结果计数', () => {
  const rows = render();
  const last = rows[rows.length - 1];

  expect(rows).toHaveLength(3);
  expect(last.querySelector('.bf-logrow__phase').textContent).toBe('汇总');
  expect(last.textContent).toContain('成功 4/5 部，1 部未匹配');
});

test('条目名落在过程行上，暂停行有自己的标签', () => {
  const [resolve, paused] = render();

  expect(resolve.textContent).toContain('某作');
  expect(resolve.textContent).toContain('命中 2 条');
  expect(paused.querySelector('.bf-logrow__phase').textContent).toBe('已暂停');
});
