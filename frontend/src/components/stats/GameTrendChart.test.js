// @vitest-environment jsdom
// 意图：钉住常玩作品图的行为契约——每条序列各画一条折线、图例列出作品名、轴文本来自数据、空数据走空态。
// 反例自检：删掉 VisLine 会让第一条变红；把图例的 each 删掉会让第二条变红；把 bucketLabel 写死会让第三条变红；把 empty 判断去掉会让第四条变红。
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { Line } from '@unovis/ts/components/line';
import { mountChart } from '../../test/chart-dom.js';
import GameTrendChart from './GameTrendChart.svelte';

const state = vi.hoisted(() => ({ data: null }));

vi.mock('../../lib/api.js', () => ({
  api: { dashboardGameTrend: () => Promise.resolve(state.data) },
}));

const BUCKETS = [
  '2025-12-01', '2026-01-01', '2026-02-01', '2026-03-01', '2026-04-01', '2026-05-01',
  '2026-06-01', '2026-07-01', '2026-08-01', '2026-09-01', '2026-10-01', '2026-11-01',
];

const SERIES = [
  { game_id: 'g1', name: '甲作品', total: 10800, values: [0, 0, 0, 0, 0, 0, 0, 3600, 7200, 0, 0, 0] },
  { game_id: 'g2', name: '乙作品', total: 5400, values: [0, 0, 1800, 3600, 0, 0, 0, 0, 0, 0, 0, 0] },
];

let chart = null;

beforeEach(() => {
  state.data = { buckets: BUCKETS, series: SERIES };
});

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

function texts(target) {
  return [...target.querySelectorAll('.trend-plot svg text')].map((el) => el.textContent.trim());
}

test('每条序列各渲染一条折线', async () => {
  chart = await mountChart(GameTrendChart);
  const lines = chart.target.querySelectorAll('.' + Line.selectors.line);

  expect(lines.length).toBe(SERIES.length);
});

test('图例列出每部作品的名字', async () => {
  chart = await mountChart(GameTrendChart);

  for (const item of SERIES) expect(chart.target.textContent).toContain(item.name);
});

test('轴文本来自数据：首月带年份、跨年处再标年', async () => {
  chart = await mountChart(GameTrendChart);
  const labels = texts(chart.target);

  expect(labels).toContain('2025年12月');
  expect(labels).toContain('2026年1月');
  expect(labels).toContain('11月');
});

test('没有游玩记录时走空态', async () => {
  state.data = { buckets: BUCKETS, series: [] };
  chart = await mountChart(GameTrendChart);

  expect(chart.target.textContent).toContain('近一年暂无游玩记录');
});
