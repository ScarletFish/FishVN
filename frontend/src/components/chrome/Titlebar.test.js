// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

import Titlebar from './Titlebar.svelte';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

function render(props = {}) {
  const modalRoot = document.createElement('div');
  modalRoot.id = 'modal-root';
  const target = document.createElement('div');
  document.body.append(modalRoot, target);
  app = mount(Titlebar, { target, props });
  flushSync();
  return document.querySelector('.titlebar__sidebar-toggle');
}

test('展开态按钮标注收起，且不高亮', () => {
  const btn = render();
  expect(btn.getAttribute('aria-label')).toBe('收起侧栏');
  expect(btn.getAttribute('data-tooltip')).toBe('收起侧栏');
  expect(btn.classList.contains('is-collapsed')).toBe(false);
});

test('收起态按钮标注展开并高亮', () => {
  const btn = render({ sidebarCollapsed: true });
  expect(btn.getAttribute('aria-label')).toBe('展开侧栏');
  expect(btn.classList.contains('is-collapsed')).toBe(true);
});

test('点击按钮触发切换回调', () => {
  const ontogglesidebar = vi.fn();
  const btn = render({ ontogglesidebar });
  btn.click();
  expect(ontogglesidebar).toHaveBeenCalledOnce();
});

test('详情模式下开关按钮仍在且排在最前', () => {
  const btn = render({ mode: 'detail', onback: () => {} });
  const left = document.querySelector('.titlebar__left');
  expect(left.firstElementChild).toBe(btn);
});
