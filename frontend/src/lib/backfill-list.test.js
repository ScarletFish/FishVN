import { expect, test } from 'vitest';
import { focusRows, mergeCatalog, pickOnClick, pickRow, prunePicks, visibleRows } from './backfill-list.js';

const ROWS = ['a', 'b', 'c', 'd', 'e'].map((id) => ({ game_id: id }));

const GAMES = [
  {
    id: 'a',
    name: '甲',
    vndb_id: 'v1',
    has_metadata: true,
    titles: [
      { lang: 'ja', title: '甲の題' },
      { lang: 'zh-Hans', title: '甲题', official: true },
    ],
    released: '2016-05-27',
    alias: null,
  },
  {
    id: 'b',
    name: '乙',
    vndb_id: null,
    has_metadata: false,
    titles: [],
    released: null,
    alias: null,
  },
  {
    id: 'c',
    name: '丙',
    vndb_id: 'v3',
    has_metadata: false,
    titles: [{ lang: 'ja', title: '丙の題' }],
    released: '2004-01-30',
    alias: '丙别名',
  },
];

function item(game_id, state, extra = {}) {
  return {
    game_id,
    name: '旧名',
    vndb_id: 'v-old',
    state,
    bind_source: 'existing',
    hit_title: null,
    hit_released: null,
    candidates: [],
    filled: [],
    characters: 0,
    error: null,
    ...extra,
  };
}

test('全量条目都在，顺序与库内列表一致', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(rows.map((r) => r.game_id)).toEqual(['a', 'b', 'c']);
  expect(rows.map((r) => r.name)).toEqual(['甲', '乙', '丙']);
  expect(rows.map((r) => r.vndb_id)).toEqual(['v1', null, 'v3']);
});

test('标题取中文官方，其次日文，年份取发行日的四位', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(rows[0].title).toBe('甲题');
  expect(rows[2].title).toBe('丙の題');
  expect(rows[1].title).toBe('');
  expect(rows[0].year).toBe('2016');
  expect(rows[2].year).toBe('2004');
});

test('库内没有标题时用别名兜底', () => {
  const rows = mergeCatalog([{ ...GAMES[2], titles: [] }], [], {});
  expect(rows[0].title).toBe('丙别名');
});

test('跑批带回的标题只在库内没有标题时兜底', () => {
  const withMeta = mergeCatalog(GAMES, [item('a', 'ok', { hit_title: '跑批标题' })], {});
  expect(withMeta[0].title).toBe('甲题');

  const bare = mergeCatalog([GAMES[1]], [item('b', 'ok', { hit_title: '跑批标题' })], {});
  expect(bare[0].title).toBe('跑批标题');
});

test('不在目标集的条目没有任务状态', () => {
  const rows = mergeCatalog(GAMES, [item('a', 'pending')], {});
  expect(rows[0].state).toBe('pending');
  expect(rows[1].state).toBeNull();
  expect(rows[1].filled).toEqual([]);
  expect(rows[1].candidates).toEqual([]);
  expect(rows[1].characters).toBe(0);
});

test('叠加层字段生效，但键与库内字段以目录为准', () => {
  const rows = mergeCatalog(GAMES, [item('a', 'ok', { filled: ['简介'], characters: 3 })], {});
  expect(rows[0].name).toBe('甲');
  expect(rows[0].vndb_id).toBe('v1');
  expect(rows[0].filled).toEqual(['简介']);
  expect(rows[0].characters).toBe(3);
});

test('局部修正覆盖任务状态，且不影响其它条目', () => {
  const rows = mergeCatalog(GAMES, [item('a', 'pending')], {
    b: { state: 'ok', hit_title: '新标题' },
  });
  expect(rows[0].state).toBe('pending');
  expect(rows[1].state).toBe('ok');
  expect(rows[1].hit_title).toBe('新标题');
});

test('库里已删的叠加层不会出现在列表', () => {
  const rows = mergeCatalog(GAMES, [item('ghost', 'failed')], {
    ghost2: { state: 'ok' },
  });
  expect(rows).toHaveLength(3);
  expect(rows.some((r) => r.game_id.startsWith('ghost'))).toBe(false);
});

test('待处理只看有没有元数据行，与跑没跑过批无关', () => {
  const idle = mergeCatalog(GAMES, [], {});
  expect([...focusRows(idle)]).toEqual(['b', 'c']);

  const ran = mergeCatalog(
    GAMES,
    [item('a', 'failed'), item('b', 'unmatched'), item('c', 'ok')],
    {},
  );
  expect([...focusRows(ran)]).toEqual(['b', 'c']);
});

test('元数据行落地后待处理随之收窄', () => {
  const before = mergeCatalog(GAMES, [], {});
  expect([...focusRows(before)]).toEqual(['b', 'c']);

  const after = mergeCatalog(
    GAMES.map((game) => (game.id === 'b' ? { ...game, has_metadata: true } : game)),
    [item('b', 'ok', { bind_source: 'search' })],
    {},
  );
  expect([...focusRows(after)]).toEqual(['c']);
});

