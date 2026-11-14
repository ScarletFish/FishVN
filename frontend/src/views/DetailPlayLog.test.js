// @vitest-environment jsdom
// 意图：钉住详情页游玩记录的行为契约——会话渲染成阶梯线与面积、末端只标一个累计值、轴文本按跨度格式化、元信息来自数据。
// 反例自检：删掉 VisLine 会让第一条变红；让 label 对每个点都返回值会让第一条变红；把 markLabel 的跨度判断写死会让第二条变红；把 total_seconds 换成会话数会让第三条变红。
import { afterEach, expect, test, vi } from 'vitest';
import { XYLabels } from '@unovis/ts/components/xy-labels';
import { Scatter } from '@unovis/ts/components/scatter';
import { mountChart } from '../test/chart-dom.js';
import DetailPlayLog from './DetailPlayLog.svelte';

const at = (day) => Math.floor(new Date(2026, 2, day, 20, 0, 0).getTime() / 1000);

const LOG = {
  count: 3,
  first_at: at(1),
  last_at: at(20),
  total_seconds: 5400,
  sessions: [
    {
      id: 's1',
      game_id: 'g1',
      started_at: at(1),
      last_seen_at: at(1) + 1800,
      ended_at: at(1) + 1800,
      foreground_seconds: 1800,
      alive_seconds: 1800,
    },
    {
      id: 's2',
      game_id: 'g1',
      started_at: at(9),
      last_seen_at: at(9) + 1200,
      ended_at: at(9) + 1200,
      foreground_seconds: 1200,
      alive_seconds: 1200,
    },
    {
      id: 's3',
      game_id: 'g1',
      started_at: at(20),
      last_seen_at: at(20) + 2400,
      ended_at: at(20) + 2400,
      foreground_seconds: 2400,
      alive_seconds: 2400,
    },
  ],
};

vi.mock('../lib/api.js', () => ({
  api: { gamePlayLog: () => Promise.resolve(LOG) },
}));

vi.mock('../lib/config-store.svelte.js', () => ({
  getTimeMetric: () => 'foreground',
}));

vi.mock('../lib/library-events.js', () => ({
  LIBRARY_CHANGED: 'fishvn:library-changed',
  onLibraryItemUpdated: () => () => {},
  onPlaySessionStarted: () => () => {},
  onPlaySessionEnded: () => () => {},
}));

let chart = null;

afterEach(() => {
  chart?.cleanup();
  chart = null;
});

function texts(target) {
  return [...target.querySelectorAll('.detail-playlog-chart svg text')].map((el) =>
    el.textContent.trim()
  );
}

test('三次会话渲染出阶梯线与面积，末端只标一个累计值', async () => {
  chart = await mountChart(DetailPlayLog, { gameId: 'g1' });
  const target = chart.target;

  expect(target.querySelectorAll('.detail-playlog-chart svg path').length).toBeGreaterThan(0);
  expect(target.querySelectorAll('.' + Scatter.selectors.point).length).toBe(1);
  expect(target.querySelectorAll('.' + XYLabels.selectors.label).length).toBe(1);
  expect(texts(target)).toContain('1h30m');
});

test('跨度不足两个月时 x 轴标月日，y 轴刻度按小时', async () => {
  chart = await mountChart(DetailPlayLog, { gameId: 'g1' });

  expect(texts(chart.target)).toContain('03-01');
  expect(texts(chart.target)).toContain('1h');
});

test('元信息取首次、最近、平均与最长', async () => {
  chart = await mountChart(DetailPlayLog, { gameId: 'g1' });
  const text = chart.target.textContent.replace(/\s+/g, ' ');

  expect(text).toContain('2026-03-01');
  expect(text).toContain('2026-03-20');
  expect(text).toContain('平均 30 分');
  expect(text).toContain('单次最长 40 分');
});
