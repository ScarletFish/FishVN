import { describe, expect, it } from 'vitest';
import { STATUS_KEYS, statusLabel, statusOptions } from './status.js';

describe('status 定义', () => {
  it('options 与 keys 同序同值', () => {
    expect(statusOptions().map((o) => o.value)).toEqual(STATUS_KEYS);
    expect(statusOptions(['playing', 'finished']).map((o) => o.value)).toEqual([
      'playing',
      'finished',
    ]);
  });

  it('每个状态都有 i18n 词条', () => {
    for (const key of STATUS_KEYS) {
      expect(statusLabel(key)).not.toBe(`status.${key}`);
    }
  });
});
