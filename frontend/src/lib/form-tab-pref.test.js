import { expect, test } from 'vitest';
import { resolveFormTab } from './form-tab-pref.js';

test('合法 tab 原样返回', () => {
  expect(resolveFormTab('basic')).toBe('basic');
  expect(resolveFormTab('profile')).toBe('profile');
  expect(resolveFormTab('tags')).toBe('tags');
  expect(resolveFormTab('installations')).toBe('installations');
  expect(resolveFormTab('characters')).toBe('characters');
});

test('未知或缺失值回落到基本页', () => {
  expect(resolveFormTab(null)).toBe('basic');
  expect(resolveFormTab(undefined)).toBe('basic');
  expect(resolveFormTab('')).toBe('basic');
  expect(resolveFormTab('garbage')).toBe('basic');
});
