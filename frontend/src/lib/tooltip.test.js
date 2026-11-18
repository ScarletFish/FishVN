// @vitest-environment jsdom
import { expect, test, vi } from 'vitest';

import './tooltip.js';

function target(attrs) {
  const el = document.createElement('button');
  for (const [name, value] of Object.entries(attrs)) el.setAttribute(name, value);
  document.body.append(el);
  return el;
}

function hover(el) {
  el.dispatchEvent(new MouseEvent('mouseover', { bubbles: true, clientX: 100, clientY: 100 }));
}

function tipEl() {
  return document.getElementById('globalTooltip');
}

test('通用气泡认 data-tooltip-key，切到无键帽的目标即收起', () => {
  vi.useFakeTimers();
  const first = target({ 'data-tooltip': '收起侧栏', 'data-tooltip-key': 'Ctrl+B' });
  const second = target({ 'data-tooltip': '设置' });

  hover(first);
  expect(tipEl().classList.contains('is-visible')).toBe(false);
  vi.advanceTimersByTime(500);
  expect(tipEl().classList.contains('is-visible')).toBe(true);
  expect(tipEl().querySelector('.tip-label').textContent).toBe('收起侧栏');

  const key = tipEl().querySelector('.tip-key');
  expect(key.textContent).toBe('Ctrl+B');
  expect(key.hidden).toBe(false);

  hover(second);
  vi.advanceTimersByTime(500);
  expect(tipEl().querySelector('.tip-label').textContent).toBe('设置');
  expect(key.textContent).toBe('');
  expect(key.hidden).toBe(true);

  first.remove();
  second.remove();
  vi.useRealTimers();
});

test('富文本气泡仍按目标属性切换 is-rich，正文落在 label 上', () => {
  vi.useFakeTimers();
  const rich = target({ 'data-tooltip': '路径不存在', 'data-tooltip-rich': '' });
  const plain = target({ 'data-tooltip': '设置' });

  hover(rich);
  vi.advanceTimersByTime(500);
  expect(tipEl().classList.contains('is-rich')).toBe(true);
  expect(tipEl().querySelector('.tip-label').textContent).toBe('路径不存在');

  hover(plain);
  vi.advanceTimersByTime(500);
  expect(tipEl().classList.contains('is-rich')).toBe(false);

  rich.remove();
  plain.remove();
  vi.useRealTimers();
});
