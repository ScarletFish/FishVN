// 目录高亮只是导航指示，判错就等于把用户送到错误的分区，因此边界与末项都要钉死。
// 未滚到底时只看「分区顶部有没有越过判定线」；滚到底时强制末项，因为最后几项常常凑不满一屏，
// 末项的顶部永远够不到判定线，不特判就永远亮不起来。点击态的保持由视图侧的 pinned 负责。
// 反例自检：把判定改成 entry.top >= line，第 3 条变红；去掉 atBottom 特判，末项那条变红。
import { describe, it, expect } from 'vitest';
import { SETTINGS_SECTIONS, pickActiveSection } from './settings-nav.js';

describe('pickActiveSection', () => {
  it('第一个分区尚未越过判定线时仍取第一个', () => {
    const tops = [
      { id: 'appearance', top: 80 },
      { id: 'content', top: 420 },
      { id: 'startup', top: 900 },
    ];
    expect(pickActiveSection(tops, 64)).toBe('appearance');
  });

  it('第二个分区越过判定线后取第二个', () => {
    const tops = [
      { id: 'appearance', top: -300 },
      { id: 'content', top: 20 },
      { id: 'startup', top: 500 },
    ];
    expect(pickActiveSection(tops, 64)).toBe('content');
  });

  it('分区顶部恰好落在判定线上算作已越过', () => {
    const tops = [
      { id: 'appearance', top: -300 },
      { id: 'content', top: 64 },
    ];
    expect(pickActiveSection(tops, 64)).toBe('content');
  });

  it('矮分区被整段滚过时取最后一个越线的', () => {
    const tops = [
      { id: 'appearance', top: -900 },
      { id: 'content', top: -620 },
      { id: 'startup', top: -40 },
      { id: 'vndb', top: 700 },
    ];
    expect(pickActiveSection(tops, 64)).toBe('startup');
  });

  it('最后一项越过判定线时取最后一项', () => {
    const tops = [
      { id: 'backup', top: -420 },
      { id: 'about', top: 40 },
    ];
    expect(pickActiveSection(tops, 64)).toBe('about');
  });

  it('未滚到底时最后一项没过判定线就不抢前一个分区', () => {
    const tops = [
      { id: 'backup', top: -200 },
      { id: 'about', top: 380 },
    ];
    expect(pickActiveSection(tops, 64, false)).toBe('backup');
  });

  it('滚到底时取最后一项，即使它的顶部还在判定线下方', () => {
    const tops = [
      { id: 'backup', top: -200 },
      { id: 'about', top: 380 },
    ];
    expect(pickActiveSection(tops, 64, true)).toBe('about');
  });

  it('没有分区时返回 null', () => {
    expect(pickActiveSection([], 64)).toBe(null);
  });
});

describe('SETTINGS_SECTIONS', () => {
  it('六个分区按滚动流顺序排列且 id 唯一', () => {
    expect(SETTINGS_SECTIONS.map((s) => s.id)).toEqual([
      'appearance',
      'content',
      'startup',
      'vndb',
      'backup',
      'about',
    ]);
  });
});
