// @vitest-environment jsdom
// 意图：钉住通关时长对照图的行为契约——每个作品各一个散点、轴刻度按时长取整、页脚报出参与对照的作品数、空数据走空态。
// 反例自检：删掉 VisScatter 会让第一条变红；把 tickText 写成固定值会让第二条变红；把 points 换成 over 会让第三条变红；把 empty 判断去掉会让第四条变红。
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { Scatter } from '@unovis/ts/components/scatter';
import { mountChart } from '../../test/chart-dom.js';
import LengthCompareChart from './LengthCompareChart.svelte';

const state = vi.hoisted(() => ({ data: null }));

vi.mock('../../lib/api.js', () => ({
  api: { dashboardLengthCompare: () => Promise.resolve(state.data) },
}));

const POINTS = [
  { id: 'g1', name: '甲作品', expected: 36000, actual: 36000 },
  { id: 'g2', name: '乙作品', expected: 18000, actual: 5400 },
  { id: 'g3', name: '丙作品', expected: 7200, actual: 21600 },
];

let chart = null;

beforeEach(() => {
  state.data = POINTS;
});

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

function texts(target) {
  return [...target.querySelectorAll('.length-plot svg text')].map((el) => el.textContent.trim());
}

test('每个作品画一个散点', async () => {
  chart = await mountChart(LengthCompareChart);
  const dots = chart.target.querySelectorAll('.' + Scatter.selectors.point);

  expect(dots.length).toBe(POINTS.length);
});

test('两轴共用同一批时长刻度', async () => {
  chart = await mountChart(LengthCompareChart);
  const labels = texts(chart.target);

  expect(labels).toContain('0');
  expect(labels).toContain('10h');
});

test('标题旁报出参与对照的作品总数', async () => {
  chart = await mountChart(LengthCompareChart);

  expect(chart.target.textContent).toContain('共 3 部');
});

test('没有齐全的作品时走空态', async () => {
  state.data = [];
  chart = await mountChart(LengthCompareChart);

  expect(chart.target.textContent).toContain('还没有同时有 VNDB 平均通关时长与游玩时长的作品');
});
