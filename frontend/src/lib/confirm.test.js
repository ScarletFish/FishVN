// @vitest-environment jsdom
import { expect, test } from 'vitest';

import { confirm } from './confirm.svelte.js';

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
