// @vitest-environment jsdom
import { expect, test } from 'vitest';

import { confirm } from './confirm.svelte.js';

test('已有确认框打开时再调会抛错，不共用上一个 promise', () => {
  document.body.innerHTML = '<div id="app"></div><div id="modal-root"></div>';

  confirm('deleteGame', { name: '甲' });

  expect(() => confirm('deleteGame', { name: '乙' })).toThrow('已有确认框打开');
});
