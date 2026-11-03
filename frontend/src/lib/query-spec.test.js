import { describe, expect, it } from 'vitest';
import {
  dimFilled,
  emptyQueryUi,
  hasActive,
  uiToSpec,
} from './query-spec.js';

describe('query-spec', () => {
  it('空状态产生空 rows', () => {
    const spec = uiToSpec(emptyQueryUi());
    expect(spec).toEqual({ subject: 'game', rows: [], spoilerThreshold: null });
  });

  it('subject 与 spoilerThreshold 进 spec，Infinity 归 null', () => {
    const ui = emptyQueryUi();
    ui.subject = 'character';
    expect(uiToSpec(ui, Infinity)).toEqual({
      subject: 'character',
      rows: [],
      spoilerThreshold: null,
    });
    expect(uiToSpec(ui, 2).spoilerThreshold).toBe(2);
    expect(uiToSpec(emptyQueryUi(), 1).spoilerThreshold).toBe(1);
  });

  it('特征多组共享 mode，排除集生成 trait_exclude', () => {
    const ui = emptyQueryUi();
    ui.traits.open = true;
    ui.traits.mode = 'all';
    ui.traits.groups = [['t100', 't200'], ['t300']];
    ui.traits.exclude = ['t400'];
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'trait_include', mode: 'all', traits: ['t100', 't200'] },
      { kind: 'trait_include', mode: 'all', traits: ['t300'] },
      { kind: 'trait_exclude', traits: ['t400'] },
    ]);
  });

  it('role/gender/charname 生成对应行', () => {
    const ui = emptyQueryUi();
    ui.role.open = true;
    ui.role.values = ['main', 'primary'];
    ui.gender.open = true;
    ui.gender.values = ['f'];
    ui.charname.open = true;
    ui.charname.value = '  白毛  ';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'role', values: ['main', 'primary'] },
      { kind: 'gender', values: ['f'] },
      { kind: 'name_contains', value: '白毛' },
    ]);
  });

  it('va 生成 staff_ids 行', () => {
    const ui = emptyQueryUi();
    ui.va.open = true;
    expect(dimFilled('va', ui.va)).toBe(false);
    ui.va.values = ['s113'];
    expect(dimFilled('va', ui.va)).toBe(true);
    expect(uiToSpec(ui).rows).toEqual([{ kind: 'va', staff_ids: ['s113'] }]);
    ui.va.enabled = false;
    expect(uiToSpec(ui).rows).toEqual([]);
  });

  it('dimFilled 覆盖角色维度', () => {
    const ui = emptyQueryUi();
    ui.traits.open = true;
    expect(dimFilled('traits', ui.traits)).toBe(false);
    ui.traits.groups = [['t100']];
    expect(dimFilled('traits', ui.traits)).toBe(true);
    ui.role.open = true;
    ui.role.values = ['main'];
    expect(dimFilled('role', ui.role)).toBe(true);
    ui.gender.open = true;
    expect(dimFilled('gender', ui.gender)).toBe(false);
    ui.charname.open = true;
    ui.charname.value = ' ';
    expect(dimFilled('charname', ui.charname)).toBe(false);
    ui.charname.value = 'x';
    expect(dimFilled('charname', ui.charname)).toBe(true);
    expect(hasActive(ui)).toBe(true);
  });

  it('标签多组共享 mode，排除集生成 tagExclude', () => {
    const ui = emptyQueryUi();
    ui.tags.open = true;
    ui.tags.mode = 'all';
    ui.tags.groups = [['百合', '白毛'], ['泣きゲー']];
    ui.tags.exclude = ['NTR'];
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'tag_include', mode: 'all', tags: ['百合', '白毛'] },
      { kind: 'tag_include', mode: 'all', tags: ['泣きゲー'] },
      { kind: 'tag_exclude', tags: ['NTR'] },
    ]);
  });

  it('空组不生成行，但排除集保留', () => {
    const ui = emptyQueryUi();
    ui.tags.open = true;
    ui.tags.groups = [[], ['百合']];
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'tag_include', mode: 'any', tags: ['百合'] },
    ]);
  });

  it('status 只收选中值', () => {
    const ui = emptyQueryUi();
    ui.status.open = true;
    ui.status.values = ['playing', 'finished'];
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'status', values: ['playing', 'finished'] },
    ]);
  });

  it('rating 按 10 分制输入换算为 0–100 存储，空串/非法为 null', () => {
    const ui = emptyQueryUi();
    ui.rating.open = true;
    ui.rating.min = '7.5';
    ui.rating.max = 'abc';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'rating', source: 'vndb', min: 75, max: null },
    ]);
  });

  it('rating user 源同样换算', () => {
    const ui = emptyQueryUi();
    ui.rating.open = true;
    ui.rating.source = 'user';
    ui.rating.max = '6';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'rating', source: 'user', min: null, max: 60 },
    ]);
  });

  it('released 非法日期剔除、合法保留', () => {
    const ui = emptyQueryUi();
    ui.released.open = true;
    ui.released.from = '2016-04-28';
    ui.released.to = 'TBA';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'released', from: '2016-04-28', to: null },
    ]);
  });

  it('developer 去首尾空白', () => {
    const ui = emptyQueryUi();
    ui.developer.open = true;
    ui.developer.name = '  Key  ';
    expect(uiToSpec(ui).rows).toEqual([{ kind: 'developer', name: 'Key' }]);
  });

  it('length 收小时值并换算为分钟', () => {
    const ui = emptyQueryUi();
    ui.length.open = true;
    ui.length.source = 'user';
    ui.length.min = '2.5';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'length', source: 'user', min: 150, max: null },
    ]);
  });

  it('enabled=false 的维度不进 spec', () => {
    const ui = emptyQueryUi();
    ui.status.open = true;
    ui.status.values = ['playing'];
    ui.status.enabled = false;
    ui.rating.open = true;
    ui.rating.min = '7';
    expect(uiToSpec(ui).rows).toEqual([
      { kind: 'rating', source: 'vndb', min: 70, max: null },
    ]);
  });

  it('dimFilled 与 hasActive 反映生效状态', () => {
    const ui = emptyQueryUi();
    expect(hasActive(ui)).toBe(false);
    ui.rating.open = true;
    expect(dimFilled('rating', ui.rating)).toBe(false);
    // 卡片展开即算「有条件在身」，清空入口可用；enabled 不影响
    expect(hasActive(ui)).toBe(true);
    ui.rating.min = '7';
    expect(dimFilled('rating', ui.rating)).toBe(true);
    expect(hasActive(ui)).toBe(true);
    ui.rating.enabled = false;
    expect(hasActive(ui)).toBe(true);
  });
});
