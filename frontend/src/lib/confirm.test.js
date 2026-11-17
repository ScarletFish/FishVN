// @vitest-environment jsdom
import { expect, test } from 'vitest';
import { tick } from 'svelte';

import { confirm } from './confirm.svelte.js';

test('deleteGame 确认框返回是否删除文件的选项', async () => {
  document.body.innerHTML = '<div id="app"></div><div id="modal-root"></div>';
  const p = confirm('deleteGame', { name: '甲' });
  await tick();
  const boxes = document.querySelectorAll('input[type="checkbox"]');
  expect(boxes).toHaveLength(1);
  boxes[0].checked = true;
  boxes[0].dispatchEvent(new Event('change', { bubbles: true }));
  await tick();
  const buttons = document.querySelectorAll('.modal-actions button');
  buttons[1].click();
  expect(await p).toEqual({ confirmed: true, removeFiles: true });
});

test('已有确认框打开时再调会抛错，首个 promise 仍悬着且状态未被覆盖', async () => {
  document.body.innerHTML = '<div id="app"></div><div id="modal-root"></div>';

  const first = confirm('deleteGame', { name: '甲' });
  expect(first).toBeInstanceOf(Promise);
  expect(() => confirm('deleteGame', { name: '乙' })).toThrow('已有确认框打开');

  expect(
    await Promise.race([first.then(() => 'resolved'), Promise.resolve('pending')]),
  ).toBe('pending');
  expect(() => confirm('deleteGame', { name: '丙' })).toThrow('已有确认框打开');
});
