import { expect, test } from 'vitest';

import { backfillSummaryParts, backfillSummaryText } from './backfill-summary.js';

const report = (patch = {}) => ({
  total: 62,
  fetched: 58,
  bound: 0,
  unmatched: 0,
  failed: 0,
  cancelled: false,
  ...patch,
});

test('没有异常时只给成功数与总数', () => {
  expect(backfillSummaryText(report({ fetched: 62 }))).toBe('成功 62/62 部');
});

test('未匹配与失败按逗号追加，只列非零项', () => {
  expect(backfillSummaryText(report({ unmatched: 2, failed: 2 }))).toBe(
    '成功 58/62 部，2 部未匹配，2 部失败',
  );
  expect(backfillSummaryText(report({ fetched: 61, failed: 1 }))).toBe('成功 61/62 部，1 部失败');
});

test('自动绑定数不进汇总，且不会把完成数算超总数', () => {
  expect(backfillSummaryText(report({ fetched: 60, bound: 40, unmatched: 1, failed: 1 }))).toBe(
    '成功 60/62 部，1 部未匹配，1 部失败',
  );
});

test('取消时先给已处理数，再给成功与异常', () => {
  expect(
    backfillSummaryText(
      report({ fetched: 38, unmatched: 1, failed: 1, bound: 20, cancelled: true }),
    ),
  ).toBe('已处理 40/62 部，成功 38 部，1 部未匹配，1 部失败');
});

test('取消且没有成功时不出「成功 0 部」', () => {
  expect(backfillSummaryText(report({ fetched: 0, unmatched: 1, cancelled: true }))).toBe(
    '已处理 1/62 部，1 部未匹配',
  );
});

test('取消且已处理满额时也不超过总数', () => {
  expect(
    backfillSummaryText(report({ fetched: 60, unmatched: 1, failed: 1, cancelled: true })),
  ).toBe('已处理 62/62 部，成功 60 部，1 部未匹配，1 部失败');
});

test('没有报告时返回空串', () => {
  expect(backfillSummaryText(null)).toBe('');
});

test('分段按计数项切分，供日志逐项换行、不在词中断开', () => {
  expect(backfillSummaryParts(report({ unmatched: 2, failed: 2 }))).toEqual([
    '成功 58/62 部',
    '2 部未匹配',
    '2 部失败',
  ]);
  expect(backfillSummaryParts(null)).toEqual([]);
});
