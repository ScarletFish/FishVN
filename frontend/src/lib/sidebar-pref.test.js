// @vitest-environment jsdom
import { beforeEach, expect, test } from 'vitest';
import { getSidebarCollapsed, setSidebarCollapsed } from './sidebar-pref.js';

beforeEach(() => localStorage.clear());

test('默认未收起', () => {
  expect(getSidebarCollapsed()).toBe(false);
});

test('写入收起后读出为收起', () => {
  setSidebarCollapsed(true);
  expect(getSidebarCollapsed()).toBe(true);
});

test('再次写入展开后读出为展开', () => {
  setSidebarCollapsed(true);
  setSidebarCollapsed(false);
  expect(getSidebarCollapsed()).toBe(false);
});

test('只认自己命名空间的值，其他键不影响', () => {
  localStorage.setItem('fishvn.cardTitle', '1');
  expect(getSidebarCollapsed()).toBe(false);
});
