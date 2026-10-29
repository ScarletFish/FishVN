// @vitest-environment jsdom
// 回归测试：TagPicker 接收的回调 prop 必须是驼峰 onToggle / onAdd / onClose。
// Svelte 5 中 on+小写字母(onclick 除外)会被当成 DOM 事件监听器而非回调 prop，
// 若调用方写成 ontoggle/onadd/onclose，这三个回调会恒为 undefined，点击特征/自定义添加/完成都无效。
import { test, expect, vi } from 'vitest';
import { mount, unmount } from 'svelte';
import { tick } from 'svelte';
import TagPicker from './TagPicker.svelte';

vi.mock('../../lib/scrollbars.js', () => ({ panelScrollbars: () => ({ destroy() {} }) }));

const items = [
  { key: 'i1', label: 'Blue Hair', group: 'Hair', raw: { id: 'i1', name: 'Blue Hair', group: 'Hair' } },
  { key: 'i2', label: 'Red Eyes', group: 'Eyes', raw: { id: 'i2', name: 'Red Eyes', group: 'Eyes' } },
];

test('TagPicker: 点击特征触发 onToggle(key, raw)', async () => {
  const onToggle = vi.fn();
  const cmp = mount(TagPicker, {
    target: document.body,
    props: { items, selected: [], groupOrder: ['Hair', 'Eyes'], allowCustom: true, saving: false, onToggle, onAdd: vi.fn(), onClose: vi.fn() },
  });
  document.querySelectorAll('.tag-pill--pick')[0].click();
  await tick();
  expect(onToggle).toHaveBeenCalledTimes(1);
  expect(onToggle.mock.calls[0][0]).toBe('i1');
  expect(onToggle.mock.calls[0][1]).toEqual({ id: 'i1', name: 'Blue Hair', group: 'Hair' });
  unmount(cmp);
});

test('TagPicker: 自定义添加触发 onAdd(name)', async () => {
  const onAdd = vi.fn();
  const cmp = mount(TagPicker, {
    target: document.body,
    props: { items, selected: [], groupOrder: ['Hair', 'Eyes'], allowCustom: true, saving: false, onToggle: vi.fn(), onAdd, onClose: vi.fn() },
  });
  const input = document.querySelector('.tag-picker__search');
  input.value = 'Custom Trait';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();
  document.querySelector('.tag-picker__free').click();
  await tick();
  expect(onAdd).toHaveBeenCalledTimes(1);
  expect(onAdd.mock.calls[0][0]).toBe('Custom Trait');
  unmount(cmp);
});

test('TagPicker: 完成按钮触发 onClose', async () => {
  const onClose = vi.fn();
  const cmp = mount(TagPicker, {
    target: document.body,
    props: { items, selected: [], groupOrder: ['Hair', 'Eyes'], allowCustom: true, saving: false, onToggle: vi.fn(), onAdd: vi.fn(), onClose },
  });
  document.querySelector('.modal-actions .btn').click();
  await tick();
  expect(onClose).toHaveBeenCalledTimes(1);
  unmount(cmp);
});
