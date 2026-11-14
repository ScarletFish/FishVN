// @vitest-environment jsdom
// 意图：钉住发行年代图表的行为契约——每个年代桶都画出柱、标签与计数来自数据、切换统计范围重新取数。
// 反例自检：删掉 VisGroupedBar 会让第一条变红；把 VisXYLabels 去掉会让第二条变红；把 useStat 的 source 参数去掉会让第三条变红。
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync } from 'svelte';
import { mountChart } from '../../test/chart-dom.js';
import EraChart from './EraChart.svelte';

const ALL = [
  { key: 'pre2000', count: 12 },
  { key: 'y2000', count: 88 },
  { key: 'y2010', count: 29 },
  { key: 'y2020', count: 0 },
];

const FINISHED = [
  { key: 'pre2000', count: 1 },
  { key: 'y2000', count: 0 },
  { key: 'y2010', count: 2 },
  { key: 'y2020', count: 0 },
];

vi.mock('../../lib/api.js', () => ({
  api: {
    dashboardReleaseEras: (status) => Promise.resolve(status === 'finished' ? FINISHED : ALL),
  },
}));

const ERAS = ['2000 年前', '2000 年代', '2010 年代', '2020 年代'];
const COUNTS = ['12', '88', '29', '0'];

let chart = null;

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

async function setup() {
  chart = await mountChart(EraChart);
  return chart.target;
}

test('每个年代桶画出一根柱', async () => {
  const target = await setup();
  const bars = [...target.querySelectorAll('.era-plot svg path[class$="-bar"]')];

  expect(bars.length).toBe(ALL.length);
});

test('四个年代标签与柱顶计数齐全', async () => {
  const target = await setup();
  const texts = [...target.querySelectorAll('.era-plot svg text')].map((el) => el.textContent.trim());

  for (const era of ERAS) expect(texts).toContain(era);
  for (const count of COUNTS) expect(texts).toContain(count);
});

test('切换统计范围按状态重新取数并更新合计', async () => {
  const target = await setup();
  expect(target.textContent).toContain('共 129 部');

  const chip = [...target.querySelectorAll('button')].find(
    (b) => b.textContent.trim() === '已完成'
  );
  chip.click();
  flushSync();
  await new Promise((resolve) => setTimeout(resolve, 120));
  flushSync();

  expect(target.textContent).toContain('共 3 部');
});
