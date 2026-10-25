import { expect, test } from 'vitest';
import { resolveTheme } from './theme.js';

test('显式深浅不随系统变化', () => {
  expect(resolveTheme('light', true)).toBe('light');
  expect(resolveTheme('light', false)).toBe('light');
  expect(resolveTheme('dark', true)).toBe('dark');
  expect(resolveTheme('dark', false)).toBe('dark');
});

test('跟随系统按系统深浅解析', () => {
  expect(resolveTheme('system', true)).toBe('dark');
  expect(resolveTheme('system', false)).toBe('light');
});

test('未知模式回落到浅色', () => {
  expect(resolveTheme('sepia', true)).toBe('light');
  expect(resolveTheme(undefined, true)).toBe('light');
});
