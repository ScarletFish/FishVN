// @vitest-environment jsdom
// 意图：钉住通关时长构成图的行为契约——五个档位各一段、图例列出档位与占比、中心显示总部数、空数据走空态。
// 反例自检：删掉 VisDonut 会让第一条变红；把条目的百分比算成固定值会让第二条变红；把 centralLabel 去掉会让第三条变红；把 empty 判断去掉会让第四条变红。
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { Donut } from '@unovis/ts/components/donut';
import { mountChart, waitFor } from '../../test/chart-dom.js';
import LengthRingChart from './LengthRingChart.svelte';

const state = vi.hoisted(() => ({ data: null }));

vi.mock('../../lib/api.js', () => ({
  api: { dashboardLengthBuckets: () => Promise.resolve(state.data) },
}));

const BUCKETS = [
  { key: 'b1', count: 1 },
  { key: 'b2', count: 2 },
  { key: 'b3', count: 3 },
  { key: 'b4', count: 4 },
  { key: 'b5', count: 5 },
];

let chart = null;

beforeEach(() => {
  state.data = BUCKETS;
});

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

async function mountRing() {
  chart = await mountChart(LengthRingChart);
  await waitFor(chart.target, '.' + Donut.selectors.segment);
  return chart.target;
}

test('五个档位各有一段弧', async () => {
  await mountRing();
  const segments = chart.target.querySelectorAll('.' + Donut.selectors.segment);

  expect(segments.length).toBe(BUCKETS.length);
});

test('图例列出档位名与百分比', async () => {
  await mountRing();
  const text = chart.target.textContent;

  expect(text).toContain('≤10');
  expect(text).toContain('>50');
  expect(text).toContain('33%');
});

test('中心显示总部数', async () => {
  await mountRing();
  const label = chart.target.querySelector('.ring-center b');

  expect(label.textContent.trim()).toBe('15');
});

test('没有通关时长时走空态', async () => {
  state.data = BUCKETS.map((b) => ({ ...b, count: 0 }));
  chart = await mountChart(LengthRingChart);

  expect(chart.target.textContent).toContain('暂无通关时长数据');
});
