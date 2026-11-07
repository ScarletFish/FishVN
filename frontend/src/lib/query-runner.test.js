// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';

import { createQueryRunner } from './query-runner.js';
import { notifyLibraryItemUpdated, onLibraryItemsThrottled } from './library-events.js';

function fakeStore(patch = {}) {
  return {
    results: [],
    resultsSubject: 'game',
    resultsCache: { game: [], character: [] },
    lastJson: { game: null, character: null },
    ...patch,
  };
}

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

afterEach(() => {
  vi.useRealTimers();
});

test('同一条件再加载一次也会真的查询，不吃缓存', async () => {
  const store = fakeStore();
  const query = vi.fn().mockResolvedValue([{ id: 'a' }]);
  const runner = createQueryRunner({ query, store });
  const spec = { subject: 'game' };

  await runner.load(spec);
  await runner.load(spec);

  expect(query).toHaveBeenCalledTimes(2);
  expect(runner.stats()).toMatchObject({ queries: 2, applied: 2, dropped: 0 });
});

test('缓存铺底不会把在途查询判作废', async () => {
  const store = fakeStore();
  const d = deferred();
  const runner = createQueryRunner({ query: vi.fn(() => d.promise), store });

  const inflight = runner.load({ subject: 'game' });
  runner.paint({ subject: 'character' });
  d.resolve([{ id: 'a' }]);

  expect(await inflight).toBe(true);
  expect(store.results).toEqual([{ id: 'a' }]);
  expect(store.resultsSubject).toBe('game');
  expect(runner.stats().dropped).toBe(0);
});

test('只有后发起的查询能作废前一次', async () => {
  const store = fakeStore();
  const first = deferred();
  const second = deferred();
  const queue = [first, second];
  let next = 0;
  const runner = createQueryRunner({ query: vi.fn(() => queue[next++].promise), store });

  const a = runner.load({ subject: 'game' });
  const b = runner.load({ subject: 'character' });
  second.resolve([{ id: 'char' }]);
  expect(await b).toBe(true);
  first.resolve([{ id: 'game' }]);
  expect(await a).toBe(false);

  expect(store.resultsSubject).toBe('character');
  expect(store.results).toEqual([{ id: 'char' }]);
  expect(runner.stats().dropped).toBe(1);
});

test('查询失败只上报错误，不清掉已有结果', async () => {
  const store = fakeStore({ results: [{ id: 'keep' }] });
  const onSettled = vi.fn();
  const runner = createQueryRunner({
    query: vi.fn().mockRejectedValue(new Error('db busy')),
    store,
    onSettled,
  });

  expect(await runner.load({ subject: 'game' })).toBe(false);
  expect(store.results).toEqual([{ id: 'keep' }]);
  expect(runner.stats().failed).toBe(1);
  expect(onSettled).toHaveBeenCalledWith('db busy', expect.objectContaining({ subject: 'game' }));
});

test('没有缓存时 paint 返回 false，有缓存时铺上并返回 true', () => {
  const store = fakeStore();
  const runner = createQueryRunner({ query: vi.fn(), store });

  expect(runner.paint({ subject: 'character' })).toBe(false);
  expect(store.results).toEqual([]);

  store.resultsCache.game = [{ id: 'g1' }];
  store.resultsSubject = 'character';
  expect(runner.paint({ subject: 'game' })).toBe(true);
  expect(store.results).toEqual([{ id: 'g1' }]);
});

test('补全完成事件经 5 秒节流后落进 store', async () => {
  vi.useFakeTimers();
  const store = fakeStore();
  let version = 0;
  const query = vi.fn(async () => [{ id: `v${++version}` }]);
  const runner = createQueryRunner({ query, store });
  const spec = { subject: 'game' };
  const stop = onLibraryItemsThrottled(() => runner.load(spec, { push: true }));

  await runner.load(spec);
  expect(store.results).toEqual([{ id: 'v1' }]);

  notifyLibraryItemUpdated('g1');
  notifyLibraryItemUpdated('g2');
  notifyLibraryItemUpdated('g3');
  await vi.advanceTimersByTimeAsync(10);
  expect(query).toHaveBeenCalledTimes(2);
  expect(store.results).toEqual([{ id: 'v2' }]);

  notifyLibraryItemUpdated('g4');
  await vi.advanceTimersByTimeAsync(5000);
  expect(query).toHaveBeenCalledTimes(3);
  expect(store.results).toEqual([{ id: 'v3' }]);
  expect(runner.stats()).toMatchObject({ reloads: 2, applied: 3, dropped: 0 });

  stop();
});
