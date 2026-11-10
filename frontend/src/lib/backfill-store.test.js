// @vitest-environment jsdom
import { expect, test, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  api: {
    backfillStatus: vi.fn(),
    listGames: vi.fn(),
    eventsSince: vi.fn(),
    backfillItem: vi.fn(),
  },
  handlers: [],
}));

vi.mock('./api.js', () => ({ api: mocks.api }));

vi.mock('./events.js', () => ({
  subscribe: vi.fn(async (_name, handler) => {
    mocks.handlers.push(handler);
    return () => {};
  }),
}));

import { backfillStore, unwatchBackfill, watchBackfill } from './backfill-store.svelte.js';

test('完结事件不等节流，立即重拉状态并写进运行日志', async () => {
  mocks.api.backfillStatus.mockResolvedValue({
    run_id: 3,
    running: true,
    total: 5,
    done: 0,
    items: [],
  });
  mocks.api.listGames.mockResolvedValue([]);
  mocks.api.eventsSince.mockResolvedValue([]);

  await watchBackfill();
  expect(mocks.api.backfillStatus).toHaveBeenCalledTimes(1);

  mocks.api.backfillStatus.mockResolvedValue({
    run_id: 3,
    running: false,
    total: 5,
    done: 5,
    items: [],
    finished: { total: 5, fetched: 5, unmatched: 0, failed: 0, cancelled: false },
  });

  mocks.handlers[0]({ phase: 'finished', run_id: 3, seq: 1 });

  expect(mocks.api.backfillStatus).toHaveBeenCalledTimes(2);
  await vi.waitFor(() => expect(backfillStore.status.finished?.fetched).toBe(5));
  expect(backfillStore.logs).toHaveLength(1);

  unwatchBackfill();
});
