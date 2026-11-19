// @vitest-environment jsdom
import { afterEach, expect, test } from 'vitest';

import { placeBubble, selectionInfo } from './selection.js';

afterEach(() => {
  window.getSelection()?.removeAllRanges();
  document.body.innerHTML = '';
});

test('气泡默认浮在选区上方', () => {
  const { left, top } = placeBubble(300, 200, 80, 30, { width: 800, height: 600 });

  expect(left).toBe(260);
  expect(top).toBe(160);
});

test('贴近视口顶部时翻到选区下方', () => {
  const { top } = placeBubble(300, 20, 80, 30, { width: 800, height: 600 });

  expect(top).toBe(44);
});

test('靠右时收进视口内', () => {
  const { left } = placeBubble(790, 200, 80, 30, { width: 800, height: 600 });

  expect(left).toBe(712);
});

test('靠左时收进视口内', () => {
  const { left } = placeBubble(10, 200, 80, 30, { width: 800, height: 600 });

  expect(left).toBe(8);
});

test('内容区选中文本能取到选区信息', () => {
  document.body.innerHTML = '<main class="shell__main"><p id="t">标题文本</p></main>';
  const p = document.getElementById('t');
  const range = document.createRange();
  range.selectNodeContents(p);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
  Range.prototype.getBoundingClientRect = () => ({ left: 10, top: 20, width: 100, height: 16 });

  expect(selectionInfo(sel)?.text).toBe('标题文本');
});

test('模态框内的选中文本同样被识别', () => {
  document.body.innerHTML = '<div class="modal-overlay"><p id="t">字段说明</p></div>';
  const p = document.getElementById('t');
  const range = document.createRange();
  range.selectNodeContents(p);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
  Range.prototype.getBoundingClientRect = () => ({ left: 10, top: 20, width: 100, height: 16 });

  expect(selectionInfo(sel)?.text).toBe('字段说明');
});

test('没有选区时返回空', () => {
  expect(selectionInfo(window.getSelection())).toBe(null);
});
