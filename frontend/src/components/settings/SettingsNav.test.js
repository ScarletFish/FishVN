// @vitest-environment jsdom
// 目录只是导航入口，点击必须把分区 id 交给上层，不能吞在组件里；高亮与 aria 状态要同源。
// 反例自检：onclick 改成空函数，「点击分区项交出 id」变红；active 判定改成恒等，「当前分区只有一项标 active」变红。
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

import SettingsNav from './SettingsNav.svelte';

if (typeof ResizeObserver === 'undefined') {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

const SECTIONS = [
  { id: 'appearance', label: '外观' },
  { id: 'content', label: '内容' },
  { id: 'startup', label: '启动' },
];

function render(props = {}) {
  const target = document.createElement('div');
  document.body.append(target);
  app = mount(SettingsNav, {
    target,
    props: { sections: SECTIONS, active: 'content', ...props },
  });
  flushSync();
  return [...document.querySelectorAll('.settings-nav__item')];
}

test('点击分区项把 id 交给 ongoto', () => {
  const ongoto = vi.fn();
  render({ ongoto });

  document.querySelectorAll('.settings-nav__item')[2].click();

  expect(ongoto).toHaveBeenCalledWith('startup');
});

test('当前分区只有一项标 active 并带 aria-current', () => {
  const [appearance, content, startup] = render();

  expect(appearance.classList.contains('active')).toBe(false);
  expect(content.classList.contains('active')).toBe(true);
  expect(startup.classList.contains('active')).toBe(false);
  expect(content.getAttribute('aria-current')).toBe('true');
  expect(appearance.getAttribute('aria-current')).toBe(null);
});
