// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';

import {
  notifyLibraryItemUpdated,
  onLibraryItemUpdated,
  onLibraryItemsThrottled,
} from './library-events.js';

afterEach(() => {
  vi.useRealTimers();
});

test('单部广播立即回调并带上 game_id', () => {
  const handler = vi.fn();
  const stop = onLibraryItemUpdated(handler);

  notifyLibraryItemUpdated('g7');
  notifyLibraryItemUpdated('g9');

  expect(handler).toHaveBeenCalledTimes(2);
  expect(handler).toHaveBeenNthCalledWith(1, 'g7');
  expect(handler).toHaveBeenNthCalledWith(2, 'g9');
  stop();
});

test('注销后不再回调', () => {
  const handler = vi.fn();
  const stop = onLibraryItemUpdated(handler);

  stop();
  notifyLibraryItemUpdated('g7');

  expect(handler).toHaveBeenCalledTimes(0);
});

test('连续多部完成只合并成一次刷新', () => {
  vi.useFakeTimers();
  const handler = vi.fn();
  const stop = onLibraryItemsThrottled(handler, 1000);

  notifyLibraryItemUpdated('g1');
  notifyLibraryItemUpdated('g2');
  notifyLibraryItemUpdated('g3');
  expect(handler).toHaveBeenCalledTimes(0);

  vi.advanceTimersByTime(1000);
  expect(handler).toHaveBeenCalledTimes(1);
  stop();
});

test('间隔超过节流窗口的事件各刷一次', () => {
  vi.useFakeTimers();
  const handler = vi.fn();
  const stop = onLibraryItemsThrottled(handler, 1000);

  notifyLibraryItemUpdated('g1');
  vi.advanceTimersByTime(1000);
  notifyLibraryItemUpdated('g2');
  vi.advanceTimersByTime(1000);

  expect(handler).toHaveBeenCalledTimes(2);
  stop();
});

test('注销后连挂起的刷新一并取消', () => {
  vi.useFakeTimers();
  const handler = vi.fn();
  const stop = onLibraryItemsThrottled(handler, 1000);

  notifyLibraryItemUpdated('g1');
  stop();
  vi.advanceTimersByTime(2000);

  expect(handler).toHaveBeenCalledTimes(0);
});
