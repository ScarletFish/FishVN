import { expect, test } from 'vitest';
import { resolveTheme } from './theme.js';

test.each([
  ['显式浅色不随系统变化（系统深色）', 'light', true, 'light'],
  ['显式浅色不随系统变化（系统浅色）', 'light', false, 'light'],
  ['显式深色不随系统变化（系统深色）', 'dark', true, 'dark'],
  ['显式深色不随系统变化（系统浅色）', 'dark', false, 'dark'],
  ['跟随系统且系统为深色时解析为深色', 'system', true, 'dark'],
  ['跟随系统且系统为浅色时解析为浅色', 'system', false, 'light'],
  ['未知模式 sepia 回落到浅色', 'sepia', true, 'light'],
  ['未给模式回落到浅色', undefined, true, 'light'],
])('%s', (desc, mode, systemDark, expected) => {
  expect(resolveTheme(mode, systemDark)).toBe(expected);
});
