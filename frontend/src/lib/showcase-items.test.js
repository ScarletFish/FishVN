import { describe, expect, it } from 'vitest';
import { SHOWCASE_LIMIT, applyAmbience, buildShowcaseItems, stageDescriptor } from './showcase-items.js';

function row(id, extra = {}) {
  return { id, name: id, last_played_at: 0, ...extra };
}

describe('buildShowcaseItems', () => {
  it('有游玩记录的排在未游玩之前', () => {
    const list = buildShowcaseItems([
      row('在玩', { name: '在玩' }),
      row('最近', { last_played_at: 100, name: '最近' }),
    ]);
    expect(list.map((i) => i.id)).toEqual(['最近', '在玩']);
  });

  it('最近游玩按时间倒序', () => {
    const list = buildShowcaseItems([
      row('早', { last_played_at: 10 }),
      row('晚', { last_played_at: 90 }),
      row('中', { last_played_at: 50 }),
    ]);
    expect(list.map((i) => i.id)).toEqual(['晚', '中', '早']);
  });

  it('无记录时按最近改状态倒序，空值落尾', () => {
    const list = buildShowcaseItems([
      row('c', { name: 'c' }),
      row('b', { name: 'b', state_modified_at: 100 }),
      row('a', { name: 'a', state_modified_at: 300 }),
    ]);
    expect(list.map((i) => i.id)).toEqual(['a', 'b', 'c']);
  });

  it('同档按名称排序', () => {
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
  it('自定义图优先于封面，观感跟随展厅设置', () => {
    const desc = stageDescriptor({
      cover_path: 'cover.jpg',
      ambience_path: 'cg.png',
      showcase_mode: 'clear',
    });
    expect(desc.path).toBe('cg.png');
    expect(desc.mode).toBe('clear');
  });

  it('展厅观感独立于横幅观感', () => {
    expect(
      stageDescriptor({ ambience_mode: 'clear', showcase_mode: 'soft' }).mode,
    ).toBe('soft');
    expect(
      stageDescriptor({ ambience_mode: 'soft', showcase_mode: 'clear' }).mode,
    ).toBe('clear');
  });

  it('无自定义图时回落封面', () => {
    const desc = stageDescriptor({ cover_path: 'cover.jpg', showcase_mode: 'soft' });
    expect(desc.path).toBe('cover.jpg');
    expect(desc.mode).toBe('soft');
  });

  it('空值与裸 null 走默认取景', () => {
    const desc = stageDescriptor(null);
    expect(desc.path).toBe('');
    expect(desc.mode).toBe('soft');
    expect(desc.showcaseFocal).toBe('50% 22%');
    expect(desc.coverFocal).toBe('50% 50%');
    const blank = stageDescriptor({
      ambience_path: '  ',
      cover_path: 'c.jpg',
      ambience_focal_x: null,
      cover_focal_y: null,
    });
    expect(blank.path).toBe('c.jpg');
    expect(blank.showcaseFocal).toBe('50% 22%');
    expect(blank.coverFocal).toBe('50% 50%');
    const spaces = stageDescriptor({ showcase_focal_x: '  ', showcase_focal_y: '  ' });
    expect(spaces.showcaseFocal).toBe('50% 22%');
  });

  it('保留作品设定的取景位置', () => {
    const desc = stageDescriptor({
      showcase_focal_x: 30,
      showcase_focal_y: 40,
      cover_focal_x: 20,
      cover_focal_y: 60,
    });
    expect(desc.showcaseFocal).toBe('30% 40%');
    expect(desc.coverFocal).toBe('20% 60%');
  });

  it('展厅焦点独立于横幅焦点', () => {
    const desc = stageDescriptor({
      ambience_focal_x: 30,
      ambience_focal_y: 40,
      showcase_focal_x: 70,
      showcase_focal_y: 80,
    });
    expect(desc.showcaseFocal).toBe('70% 80%');
  });

  it('按是否设置了自定义图给出菜单当前值', () => {
    expect(stageDescriptor({ ambience_path: 'cg.png' }).source).toBe('custom');
    expect(stageDescriptor({ ambience_path: '  ' }).source).toBe('follow');
    expect(stageDescriptor({ cover_path: 'cover.jpg' }).source).toBe('follow');
    expect(stageDescriptor(null).source).toBe('follow');
  });
});

describe('applyAmbience', () => {
  it('把元数据覆盖回展厅行，封面字段改名', () => {
    const item = { id: 'a', cover_path: 'old.jpg', ambience_path: null, ambience_mode: 'soft' };
    const next = applyAmbience(item, {
      image_path: 'new.jpg',
      ambience_path: 'cg.png',
      ambience_mode: 'clear',
      showcase_focal_x: 70,
      showcase_focal_y: 80,
      showcase_mode: 'clear',
    });
    expect(next).toEqual({
      id: 'a',
      cover_path: 'new.jpg',
      ambience_path: 'cg.png',
      ambience_mode: 'clear',
      showcase_focal_x: 70,
      showcase_focal_y: 80,
      showcase_mode: 'clear',
    });
    expect(item.cover_path).toBe('old.jpg');
  });

  it('置空的路径落为跟随封面', () => {
    const next = applyAmbience(
      { id: 'a', cover_path: 'c.jpg', ambience_path: 'cg.png', ambience_mode: 'clear' },
      { image_path: 'c.jpg', ambience_path: null, ambience_mode: null },
    );
    expect(next.ambience_path).toBeNull();
    expect(next.ambience_mode).toBe('soft');
  });

  it('缺失的焦点沿用原值', () => {
    const next = applyAmbience(
      {
        id: 'a',
        showcase_focal_x: 70,
        showcase_focal_y: 80,
        showcase_mode: 'clear',
      },
      { image_path: null, ambience_path: null },
    );
    expect(next.showcase_focal_x).toBe(70);
    expect(next.showcase_focal_y).toBe(80);
    expect(next.showcase_mode).toBe('clear');
    expect(next.cover_path).toBeNull();
  });

  it('无行或无元数据时原样返回', () => {
    expect(applyAmbience(null, { image_path: 'a.jpg' })).toBeNull();
    const item = { id: 'a' };
    expect(applyAmbience(item, null)).toBe(item);
  });
});
