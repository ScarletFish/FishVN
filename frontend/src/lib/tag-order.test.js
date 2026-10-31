import { describe, expect, it, beforeEach } from 'vitest';
import { classifyTags, setTagGroups, tagCategory, tagScope } from './tag-order.svelte.js';

const tag = (id, name, extra = {}) => ({
  id,
  name,
  category: 'cont',
  rating: 1,
  vn_count: 100,
  spoiler: 0,
  lie: false,
  ...extra,
});

const THEME_A = tag('g96', 'Romance', { rating: 2.5, vn_count: 20000 });
const THEME_B = tag('g2', 'Fantasy', { rating: 2.2, vn_count: 13000 });
const THEME_C = tag('g147', 'Drama', { rating: 1.8, vn_count: 12000 });
const PLOT_A = tag('g148', 'Multiple Endings', { rating: 2.1, vn_count: 12000 });
const PLOT_B = tag('g606', 'Branching Plot', { rating: 1.5, vn_count: 2800 });
const CHARACTER_A = tag('g133', 'Male Protagonist', { rating: 2.4, vn_count: 23000 });
const SETTING_A = tag('g305', 'School', { rating: 2.0, vn_count: 9000 });
const STYLE_A = tag('g35', 'RPG', { rating: 1.2, vn_count: 1000 });
const TECH_A = tag('g32', 'ADV', { rating: 3, vn_count: 36000 });
const TECH_B = tag('g452', 'Skip Scenes', { rating: 2.5, vn_count: 8000 });
const WILDCARD = tag('g728', 'Protagonist with a Face', { rating: 3.0, vn_count: 12305 });
const RARE_HIGH = tag('g3005', 'Major Antagonist', { rating: 3.0, vn_count: 491 });
const CHARACTER_B = tag('g692', 'Proactive Protagonist', { rating: 2.9, vn_count: 762 });
const CUSTOM_A = { name: '神作' };
const CUSTOM_B = { name: '系列补完' };
const ERO_A = tag('g304', 'High Sexual Content', { category: 'ero', rating: 2.9, vn_count: 11000 });
const SPOILER_A = tag('g999', '剧透结局', { spoiler: 1 });
const CHARACTER_SPOILER = tag('g505', '剧透角色', { spoiler: 2 });
const MILD_SPOILER = tag('g501', '轻微剧透', { spoiler: 1 });
const MAJOR_SPOILER = tag('g502', '严重剧透', { spoiler: 2 });
const LIE_A = tag('g503', '虚假标签', { lie: true });
const NEW_TAG = tag('g99999', 'VNDB 新标签', { rating: 2.6, vn_count: 5000 });

const GROUPS = {
  roots: {
    g96: 'g1',
    g2: 'g1',
    g147: 'g1',
    g148: 'g24',
    g606: 'g24',
    g133: 'g20',
    g305: 'g22',
    g35: 'g674',
    g304: 'g24',
    g999: 'g1',
    g728: 'g20',
    g3005: 'g20',
    g692: 'g20',
    g505: 'g20',
  },
  excluded: ['g32', 'g452'],
};

const names = (list) => list.map((t) => t.name);

beforeEach(() => {
  setTagGroups(GROUPS);
});

describe('tagCategory', () => {
  it('按五个根归类，字典未收录的 id 归其他', () => {
    expect(tagCategory(THEME_A)).toBe('theme');
    expect(tagCategory(CHARACTER_A)).toBe('character');
    expect(tagCategory(SETTING_A)).toBe('setting');
    expect(tagCategory(PLOT_A)).toBe('plot');
    expect(tagCategory(STYLE_A)).toBe('style');
    expect(tagCategory(NEW_TAG)).toBe('other');
  });

  it('技术向单列一类', () => {
    expect(tagCategory(TECH_A)).toBe('tech');
    expect(tagCategory(TECH_B)).toBe('tech');
  });

  it('无 id 的标签归自定义', () => {
    expect(tagCategory(CUSTOM_A)).toBe('custom');
    expect(tagCategory({ tag: '手输' })).toBe('custom');
    expect(tagCategory(null)).toBe('custom');
  });

  it('接受裸 id，与对象判定一致', () => {
    expect(tagCategory('g96')).toBe('theme');
    expect(tagCategory('g32')).toBe('tech');
    expect(tagCategory('')).toBe('custom');
  });
});

