// @vitest-environment jsdom
import { afterEach, expect, test } from 'vitest';

import './native-menu.js';
import { closeEditMenu, currentEditMenu } from './edit-menu.svelte.js';

function content() {
  let main = document.querySelector('.shell__main');
  if (!main) {
    main = document.createElement('main');
    main.className = 'shell__main';
    document.body.append(main);
  }
  return main;
}

function mount(host, tag, attrs = {}) {
  const el = document.createElement(tag);
  for (const [name, value] of Object.entries(attrs)) el.setAttribute(name, value);
  host.append(el);
  return el;
}

function select(node) {
  const range = document.createRange();
  range.selectNodeContents(node);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
  return sel;
}

function fire(target) {
  const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  return event.defaultPrevented;
}

afterEach(() => {
  closeEditMenu();
  window.getSelection()?.removeAllRanges();
  document.body.innerHTML = '';
});

test('内容区有选区时拦掉原生菜单并弹出编辑菜单', () => {
  const el = mount(content(), 'p');
  el.textContent = 'サクラノ詩';
  select(el);

  expect(fire(el)).toBe(true);
  expect(currentEditMenu()?.kind).toBe('text');
  expect(currentEditMenu()?.text).toBe('サクラノ詩');
});

test('内容区无选区时不弹菜单', () => {
  const el = mount(content(), 'div');

  expect(fire(el)).toBe(true);
  expect(currentEditMenu()).toBe(null);
});

test('可输入元素拦掉原生菜单并以编辑档弹出', () => {
  const field = mount(document.body, 'input');

  expect(fire(field)).toBe(true);
  expect(currentEditMenu()?.kind).toBe('editable');
});

test('可输入元素内部的节点一并接管', () => {
  const field = mount(content(), 'textarea');
  const nested = mount(field, 'b');

  expect(fire(nested)).toBe(true);
  expect(currentEditMenu()?.kind).toBe('editable');
});

test('内容区之外的普通元素既不弹菜单也不出原生菜单', () => {
  const el = mount(document.body, 'div');
  el.textContent = '结构';
  select(el);

  expect(fire(el)).toBe(true);
  expect(currentEditMenu()).toBe(null);
});

test('contenteditable="false" 不算可编辑', () => {
  const el = mount(content(), 'div', { contenteditable: 'false' });

  expect(fire(el)).toBe(true);
  expect(currentEditMenu()).toBe(null);
});
