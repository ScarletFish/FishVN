// @vitest-environment jsdom
import { expect, test } from 'vitest';

import { cardStep, cardWidth, computeColsFor, pxLength } from './grid-metrics.js';

function withWidth(el, width) {
  Object.defineProperty(el, 'clientWidth', { value: width, configurable: true });
  return el;
}

test('列数按基准卡宽与容器宽的纯函数算', () => {
  const el = withWidth(document.createElement('div'), 1000);
  expect(computeColsFor(el, { baselineW: 320, gap: 12 })).toBe(3);

  withWidth(el, 664);
  expect(computeColsFor(el, { baselineW: 320, gap: 12 })).toBe(2);

  withWidth(el, 100);
  expect(computeColsFor(el, { baselineW: 320, gap: 12 })).toBe(1);
});

test('容器隐藏时沿用已有的列数，不退回 1', () => {
  const el = document.createElement('div');
  el.style.setProperty('--cols', '5');

  expect(computeColsFor(el, { baselineW: 320, gap: 12 })).toBe(5);
});

test('每张卡的渲染宽与滚动步长', () => {
  // 容器 1000、3 列、间距 12 → 卡宽 (1000 − 24)/3 = 325.33，步长再加一个间距
  expect(cardWidth(1000, 3, 12)).toBeCloseTo(325.333, 3);
  expect(cardStep(1000, 3, 12)).toBeCloseTo(337.333, 3);
  expect(cardStep(1000, 1, 12)).toBe(1012);
});

test('rem 基准按根字号换算成 px', () => {
  const el = document.createElement('div');
  expect(pxLength(el, '17.3125rem')).toBeCloseTo(277, 3);
  expect(pxLength(el, '320px')).toBe(320);
  expect(pxLength(el, '')).toBe(0);
});
