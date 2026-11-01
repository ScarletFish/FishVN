// @vitest-environment jsdom
import { expect, test, vi } from 'vitest';

import './side-tip.js';
import { setTheme } from './theme.js';

window.matchMedia = () => ({
  matches: false,
  addEventListener() {},
  removeEventListener() {},
});

function tipEl() {
  return document.querySelector('.sidebar-tooltip');
}

function tipText() {
  return tipEl().querySelector('.sidebar-tooltip-text').textContent;
}

function button(label) {
  const el = document.createElement('button');
  el.setAttribute('data-tip', label);
  document.body.append(el);
  return el;
}

function enter(el) {
  el.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
}

function leave(el) {
  el.dispatchEvent(new MouseEvent('mouseout', { bubbles: true }));
}

test('侧栏气泡全局只有一枚，换项就地换字，离开才收起', async () => {
  vi.useFakeTimers();
  const first = button('统计');
  const second = button('设置');

  enter(first);
  expect(tipEl().classList.contains('is-visible')).toBe(false);
  vi.advanceTimersByTime(400);
  expect(tipEl().classList.contains('is-visible')).toBe(true);
  expect(tipText()).toBe('统计');

  enter(second);
  expect(document.querySelectorAll('.sidebar-tooltip').length).toBe(1);
  expect(tipText()).toBe('设置');
  expect(tipEl().classList.contains('is-visible')).toBe(true);

  second.setAttribute('data-tip', '切换到日间模式');
  await vi.advanceTimersByTimeAsync(0);
  expect(tipText()).toBe('切换到日间模式');
  expect(tipEl().classList.contains('is-visible')).toBe(true);

  leave(second);
  expect(tipEl().classList.contains('is-visible')).toBe(true);
  vi.advanceTimersByTime(120);
  expect(tipEl().classList.contains('is-visible')).toBe(false);

  first.remove();
  second.remove();
  vi.useRealTimers();
});

test('主题落定即收起气泡，不等指针移动也不走淡出', async () => {
  vi.useFakeTimers();
  const el = button('切换日间 / 夜间');
  enter(el);
  vi.advanceTimersByTime(400);
  expect(tipEl().classList.contains('is-visible')).toBe(true);

  setTheme('dark');

  expect(tipEl().classList.contains('is-visible')).toBe(false);
  expect(tipEl().style.transition).toBe('none');

  vi.advanceTimersByTime(16);
  expect(tipEl().style.transition).toBe('');

  el.remove();
  vi.useRealTimers();
});
