// @vitest-environment jsdom
import { expect, test } from 'vitest';

import './native-menu.js';

function fire(target) {
  const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event.defaultPrevented;
}

function mount(tag, attrs = {}) {
  const el = document.createElement(tag);
  for (const [name, value] of Object.entries(attrs)) el.setAttribute(name, value);
  document.body.append(el);
  return el;
}

test('普通元素上拦掉原生菜单', () => {
  const host = mount('div');
  const inner = mount('span');
  host.append(inner);

  expect(fire(host)).toBe(true);
  expect(fire(inner)).toBe(true);
});

test('可输入元素保留原生菜单', () => {
  expect(fire(mount('input'))).toBe(false);
  expect(fire(mount('textarea'))).toBe(false);
  expect(fire(mount('div', { contenteditable: 'true' }))).toBe(false);
});

test('可输入元素内部的节点一并放行', () => {
  const field = mount('textarea');
  const nested = mount('b');
  field.append(nested);

  expect(fire(nested)).toBe(false);
});

test('contenteditable="false" 不算可编辑', () => {
  expect(fire(mount('div', { contenteditable: 'false' }))).toBe(true);
});
