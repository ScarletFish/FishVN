import { describe, it, expect, vi } from 'vitest';

vi.mock('./tag-label.svelte.js', () => ({
  tagLabel: (v) => (typeof v === 'object' ? v.name || v.tag || '' : ''),
  tagLabelByName: (n) => n,
  traitLabel: (v) => (typeof v === 'object' ? v.name || v.id : v),
  traitKeyLabel: (k) => ({ i8: '傲娇', g1: '黑发', g2: '长发' })[k] ?? k,
}));

import { buildQueryMarkdown } from './query-export.js';

// 测试用标签映射：直接回显 id / 名称，避免依赖全局字典。
const labels = {
  tag: (id) => `标签${id}`,
  trait: (t) => t.name || t.id,
  status: (k) => k,
};

const NOW = new Date('2026-11-08T18:30:00');

describe('buildQueryMarkdown', () => {
  it('导出作品：头部语境、筛选摘要、字段均正确', () => {
    const md = buildQueryMarkdown({
      subject: 'game',
      ui: {
        subject: 'game',
        tags: { groups: [['t1', 't2']], exclude: [], mode: 'any', open: true, enabled: true },
        status: { values: ['finished'], open: true, enabled: true },
      },
      rows: [
        {
          name: '测试作',
          vndb_id: 'v123',
          alias: 'Alias',
          status: 'finished',
          user_rating: 90,
          vndb_rating: 78.1,
          va_names: 'Morio Toshihiko 森川智之,能登麻美子 能登 麻美子',
          va_original_names: 'Morio Toshihiko,能登麻美子',
        },
      ],
      labels,
      now: NOW,
    });
    expect(md).toContain('# FishVN 筛选结果 · 作品');
    expect(md).toContain(
      '以下是在筛选条件（标签：含任一：标签t1、标签t2；状态：finished）下命中的 1 个作品。',
    );
    expect(md).toContain('VNDB：https://vndb.org/v123');
    expect(md).toContain('我的评分：9.0');
    expect(md).toContain('VNDB 评分：7.81');
    expect(md).toContain('主要声优：Morio Toshihiko、能登麻美子');
  });

  it('导出角色：特性按 group_name 分层', () => {
    const md = buildQueryMarkdown({
      subject: 'character',
      ui: { subject: 'character', role: { values: ['main'], open: true, enabled: true } },
      rows: [
        {
          name: '角色A',
          game_title: '作X',
          original: 'Orig',
          role: 'primary',
          gender: ['f'],
          traits: [
            { id: 'i1', name: '傲娇', group_name: 'Personality' },
            { id: 'i2', name: '黑发', group_name: 'Hair' },
            { id: 'i3', name: '性行为', group_name: 'Engages in (Sexual)' },
          ],
        },
      ],
      labels,
      now: NOW,
    });
    expect(md).toContain('# FishVN 筛选结果 · 角色');
    expect(md).toContain('## Orig（出自《作X》）');
    expect(md).toContain('筛选条件（戏份：主角）');
    expect(md).not.toContain('显示名：');
    expect(md).toContain('戏份：主要角色');
    expect(md).toContain('性别：女');
    expect(md).toContain('- 性格：傲娇');
    expect(md).toContain('- 毛发：黑发');
    expect(md).not.toContain('性行为');
    expect(md).not.toContain('Engages in (Sexual)');
  });

  it('转义标题里的行首 #，避免破坏标题层级', () => {
    const md = buildQueryMarkdown({
      subject: 'game',
      ui: { subject: 'game' },
      rows: [{ name: '#奇怪名' }],
      labels,
      now: NOW,
    });
    expect(md).toContain('## \\#奇怪名');
    expect(md).not.toContain('## #奇怪名');
  });

  it('未设条件时摘要写「未设条件」', () => {
    const md = buildQueryMarkdown({
      subject: 'game',
      ui: { subject: 'game' },
      rows: [],
      labels,
      now: NOW,
    });
    expect(md).toContain('筛选条件（未设条件）');
  });

  it('标签以名称存储时摘要仍能反查出条件', () => {
    const md = buildQueryMarkdown({
      subject: 'game',
      ui: {
        subject: 'game',
        tags: { groups: [['百合', '治愈']], exclude: ['猎奇'], mode: 'any' },
      },
      rows: [{ name: '作A' }],
      now: NOW,
    });
    expect(md).toContain('筛选条件（标签：含任一：百合、治愈；排除：猎奇）');
  });

  it('特性以 id 字符串存储时摘要反查为名字，未知 id 保留原值', () => {
    const md = buildQueryMarkdown({
      subject: 'character',
      ui: {
        subject: 'character',
        traits: { groups: [['i8', 'g1']], exclude: ['g2', 'x9'], mode: 'any' },
      },
      rows: [{ name: '角色A' }],
      now: NOW,
    });
    expect(md).toContain('筛选条件（特性：含任一：傲娇、黑发；排除：长发、x9）');
  });

  it('多分组条件按组分别表述，不压平成一句', () => {
    const md = buildQueryMarkdown({
      subject: 'game',
      ui: { subject: 'game', tags: { groups: [['A'], ['B', 'C']], exclude: [], mode: 'all' } },
      rows: [{ name: '作A' }],
      labels,
      now: NOW,
    });
    expect(md).toContain('筛选条件（标签：含全部：标签A；含全部：标签B、标签C）');
  });

  it('声优条件有名字映射时写名字，缺映射时退回人数', () => {
    const ui = { subject: 'game', va: { values: ['s1', 's2'] } };
    const named = buildQueryMarkdown({
      subject: 'game',
      ui,
      rows: [{ name: '作A' }],
      labels: { ...labels, va: (id) => ({ s1: '声优A', s2: '声优B' })[id] },
      now: NOW,
    });
    expect(named).toContain('筛选条件（声优：声优A、声优B）');
    const counted = buildQueryMarkdown({
      subject: 'game',
      ui,
      rows: [{ name: '作A' }],
      labels,
      now: NOW,
    });
    expect(counted).toContain('筛选条件（声优：2 位）');
  });
});
