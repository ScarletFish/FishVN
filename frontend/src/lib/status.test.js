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
  it('主列表无重复', () => {
    expect(new Set(STATUS_KEYS).size).toBe(STATUS_KEYS.length);
  });

  it('默认状态在主列表内', () => {
    expect(STATUS_KEYS).toContain(DEFAULT_STATUS);
  });

  it('首页状态是主列表的子集', () => {
    for (const key of HOME_STATUS_KEYS) expect(STATUS_KEYS).toContain(key);
  });

  it('同步报告顺序是主列表的一个排列（不重不漏）', () => {
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
