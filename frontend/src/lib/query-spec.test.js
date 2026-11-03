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
    expect(spec).toEqual({ subject: 'game', rows: [] });
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
