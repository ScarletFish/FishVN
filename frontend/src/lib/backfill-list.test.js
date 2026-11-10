import { expect, test } from 'vitest';
import { countsOf, focusRows, mergeCatalog, pickRow, visibleRows } from './backfill-list.js';

const GAMES = [
  { id: 'a', name: '甲', vndb_id: 'v1' },
  { id: 'b', name: '乙', vndb_id: null },
  { id: 'c', name: '丙', vndb_id: 'v3' },
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

test('不在目标集的条目没有任务徽章', () => {
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

test('待处理是资料不全与需核对绑定的并集，与最近一轮跑过哪些无关', () => {
  const rows = mergeCatalog(
    GAMES,
    [
      item('a', 'ok', { bind_source: 'existing' }),
      item('b', 'ok', { bind_source: 'search' }),
      item('c', 'failed'),
    ],
    {},
  );
  expect([...focusRows(rows, ['c'])]).toEqual(['b', 'c']);
  expect([...focusRows(rows, [])]).toEqual(['b']);
});

test('重试只覆盖几条时，待处理清单不缩水', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'failed'), item('c', 'failed')],
    {},
  );
  const scope = focusRows(rows, ['a', 'c']);
  expect(visibleRows(rows, 'failed', scope, false).map((r) => r.game_id)).toEqual(['a', 'c']);
});

test('范围内按状态筛选，不跨出范围', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'ok'), item('b', 'unmatched'), item('c', 'failed')],
    {},
  );
  const scope = new Set(['a', 'c']);
  expect(visibleRows(rows, 'all', scope, false).map((r) => r.game_id)).toEqual(['a', 'c']);
  expect(visibleRows(rows, 'ok', scope, false).map((r) => r.game_id)).toEqual(['a']);
  expect(visibleRows(rows, 'failed', scope, false).map((r) => r.game_id)).toEqual(['c']);
  expect(visibleRows(rows, 'unmatched', scope, false)).toEqual([]);
});

test('范围为 null 时不限范围，状态与仅看无 ID 仍可叠加', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'ok'), item('b', 'unmatched'), item('c', 'failed')],
    {},
  );
  expect(visibleRows(rows, 'all', null, false).map((r) => r.game_id)).toEqual(['a', 'b', 'c']);
  expect(visibleRows(rows, 'all', null, true).map((r) => r.game_id)).toEqual(['b']);
  expect(visibleRows(rows, 'failed', null, false).map((r) => r.game_id)).toEqual(['c']);
});

test('计数随范围收窄，只数范围内的条目', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'ok'), item('b', 'unmatched'), item('c', 'failed')],
    {},
  );
  expect(countsOf(rows, null)).toEqual({
    all: 3,
    ok: 1,
    review: 0,
    unmatched: 1,
    failed: 1,
  });
  expect(countsOf(rows, new Set(['a', 'c']))).toEqual({
    all: 2,
    ok: 1,
    review: 0,
    unmatched: 0,
    failed: 1,
  });
});

test('需复核只收搜索绑定且成功的条目', () => {
  const rows = mergeCatalog(
    GAMES,
    [
      item('a', 'ok', { bind_source: 'existing' }),
      item('b', 'ok', { bind_source: 'search', hit_count: 3 }),
      item('c', 'failed', { bind_source: 'search' }),
    ],
    {},
  );
  expect(visibleRows(rows, 'review', null, false).map((r) => r.game_id)).toEqual(['b']);
  expect(countsOf(rows, null).review).toBe(1);
});

test('需复核不受仅看无 ID 影响', () => {
  const rows = mergeCatalog(
    GAMES,
    [
      item('b', 'ok', { bind_source: 'search' }),
      item('c', 'ok', { bind_source: 'search' }),
    ],
    {},
  );
  expect(visibleRows(rows, 'review', null, false).map((r) => r.game_id)).toEqual(['b', 'c']);
  expect(visibleRows(rows, 'review', null, true).map((r) => r.game_id)).toEqual(['b', 'c']);
});

test('人工改绑后自动退出需复核，待处理集合随之收窄', () => {
  const before = mergeCatalog(GAMES, [item('b', 'ok', { bind_source: 'search' })], {});
  const after = mergeCatalog(GAMES, [item('b', 'ok', { bind_source: 'search' })], {
    b: { bind_source: 'existing', hit_title: '人工选中的作品' },
  });
  expect([...focusRows(before, [])]).toEqual(['b']);
  expect([...focusRows(after, [])]).toEqual([]);
  expect(visibleRows(after, 'review', null, false)).toEqual([]);
});

test('按 id 取选中行，空 id 返回 null', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(pickRow(rows, 'b').name).toBe('乙');
  expect(pickRow(rows, 'nope')).toBeNull();
  expect(pickRow(rows, null)).toBeNull();
});