describe('tagScope', () => {
  it('技术向标签在任何界面都不显示', () => {
    expect(tagScope('g32')).toBe('hidden');
    expect(tagScope('g452')).toBe('hidden');
  });

  it('题材与角色两处都显示', () => {
    expect(tagScope('g96')).toBe('full');
    expect(tagScope('g133')).toBe('full');
  });

  it('舞台情节风格只进详情页', () => {
    expect(tagScope('g305')).toBe('detail');
    expect(tagScope('g148')).toBe('detail');
    expect(tagScope('g35')).toBe('detail');
  });

  it('手输标签与字典未收录的 id 按可统计处理', () => {
    expect(tagScope(undefined)).toBe('full');
    expect(tagScope('g99999')).toBe('full');
  });
});

describe('classifyTags 的展示顺序', () => {
  it('未加载分组表时回落按权重排序', () => {
    setTagGroups(null);
    const out = classifyTags([THEME_B, THEME_A, PLOT_B]);
    expect(names(out.shown)).toEqual(['Romance', 'Fantasy', 'Branching Plot']);
  });

  it('自定义组整组前置，其余五组按轮询顺序交错', () => {
    const out = classifyTags([
      THEME_A,
      THEME_B,
      THEME_C,
      PLOT_A,
      PLOT_B,
      CHARACTER_A,
      SETTING_A,
      STYLE_A,
      CUSTOM_A,
      CUSTOM_B,
    ]);
    expect(names(out.shown)).toEqual([
      '神作',
      '系列补完',
      'Romance',
      'Multiple Endings',
      'Male Protagonist',
      'School',
      'RPG',
      'Fantasy',
      'Branching Plot',
      'Drama',
    ]);
  });

  it('自定义标签保持录入顺序且不参与权重竞争', () => {
    const out = classifyTags([THEME_A, CUSTOM_B, CUSTOM_A]);
    expect(names(out.shown)).toEqual(['系列补完', '神作', 'Romance']);
  });

  it('组内按权重降序', () => {
    const out = classifyTags([THEME_C, THEME_A, THEME_B]);
    expect(names(out.shown)).toEqual(['Romance', 'Fantasy', 'Drama']);
  });

  it('同分时稀有标签靠前，通配标签压后', () => {
    const out = classifyTags([WILDCARD, RARE_HIGH]);
    expect(names(out.shown)).toEqual(['Major Antagonist', 'Protagonist with a Face']);
  });

  it('rating 差距优先于稀有度：3.0 通配仍高于 2.9 特色', () => {
    const out = classifyTags([WILDCARD, CHARACTER_B]);
    expect(names(out.shown)).toEqual(['Protagonist with a Face', 'Proactive Protagonist']);
  });

  it('未命中的同步标签落入其他组，沉底追加', () => {
    const out = classifyTags([NEW_TAG, THEME_A, PLOT_A]);
    expect(names(out.shown)).toEqual(['Romance', 'Multiple Endings', 'VNDB 新标签']);
  });

  it('前八条覆盖多个属性维度', () => {
    const out = classifyTags([
      THEME_A,
      THEME_B,
      THEME_C,
      THEME_C,
      PLOT_A,
      PLOT_B,
      CHARACTER_A,
      SETTING_A,
      STYLE_A,
      NEW_TAG,
    ]).shown.slice(0, 8);
    expect(names(out)).toEqual([
      'Romance',
      'Multiple Endings',
      'Male Protagonist',
      'School',
      'RPG',
      'Fantasy',
      'Branching Plot',
      'Drama',
    ]);
  });
});

describe('classifyTags 的隐藏判定', () => {
  it('排除技术支标签与错挂父链的技术功能', () => {
    const out = classifyTags([TECH_A, TECH_B, THEME_A]);
    expect(names(out.shown)).toEqual(['Romance']);
  });

  it('R18 开关开启时排除成人向标签，关闭时保留', () => {
    expect(names(classifyTags([THEME_A, ERO_A], { hideR18: true }).shown)).toEqual(['Romance']);
    expect(names(classifyTags([THEME_A, ERO_A]).shown)).toEqual([
      'Romance',
      'High Sexual Content',
    ]);
  });

  it('过滤 lie 标签', () => {
    const out = classifyTags([LIE_A, THEME_A], { spoilerLevel: 'show_all' });
    expect(names(out.shown)).toEqual(['Romance']);
  });

  it('排除池按 id 命中字典标签', () => {
    const out = classifyTags([THEME_A, CHARACTER_A], { excludedTags: ['g133'] });
    expect(names(out.shown)).toEqual(['Romance']);
  });

  it('排除池按名称命中手输标签', () => {
    const out = classifyTags([THEME_A, CUSTOM_A], { excludedTags: ['神作'] });
    expect(names(out.shown)).toEqual(['Romance']);
  });

  it('名称形条目不命中有 id 的字典标签', () => {
    const out = classifyTags([THEME_A], { excludedTags: ['Romance'] });
    expect(names(out.shown)).toEqual(['Romance']);
  });

  it('排除同时作用于剧透折叠列表', () => {
    const out = classifyTags([THEME_A, SPOILER_A, CHARACTER_SPOILER], { excludedTags: ['g505'] });
    expect(names(out.spoilers)).toEqual(['剧透结局']);
    expect(names(out.shown)).toEqual(['Romance']);
  });
});

