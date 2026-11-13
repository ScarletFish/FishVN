import { describe, expect, it } from 'vitest';
import { SHOWCASE_LIMIT, buildShowcaseItems, stageDescriptor } from './showcase-items.js';

function row(id, extra = {}) {
  return { id, name: id, status: 'wishlist', last_played_at: 0, ...extra };
}

describe('buildShowcaseItems', () => {
  it('按最近游玩、游玩中、其余状态的顺序排列', () => {
    const list = buildShowcaseItems([
      row('计划', { status: 'wishlist', name: '计划' }),
      row('在玩', { status: 'playing', name: '在玩' }),
      row('最近', { status: 'finished', last_played_at: 100, name: '最近' }),
    ]);
    expect(list.map((i) => i.id)).toEqual(['最近', '在玩', '计划']);
  });

  it('最近游玩按时间倒序', () => {
    const list = buildShowcaseItems([
      row('早', { last_played_at: 10 }),
      row('晚', { last_played_at: 90 }),
      row('中', { last_played_at: 50 }),
    ]);
    expect(list.map((i) => i.id)).toEqual(['晚', '中', '早']);
  });

  it('同组按名称排序', () => {
    const list = buildShowcaseItems([
      row('b', { name: 'b' }),
      row('a', { name: 'a' }),
      row('c', { name: 'c' }),
    ]);
    expect(list.map((i) => i.name)).toEqual(['a', 'b', 'c']);
  });

  it('按 id 去重并过滤无效行', () => {
    const list = buildShowcaseItems([
      row('a', { name: '首个' }),
      row('a', { name: '重复' }),
      null,
      { name: '无 id' },
      row('b'),
    ]);
    expect(list).toHaveLength(2);
    expect(list[0].name).toBe('首个');
  });

  it('限量且不改原数组', () => {
    const source = [row('a'), row('b'), row('c')];
    expect(buildShowcaseItems(source, 2)).toHaveLength(2);
    expect(source).toHaveLength(3);
    expect(buildShowcaseItems(source, 0)).toEqual([]);
    expect(buildShowcaseItems(source, SHOWCASE_LIMIT)).toHaveLength(3);
  });

  it('非数组输入返回空', () => {
    expect(buildShowcaseItems(null)).toEqual([]);
    expect(buildShowcaseItems(undefined)).toEqual([]);
  });
});

describe('stageDescriptor', () => {
  it('自定义图优先于封面，观感跟随作品设置', () => {
    const desc = stageDescriptor({
      cover_path: 'cover.jpg',
      ambience_path: 'cg.png',
      ambience_mode: 'clear',
    });
    expect(desc.path).toBe('cg.png');
    expect(desc.mode).toBe('clear');
  });

  it('无自定义图时回落封面', () => {
    const desc = stageDescriptor({ cover_path: 'cover.jpg', ambience_mode: 'soft' });
    expect(desc.path).toBe('cover.jpg');
    expect(desc.mode).toBe('soft');
  });

  it('空值与裸 null 走默认取景', () => {
    const desc = stageDescriptor(null);
    expect(desc.path).toBe('');
    expect(desc.mode).toBe('soft');
    expect(desc.focal).toBe('50% 22%');
    expect(desc.coverFocal).toBe('50% 50%');
    const blank = stageDescriptor({
      ambience_path: '  ',
      cover_path: 'c.jpg',
      ambience_focal_x: null,
      cover_focal_y: null,
    });
    expect(blank.path).toBe('c.jpg');
    expect(blank.focal).toBe('50% 22%');
    expect(blank.coverFocal).toBe('50% 50%');
  });

  it('保留作品设定的取景位置', () => {
    const desc = stageDescriptor({
      ambience_focal_x: 30,
      ambience_focal_y: 40,
      cover_focal_x: 20,
      cover_focal_y: 60,
    });
    expect(desc.focal).toBe('30% 40%');
    expect(desc.coverFocal).toBe('20% 60%');
  });
});