test('范围按元数据行收窄，与任务状态无关', () => {
  const rows = mergeCatalog(GAMES, [item('a', 'failed'), item('c', 'failed')], {});
  const scope = focusRows(rows);
  expect(visibleRows(rows, scope, false).map((r) => r.game_id)).toEqual(['b', 'c']);
  expect(visibleRows(rows, null, false).map((r) => r.game_id)).toEqual(['a', 'b', 'c']);
});

test('仅看无 ID 只看库里有没有绑定，与范围独立', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(visibleRows(rows, null, true).map((r) => r.game_id)).toEqual(['b']);
  expect(visibleRows(rows, focusRows(rows), true).map((r) => r.game_id)).toEqual(['b']);
});

test('按 id 取选中行，空 id 返回 null', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(pickRow(rows, 'b').name).toBe('乙');
  expect(pickRow(rows, 'nope')).toBeNull();
  expect(pickRow(rows, null)).toBeNull();
});

test('Shift 点击选中锚点到点击处之间的全部行，往下往上都一样', () => {
  const down = pickOnClick(
    { picked: ['b'], anchorId: 'b', selected: 'b' },
    ROWS,
    'd',
    { shift: true },
  );
  expect(down.picked).toEqual(['b', 'c', 'd']);

  const up = pickOnClick(
    { picked: ['d'], anchorId: 'd', selected: 'd' },
    ROWS,
    'b',
    { shift: true },
  );
  expect(up.picked).toEqual(['b', 'c', 'd']);
});

test('Shift 区间只落在当前可见行上，被筛掉的行不会被顺带选中', () => {
  const visible = ROWS.filter((row) => row.game_id !== 'c');
  const next = pickOnClick(
    { picked: ['a'], anchorId: 'a', selected: 'a' },
    visible,
    'd',
    { shift: true },
  );
  expect(next.picked).toEqual(['a', 'b', 'd']);
});

test('锚点被筛掉后 Shift 点击只选这一行，并把锚点挪到它', () => {
  const visible = ROWS.filter((row) => row.game_id !== 'a');
  const next = pickOnClick(
    { picked: ['a'], anchorId: 'a', selected: 'a' },
    visible,
    'd',
    { shift: true },
  );
  expect(next.picked).toEqual(['d']);
  expect(next.anchorId).toBe('d');
  expect(next.selected).toBe('d');
});

test('Ctrl 点击逐个增减，取消的正是面板那一行时面板退到集合末尾', () => {
  const add = pickOnClick(
    { picked: ['a', 'b'], anchorId: 'b', selected: 'b' },
    ROWS,
    'd',
    { toggle: true },
  );
  expect(add.picked).toEqual(['a', 'b', 'd']);
  expect(add.selected).toBe('d');

  const drop = pickOnClick(
    { picked: ['a', 'b', 'd'], anchorId: 'd', selected: 'd' },
    ROWS,
    'd',
    { toggle: true },
  );
  expect(drop.picked).toEqual(['a', 'b']);
  expect(drop.selected).toBe('b');
});

test('Ctrl 取消最后一行时面板关闭', () => {
  const next = pickOnClick(
    { picked: ['a'], anchorId: 'a', selected: 'a' },
    ROWS,
    'a',
    { toggle: true },
  );
  expect(next.picked).toEqual([]);
  expect(next.selected).toBeNull();
});

test('Ctrl+Shift 把区间并进已有选择，不清掉之前选的', () => {
  const next = pickOnClick(
    { picked: ['a'], anchorId: 'c', selected: 'c' },
    ROWS,
    'e',
    { shift: true, toggle: true },
  );
  expect(next.picked).toEqual(['a', 'c', 'd', 'e']);
});

test('多选状态下单击收敛为单选；只有唯一选中项再点一次才清空', () => {
  const collapse = pickOnClick(
    { picked: ['a', 'b', 'c'], anchorId: 'a', selected: 'c' },
    ROWS,
    'b',
    {},
  );
  expect(collapse.picked).toEqual(['b']);
  expect(collapse.anchorId).toBe('b');
  expect(collapse.selected).toBe('b');

  const clear = pickOnClick({ picked: ['b'], anchorId: 'b', selected: 'b' }, ROWS, 'b', {});
  expect(clear.picked).toEqual([]);
  expect(clear.selected).toBeNull();
});

test('不可见的行被剔出选中集，右栏与锚点一起断开', () => {
  const alive = new Set(['b', 'd']);
  const next = prunePicks(
    { picked: ['a', 'b', 'c'], anchorId: 'a', selected: 'c' },
    alive,
  );
  expect(next.picked).toEqual(['b']);
  expect(next.anchorId).toBeNull();
  expect(next.selected).toBeNull();
});

test('整份选中集都还可见时原样保留', () => {
  const alive = new Set(['a', 'b']);
  expect(prunePicks({ picked: ['a', 'b'], anchorId: 'b', selected: 'a' }, alive)).toEqual({
    picked: ['a', 'b'],
    anchorId: 'b',
    selected: 'a',
  });
  expect(prunePicks({}, alive)).toEqual({ picked: [], anchorId: null, selected: null });
});
