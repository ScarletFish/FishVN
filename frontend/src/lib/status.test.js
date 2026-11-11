import { describe, expect, it } from 'vitest';
import { STATUS_KEYS, statusLabel, statusOptions } from './status.js';

describe('status 定义', () => {
  it('按传入的键生成选项，标签为中文文案', () => {
    expect(statusOptions(['playing', 'finished'])).toEqual([
      { value: 'playing', label: '游玩中' },
      { value: 'finished', label: '已完成' },
    ]);
  });

  it('每个状态都有中文词条', () => {
    expect(STATUS_KEYS.map(statusLabel)).toEqual([
      '游玩中',
      '计划中',
      '已完成',
      '已搁置',
      '已弃坑',
    ]);
  });
});
