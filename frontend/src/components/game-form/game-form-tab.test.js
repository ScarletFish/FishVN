// @vitest-environment jsdom
// 回归：App 把 initialTab 与 bind:tab 指向同一个 formTab，切 tab 会让 initialTab 跟着变。
// 初始化 effect 若依赖它，整份表单与下载资料结果会被重跑清空。
import { test, expect, vi } from 'vitest';
import { tick } from 'svelte';
import { asClassComponent } from 'svelte/legacy';

vi.mock('../../lib/api.js', () => {
  const api = new Proxy(
    {},
    {
      get: (_target, key) =>
        key === 'tagDictionary'
          ? () => Promise.resolve({ tags: {}, traits: {}, tag_groups: null })
          : () => Promise.resolve([]),
    }
  );
  return { api };
});

vi.mock('../../lib/scrollbars.js', () => ({
  useScrollbars: () => {},
  getScrollViewport: () => null,
  panelScrollbars: () => ({ destroy() {} }),
}));

import GameForm from '../GameForm.svelte';

globalThis.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
};
globalThis.requestAnimationFrame ??= (cb) => setTimeout(cb, 0);
globalThis.cancelAnimationFrame ??= clearTimeout;

const Host = asClassComponent(GameForm);

function setState(node, value) {
  node.value = value;
  node.dispatchEvent(new Event('input', { bubbles: true }));
}

test('切 tab 并回写 initialTab 后，表单字段与下载资料侧栏都保留', async () => {
  document.body.innerHTML = '<div id="app"></div><div id="modal-root"></div>';
  const cmp = new Host({
    target: document.body,
    props: { open: true, gameId: null, prefill: null, initialTab: 'basic', tab: 'basic' },
  });
  await tick();

  setState(document.querySelector('#game-form-name'), '测试作品');
  await tick();
  document.querySelector('.game-form__sync').click();
  await tick();
  expect(document.querySelector('.modal--aside')).not.toBeNull();

  document.querySelectorAll('.modal-tab')[1].click();
  await tick();
  cmp.$set({ initialTab: 'profile' });
  await tick();

  expect(document.querySelector('#game-form-name').value).toBe('测试作品');
  expect(document.querySelector('.modal--aside')).not.toBeNull();
  cmp.$destroy();
});
