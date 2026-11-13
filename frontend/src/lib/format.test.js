import { expect, test } from 'vitest';

import { formatBytes, formatDateTime, formatScore, formatScoreShort } from './format.js';

test.each([
  ['formatScore 91 保留两位小数', formatScore, 91, '9.10'],
  ['formatScore 80 保留两位小数', formatScore, 80, '8.00'],
  ['formatScore 75 保留两位小数', formatScore, 75, '7.50'],
  ['formatScoreShort 整数 80 不补小数位', formatScoreShort, 80, '8'],
  ['formatScoreShort 整数 100 不补小数位', formatScoreShort, 100, '10'],
  ['formatScoreShort 整数 0 不补小数位', formatScoreShort, 0, '0'],
  ['formatScoreShort 非整数 91 保留一位', formatScoreShort, 91, '9.1'],
  ['formatScoreShort 非整数 75 保留一位', formatScoreShort, 75, '7.5'],
  ['formatScoreShort 非整数 719 保留一位', formatScoreShort, 719, '71.9'],
  ['formatBytes 0 字节', formatBytes, 0, '0 B'],
  ['formatBytes 不足 1KB 按字节', formatBytes, 1023, '1023 B'],
  ['formatBytes 1KB 进位取整', formatBytes, 1024, '1 KB'],
  ['formatBytes 1.5KB 取整', formatBytes, 1536, '2 KB'],
  ['formatBytes 1MB 一位小数', formatBytes, 1024 * 1024, '1.0 MB'],
  ['formatBytes 1.5GB 一位小数', formatBytes, 1.5 * 1024 ** 3, '1.5 GB'],
  ['formatBytes 1TB 一位小数', formatBytes, 1024 ** 4, '1.0 TB'],
  ['formatBytes 负数留空', formatBytes, -1, ''],
])('%s', (desc, format, input, expected) => {
  expect(format(input)).toBe(expected);
});

test('formatDateTime 按本地时间补零到分钟', () => {
  const at = new Date(2026, 5, 13, 0, 53, 8);
  expect(formatDateTime(Math.floor(at.getTime() / 1000))).toBe('2026-06-13 00:53');
});

test.each([
  ['零值留空', 0, ''],
  ['负数留空', -1, ''],
  ['空值留空', null, ''],
  ['非数字留空', 'abc', ''],
  ['超出日期范围留空', Number.MAX_SAFE_INTEGER * 1000, ''],
])('formatDateTime %s', (desc, input, expected) => {
  expect(formatDateTime(input)).toBe(expected);
});
