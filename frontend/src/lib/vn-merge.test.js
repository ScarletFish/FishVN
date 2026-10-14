import { expect, test } from 'vitest';
import { applyIncoming, asText, conflictRows, stripBbcode } from './vn-merge.js';

const VN = {
  id: 'v11',
  title: 'Fate/stay night',
  alttitle: 'フェイト／ステイナイト',
  released: '2004-01-30',
  rating: 86.3,
  votecount: 19334,
  description: 'In Fuyuki, a secret battle rages. See [url=https://x.jp]more[/url].',
  titles: [{ lang: 'ja', title: 'フェイト／ステイナイト' }],
  tags: [{ id: 'g1', name: 'Gore', rating: 3 }],
  developers: [{ id: 'd1', name: 'TYPE-MOON', original: 'タイプムーン' }],
};

function form(over = {}) {
  return {
    name: '',
    alias: '',
    released: '',
    rating: '',
    votecount: '',
    description: '',
    tags: [],
    developers: '',
    ...over,
  };
}

test('BBCode 被剥掉，spoiler 与 raw 整块丢弃', () => {
  expect(stripBbcode('[spoiler]结局[/spoiler]正文')).toBe('正文');
  expect(stripBbcode('[url=https://a.b]链接文字[/url]')).toBe('链接文字');
  expect(stripBbcode('尾巴[hr]')).toBe('尾巴');
  expect(stripBbcode(null)).toBe('');
});

test('简介里的 BBCode 在读入时就已剥掉', () => {
  expect(applyIncoming(form(), VN, new Set()).description).toBe(
    'In Fuyuki, a secret battle rages. See more.'
  );
});

test('全空的表单没有冲突，全部直接填', () => {
  expect(conflictRows(form(), VN)).toEqual([]);
  const next = applyIncoming(form(), VN, new Set());
  expect(next.name).toBe('Fate/stay night');
  expect(next.tags.map((t) => t.name)).toEqual(['Gore']);
  expect(next.developers).toBe('TYPE-MOON');
});

test('一样的值不算冲突', () => {
  const filled = applyIncoming(form(), VN, new Set());
  expect(conflictRows(filled, VN)).toEqual([]);
});

test('有差异的字段进冲突列表，空的不进', () => {
  const current = form({
    name: 'Fate/stay night',
    rating: '80',
    votecount: '999',
    tags: [{ name: 'Old' }],
  });
  const keys = conflictRows(current, VN).map((r) => r.key);
  expect(keys.sort()).toEqual(['rating', 'tags', 'votecount']);
});

/** 用户默认意图是覆盖，但没勾的字段必须保持原样 */
test('只有选中的字段取 VNDB 的值', () => {
  const current = form({ rating: '80', votecount: '999' });
  const next = applyIncoming(current, VN, new Set(['rating']));
  expect(next.rating).toBe('86.3');
  expect(next.votecount).toBe('999');
});

test('标签顺序不同不算冲突', () => {
  const current = form({
    tags: [
      { name: 'B' },
      { name: 'A' },
    ],
  });
  const vn = { ...VN, tags: [{ name: 'A' }, { name: 'B' }] };
  expect(conflictRows(current, vn)).toEqual([]);
});

test('开发商按名字集合比，顺序不同不算冲突', () => {
  const current = form({ developers: 'TYPE-MOON, 同人社团' });
  const vn = { ...VN, developers: [{ name: '同人社团' }, { name: 'TYPE-MOON' }] };
  expect(conflictRows(current, vn)).toEqual([]);
});

test('评分 80 与 80.0 不算冲突', () => {
  expect(conflictRows(form({ rating: '80' }), { ...VN, rating: 80 })).toEqual([]);
});

test('开发商取 name，为空回落 original', () => {
  const next = applyIncoming(form(), {
    ...VN,
    developers: [{ name: null, original: 'タイプムーン' }, { name: null, original: null }],
  }, new Set());
  expect(next.developers).toBe('タイプムーン');
});

test('面板显示：标签只取名字，其余转字符串', () => {
  expect(asText('tags', [{ name: 'A' }, { name: 'B' }])).toBe('A · B');
  expect(asText('rating', 86.3)).toBe('86.3');
  expect(asText('alias', null)).toBe('');
});