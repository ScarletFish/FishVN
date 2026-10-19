import { describe, expect, it } from 'vitest';
import { moveModule, setModuleEnabled, reorderModule } from './dashboard-layout.js';

const LAYOUT = [
  { id: 'continue_play', enabled: true },
  { id: 'stats', enabled: true },
];

describe('moveModule', () => {
  it('向下移动一格', () => {
    expect(moveModule(LAYOUT, 'continue_play', 1)).toEqual([
      { id: 'stats', enabled: true },
      { id: 'continue_play', enabled: true },
    ]);
  });

  it('向上移动一格', () => {
    expect(moveModule(LAYOUT, 'stats', -1)).toEqual([
      { id: 'stats', enabled: true },
      { id: 'continue_play', enabled: true },
    ]);
  });

  it('越界时原样返回不循环', () => {
    expect(moveModule(LAYOUT, 'continue_play', -1)).toEqual(LAYOUT);
    expect(moveModule(LAYOUT, 'stats', 1)).toEqual(LAYOUT);
  });

  it('未知 id 原样返回', () => {
    expect(moveModule(LAYOUT, 'nope', 1)).toEqual(LAYOUT);
  });
});

describe('setModuleEnabled', () => {
  it('只改目标模块的开关', () => {
    expect(setModuleEnabled(LAYOUT, 'stats', false)).toEqual([
      { id: 'continue_play', enabled: true },
      { id: 'stats', enabled: false },
    ]);
  });

  it('不改原数组', () => {
    setModuleEnabled(LAYOUT, 'stats', false);
    expect(LAYOUT[1].enabled).toBe(true);
  });
});

describe('reorderModule', () => {
  it('拖到下方模块的位置', () => {
    expect(reorderModule(LAYOUT, 'continue_play', 'stats')).toEqual([
      { id: 'stats', enabled: true },
      { id: 'continue_play', enabled: true },
    ]);
  });

  it('拖到上方模块的位置', () => {
    expect(reorderModule(LAYOUT, 'stats', 'continue_play')).toEqual([
      { id: 'stats', enabled: true },
      { id: 'continue_play', enabled: true },
    ]);
  });

  it('目标与来源相同或未知 id 原样返回', () => {
    expect(reorderModule(LAYOUT, 'stats', 'stats')).toEqual(LAYOUT);
    expect(reorderModule(LAYOUT, 'nope', 'stats')).toEqual(LAYOUT);
    expect(reorderModule(LAYOUT, 'stats', 'nope')).toEqual(LAYOUT);
  });
});
