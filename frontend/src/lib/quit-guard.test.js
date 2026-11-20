import { beforeEach, expect, test } from 'vitest';
import { markQuitDirty, quitGuard } from './quit-guard.svelte.js';

beforeEach(() => {
  markQuitDirty('gameForm', false);
  markQuitDirty('backfillPanel', false);
});

test('任一来源标脏即拦截退出', () => {
  markQuitDirty('gameForm', true);
  expect(quitGuard.dirty).toBe(true);
  markQuitDirty('backfillPanel', true);
  expect(quitGuard.dirty).toBe(true);
});

test('清掉一个来源不影响另一个', () => {
  markQuitDirty('gameForm', true);
  markQuitDirty('backfillPanel', true);
  markQuitDirty('gameForm', false);
  expect(quitGuard.dirty).toBe(true);
  markQuitDirty('backfillPanel', false);
  expect(quitGuard.dirty).toBe(false);
});

test('重复标脏同一来源不会残留', () => {
  markQuitDirty('gameForm', true);
  markQuitDirty('gameForm', true);
  markQuitDirty('gameForm', false);
  expect(quitGuard.dirty).toBe(false);
});
