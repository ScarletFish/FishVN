import { describe, expect, it } from 'vitest';
import {
  DEFAULT_STATUS,
  HOME_STATUS_KEYS,
  REPORT_STATUS_KEYS,
  STATUS_KEYS,
  statusLabel,
  statusOptions,
} from './status.js';

describe('status 定义', () => {
  it('常量列表自洽：无重复、默认项在内、首页为子集、报告顺序是排列', () => {
    expect(new Set(STATUS_KEYS).size).toBe(STATUS_KEYS.length);
    expect(STATUS_KEYS).toContain(DEFAULT_STATUS);
    for (const key of HOME_STATUS_KEYS) expect(STATUS_KEYS).toContain(key);
    expect([...REPORT_STATUS_KEYS].sort()).toEqual([...STATUS_KEYS].sort());
  });

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
