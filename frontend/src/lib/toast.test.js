import { afterEach, beforeEach, expect, test, vi } from 'vitest';

import { dismiss, items, toast } from './toast.svelte.js';

beforeEach(() => {
  items.length = 0;
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

test('退场中的条目不算进上限，可见的始终不超过 5 条', () => {
  for (let i = 0; i < 5; i++) toast(`第 ${i} 条`);
  dismiss(items[0].id);

  toast('新的');

  expect(items.filter((t) => t.dismissing)).toHaveLength(1);
  expect(items.filter((t) => !t.dismissing)).toHaveLength(5);
});

test('退场结束移除的是那一条，不是最后一条', async () => {
  const ids = [toast('a'), toast('b'), toast('c')];
  dismiss(ids[1]);

  await vi.advanceTimersByTimeAsync(300);

  expect(items.map((t) => t.id)).toEqual([ids[2], ids[0]]);
});
