import { expect, test } from 'vitest';

import { navKeyIndex, isSidebarToggleKey } from './shortcuts.js';

function digit(key, extra = {}) {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    isComposing: false,
    defaultPrevented: false,
    ...extra,
  };
}

test('数字键从 1 起算给出页序下标', () => {
  expect(navKeyIndex(digit('1'), 6)).toBe(0);
  expect(navKeyIndex(digit('3'), 6)).toBe(2);
  expect(navKeyIndex(digit('6'), 6)).toBe(5);
});

test('超出页数的数字不动作', () => {
  expect(navKeyIndex(digit('7'), 6)).toBeNull();
  expect(navKeyIndex(digit('9'), 6)).toBeNull();
  expect(navKeyIndex(digit('0'), 6)).toBeNull();
  expect(navKeyIndex(digit('4'), 3)).toBeNull();
});

test('带修饰键的数字不动作', () => {
  expect(navKeyIndex(digit('1', { ctrlKey: true }), 6)).toBeNull();
  expect(navKeyIndex(digit('1', { metaKey: true }), 6)).toBeNull();
  expect(navKeyIndex(digit('1', { altKey: true }), 6)).toBeNull();
  expect(navKeyIndex(digit('1', { shiftKey: true }), 6)).toBeNull();
});

test('输入法组合中与已被处理的事件不动作', () => {
  expect(navKeyIndex(digit('1', { isComposing: true }), 6)).toBeNull();
  expect(navKeyIndex(digit('1', { defaultPrevented: true }), 6)).toBeNull();
  expect(navKeyIndex(null, 6)).toBeNull();
});

test('非数字键不动作', () => {
  expect(navKeyIndex(digit('a'), 6)).toBeNull();
  expect(navKeyIndex(digit('ArrowLeft'), 6)).toBeNull();
  expect(navKeyIndex(digit('Enter'), 6)).toBeNull();
});

test('Ctrl+B 与 Cmd+B 切换侧栏，大小写都认', () => {
  expect(isSidebarToggleKey(digit('b', { ctrlKey: true }))).toBe(true);
  expect(isSidebarToggleKey(digit('B', { ctrlKey: true }))).toBe(true);
  expect(isSidebarToggleKey(digit('b', { metaKey: true }))).toBe(true);
});

test('裸 B、带其他修饰键、组合中的 B 都不切换侧栏', () => {
  expect(isSidebarToggleKey(digit('b'))).toBe(false);
  expect(isSidebarToggleKey(digit('b', { ctrlKey: true, altKey: true }))).toBe(false);
  expect(isSidebarToggleKey(digit('b', { ctrlKey: true, shiftKey: true }))).toBe(false);
  expect(isSidebarToggleKey(digit('b', { ctrlKey: true, defaultPrevented: true }))).toBe(false);
  expect(isSidebarToggleKey(null)).toBe(false);
});
