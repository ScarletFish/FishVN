// @vitest-environment jsdom
// 意图：钉住活跃度图表的行为契约——容器渲染出折线与面积、轴文本来自数据、每个桶一个数据点且 tooltip 挂载、切换粒度重新取数。
// 反例自检：删掉 VisXYContainer 会让第一条变红；把 tickFormat 改成空串会让第二条变红；把数据截断到 11 个桶会让第三条变红；把 useStat 的 source 参数去掉会让第四条变红。
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync } from 'svelte';
import { Scatter } from '@unovis/ts/components/scatter';
import { mountChart } from '../../test/chart-dom.js';
import ActivityChart from './ActivityChart.svelte';

const MONTHLY = [
  { bucket: '2025-12-01', seconds: 3600 },
  { bucket: '2026-01-01', seconds: 7200 },
  { bucket: '2026-02-01', seconds: 0 },
  { bucket: '2026-03-01', seconds: 5400 },
  { bucket: '2026-04-01', seconds: 9000 },
  { bucket: '2026-05-01', seconds: 1800 },
  { bucket: '2026-06-01', seconds: 4600 },
  { bucket: '2026-07-01', seconds: 24120 },
  { bucket: '2026-08-01', seconds: 3000 },
  { bucket: '2026-09-01', seconds: 0 },
  { bucket: '2026-10-01', seconds: 600 },
  { bucket: '2026-11-01', seconds: 1200 },
];

const WEEKLY = [
  { bucket: '2026-08-24', seconds: 1200 },
  { bucket: '2026-08-31', seconds: 900 },
  { bucket: '2026-09-07', seconds: 0 },
  { bucket: '2026-09-14', seconds: 2400 },
  { bucket: '2026-09-21', seconds: 600 },
  { bucket: '2026-09-28', seconds: 1800 },
  { bucket: '2026-10-05', seconds: 300 },
  { bucket: '2026-10-12', seconds: 0 },
  { bucket: '2026-10-19', seconds: 1500 },
  { bucket: '2026-10-26', seconds: 2700 },
  { bucket: '2026-11-02', seconds: 800 },
  { bucket: '2026-11-09', seconds: 1200 },
];

vi.mock('../../lib/api.js', () => ({
  api: {
    dashboardActivity: (granularity) =>
      Promise.resolve(granularity === 'week' ? WEEKLY : MONTHLY),
  },
}));

const Y_TICKS = ['0s', '2h47m', '5h33m', '8h20m'];

let chart = null;

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

async function setup() {
  chart = await mountChart(ActivityChart);
  return chart.target;
}

async function switchTo(target, label) {
  const chip = [...target.querySelectorAll('button')].find((b) => b.textContent.trim() === label);
  chip.click();
  flushSync();
  await new Promise((resolve) => setTimeout(resolve, 120));
  flushSync();
}

function texts(target) {
  return [...target.querySelectorAll('.activity-plot svg text')].map((el) => el.textContent.trim());
}

test('挂载后渲染出 SVG 与折线面积路径', async () => {
  const target = await setup();
  const svg = target.querySelector('.activity-plot svg');

  expect(svg).not.toBeNull();
  expect(svg.querySelectorAll('path').length).toBeGreaterThan(0);
});

test('y 轴刻度完整，x 轴首点带年份、跨年处再标年', async () => {
  const target = await setup();

  for (const tick of Y_TICKS) expect(texts(target)).toContain(tick);
  expect(texts(target)).toContain('2025年12月');
  expect(texts(target)).toContain('2026年1月');
  expect(texts(target)).toContain('11月');
  expect(texts(target)).toContain('6h42m');
});

test('十二个月各一个散点，tooltip 节点挂载', async () => {
  const target = await setup();
  const points = [...target.querySelectorAll('.' + Scatter.selectors.point)];

  expect(points.length).toBe(MONTHLY.length);
  expect(target.querySelector('.activity-plot [class*="-tooltip"]')).not.toBeNull();
});

test('切到周档按周重新取数并换成年内月/日标签', async () => {
  const target = await setup();
  expect(target.textContent).toContain('近 12 个月时长');
  expect(target.textContent).toContain('月均');

  await switchTo(target, '周');

  expect(texts(target)).toContain('8/24');
  expect(target.textContent).toContain('近 12 周时长');
  expect(target.textContent).toContain('周均');
});
