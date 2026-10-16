import { expect, test } from 'vitest';
import {
  applyIncoming,
  asText,
  bbcodeParagraphs,
  bbcodeSegments,
  conflictRows,
  stripBbcode,
} from './vn-merge.js';

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
  image: { id: 'cv75598', local_path: 'D:\\data\\covers\\vndb\\cv75598.jpg' },
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
    coverPath: '',
    ...over,
  };
}

test('BBCode 被剥掉，spoiler 与 raw 整块丢弃', () => {
  expect(stripBbcode('[spoiler]结局[/spoiler]正文')).toBe('正文');
  expect(stripBbcode('[url=https://a.b]链接文字[/url]')).toBe('链接文字');
  expect(stripBbcode('尾巴[hr]')).toBe('尾巴');
  expect(stripBbcode(null)).toBe('');
});

test('url 带标签与网址时拆成文本与链接段', () => {
  expect(bbcodeSegments('前文[url=https://a.b]链接[/url]后文')).toEqual([
    { type: 'text', text: '前文' },
    { type: 'link', text: '链接', href: 'https://a.b' },
    { type: 'text', text: '后文' },
  ]);
});

test('裸 url 标签以网址为文字', () => {
  expect(bbcodeSegments('[url]https://a.b[/url]')).toEqual([
    { type: 'link', text: 'https://a.b', href: 'https://a.b' },
  ]);
});

test('非 http(s) 的 url 段退回纯文本', () => {
  expect(bbcodeSegments('[url=javascript:alert(1)]点我[/url]')).toEqual([
    { type: 'text', text: '点我' },
  ]);
});

test('链接文字里的其余 BBCode 一并剥掉', () => {
  expect(bbcodeSegments('[url=https://a.b][b]粗体[/b]链接[/url]')).toEqual([
    { type: 'link', text: '粗体链接', href: 'https://a.b' },
  ]);
});

test('段落按空行拆分，空段与纯 BBCode 段丢弃', () => {
  expect(bbcodeParagraphs('第一段\n\n[spoiler]剧透[/spoiler]\n\n第二段[url=https://a.b]链[/url]')).toEqual([
    [{ type: 'text', text: '第一段' }],
    [
      { type: 'text', text: '第二段' },
      { type: 'link', text: '链', href: 'https://a.b' },
    ],
  ]);
  expect(bbcodeParagraphs(null)).toEqual([]);
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
  expect(next.coverPath).toBe('D:\\data\\covers\\vndb\\cv75598.jpg');
});

test('本地空封面直接采用 VNDB 的，不进冲突列表', () => {
  expect(conflictRows(form(), VN)).toEqual([]);
});

test('本地有封面时进冲突列表，勾选才替换', () => {
  const current = form({ coverPath: 'D:\\pics\\mine.jpg' });
  const keys = conflictRows(current, VN).map((r) => r.key);
  expect(keys).toEqual(['coverPath']);
  expect(applyIncoming(current, VN, new Set()).coverPath).toBe('D:\\pics\\mine.jpg');
  expect(applyIncoming(current, VN, new Set(['coverPath'])).coverPath).toBe(
    'D:\\data\\covers\\vndb\\cv75598.jpg'
  );
});

test('VNDB 没下到封面就不动本地', () => {
  const current = form({ coverPath: 'D:\\pics\\mine.jpg' });
  for (const local_path of [null, '']) {
    const failed = { ...VN, image: { id: 'cv75598', local_path } };
    expect(conflictRows(current, failed)).toEqual([]);
    expect(applyIncoming(current, failed, new Set()).coverPath).toBe('D:\\pics\\mine.jpg');
  }
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

test('多出来的键（characters / id / titles）不会进表单', () => {
  const incoming = { ...VN, characters: [{ id: 'c1', name: 'Saber' }] };
  const next = applyIncoming(form(), incoming, new Set());
  expect(Object.keys(next).sort()).toEqual(Object.keys(form()).sort());
});

test('面板显示：标签只取名字，其余转字符串', () => {
  expect(asText('tags', [{ name: 'A' }, { name: 'B' }])).toBe('A · B');
  expect(asText('rating', 86.3)).toBe('86.3');
  expect(asText('alias', null)).toBe('');
});