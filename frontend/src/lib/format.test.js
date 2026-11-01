import { describe, expect, it } from 'vitest';

import { formatScore, formatScoreShort } from './format.js';

describe('formatScore', () => {
  it('保留两位小数', () => {
    expect(formatScore(91)).toBe('9.10');
    expect(formatScore(80)).toBe('8.00');
    expect(formatScore(75)).toBe('7.50');
  });
});

describe('formatScoreShort', () => {
  it('整数不补小数位', () => {
    expect(formatScoreShort(80)).toBe('8');
    expect(formatScoreShort(100)).toBe('10');
    expect(formatScoreShort(0)).toBe('0');
  });

  it('非整数保留一位', () => {
    expect(formatScoreShort(91)).toBe('9.1');
    expect(formatScoreShort(75)).toBe('7.5');
    expect(formatScoreShort(719)).toBe('71.9');
  });
});
