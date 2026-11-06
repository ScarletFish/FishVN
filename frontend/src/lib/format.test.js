import { expect, test } from 'vitest';

import { formatScore, formatScoreShort } from './format.js';

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
])('%s', (desc, format, input, expected) => {
  expect(format(input)).toBe(expected);
});