describe('classifyTags 的剧透折叠', () => {
  it('默认按隐藏全部剧透拆分可见与折叠', () => {
    const out = classifyTags([THEME_A, MILD_SPOILER, MAJOR_SPOILER]);
    expect(names(out.shown)).toEqual(['Romance']);
    expect(names(out.spoilers)).toEqual(['轻微剧透', '严重剧透']);
  });

  it('仅隐藏严重剧透时轻微剧透照常展示', () => {
    const out = classifyTags([THEME_A, MILD_SPOILER, MAJOR_SPOILER], {
      spoilerLevel: 'hide_major',
    });
    expect(names(out.shown)).toEqual(['Romance', '轻微剧透']);
    expect(names(out.spoilers)).toEqual(['严重剧透']);
  });

  it('显示全部剧透时折叠列表为空', () => {
    const out = classifyTags([THEME_A, MILD_SPOILER, MAJOR_SPOILER], {
      spoilerLevel: 'show_all',
    });
    expect(names(out.shown)).toEqual(['Romance', '轻微剧透', '严重剧透']);
    expect(out.spoilers).toEqual([]);
  });

  it('折叠的剧透标签按同一套分组轮询排序', () => {
    const out = classifyTags([THEME_A, SPOILER_A, CHARACTER_SPOILER, PLOT_A]);
    expect(names(out.spoilers)).toEqual(['剧透结局', '剧透角色']);
    expect(names(out.shown)).toEqual(['Romance', 'Multiple Endings']);
  });

  it('揭示剧透不改动展示顺序，剧透标签始终单列', () => {
    const out = classifyTags([THEME_A, MILD_SPOILER, MAJOR_SPOILER], {
      revealSpoilers: true,
    });
    expect(names(out.shown)).toEqual(['Romance']);
    expect(names(out.spoilers)).toEqual(['轻微剧透', '严重剧透']);
  });
});

describe('classifyTags 的排除计数', () => {
  it('只统计否则会可见的排除项', () => {
    const out = classifyTags([THEME_A, CHARACTER_A, CUSTOM_A], {
      excludedTags: ['g133', '神作'],
    });
    expect(out.poolCount).toBe(2);
  });

  it('成人标签被 R18 开关挡下时不计入', () => {
    const out = classifyTags([THEME_A, ERO_A], { hideR18: true, excludedTags: ['g304'] });
    expect(names(out.shown)).toEqual(['Romance']);
    expect(out.poolCount).toBe(0);
  });

  it('技术向标签被显示范围挡下时不计入', () => {
    const out = classifyTags([THEME_A, TECH_A], { excludedTags: ['g32'] });
    expect(names(out.shown)).toEqual(['Romance']);
    expect(out.poolCount).toBe(0);
  });

  it('剧透标签折叠时不重复计入，展开后计入', () => {
    const collapsed = classifyTags([THEME_A, CHARACTER_SPOILER], { excludedTags: ['g505'] });
    expect(collapsed.poolCount).toBe(0);
    const revealed = classifyTags([THEME_A, CHARACTER_SPOILER], {
      excludedTags: ['g505'],
      revealSpoilers: true,
    });
    expect(revealed.poolCount).toBe(1);
    expect(names(revealed.shown)).toEqual(['Romance']);
  });

  it('虚假标签不计入', () => {
    const out = classifyTags([THEME_A, LIE_A], { excludedTags: ['g503'] });
    expect(out.poolCount).toBe(0);
  });
});

describe('classifyTags 的空输入', () => {
  it('空列表返回空结果', () => {
    expect(classifyTags([])).toEqual({ shown: [], spoilers: [], poolCount: 0 });
    expect(classifyTags(undefined, { hideR18: true })).toEqual({
      shown: [],
      spoilers: [],
      poolCount: 0,
    });
  });
});
