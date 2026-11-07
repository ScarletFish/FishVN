import { expect, test } from 'vitest';
import { countsOf, mergeCatalog, pickRow, visibleRows } from './backfill-list.js';

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

test('默认本次只显示冻结的目标集，全部显示全量', () => {
  const rows = mergeCatalog(GAMES, [item('a', 'pending'), item('c', 'ok')], {});
  const targetIds = ['a', 'c'];
  expect(visibleRows(rows, 'todo', targetIds, false).map((r) => r.game_id)).toEqual(['a', 'c']);
  expect(visibleRows(rows, 'all', targetIds, false).map((r) => r.game_id)).toEqual(['a', 'b', 'c']);
});

test('任务状态从待处理推进到成功，默认集合不变', () => {
  const targetIds = ['a', 'c'];
  const before = mergeCatalog(GAMES, [item('a', 'pending'), item('c', 'pending')], {});
  const after = mergeCatalog(GAMES, [item('a', 'ok'), item('c', 'unmatched')], {});
  expect(visibleRows(after, 'todo', targetIds, false).map((r) => r.game_id)).toEqual(
    visibleRows(before, 'todo', targetIds, false).map((r) => r.game_id),
  );
});

test('状态筛选与仅看无 ID 可叠加', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'ok'), item('b', 'unmatched'), item('c', 'failed')],
    {},
  );
  const all = ['a', 'b', 'c'];
  expect(visibleRows(rows, 'ok', all, false).map((r) => r.game_id)).toEqual(['a']);
  expect(visibleRows(rows, 'unmatched', all, false).map((r) => r.game_id)).toEqual(['b']);
  expect(visibleRows(rows, 'failed', all, false).map((r) => r.game_id)).toEqual(['c']);
  expect(visibleRows(rows, 'all', all, true).map((r) => r.game_id)).toEqual(['b']);
  expect(visibleRows(rows, 'todo', all, true).map((r) => r.game_id)).toEqual(['b']);
});

test('计数覆盖本次、全部与各结果状态', () => {
  const rows = mergeCatalog(
    GAMES,
    [item('a', 'ok'), item('b', 'unmatched'), item('c', 'failed')],
    {},
  );
  expect(countsOf(rows, ['a', 'c'])).toEqual({
    todo: 2,
    review: 0,
    all: 3,
    ok: 1,
    unmatched: 1,
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
  const all = ['a', 'b', 'c'];
  expect(visibleRows(rows, 'review', all, false).map((r) => r.game_id)).toEqual(['b']);
  expect(countsOf(rows, all).review).toBe(1);
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
  const all = ['a', 'b', 'c'];
  expect(visibleRows(rows, 'review', all, false).map((r) => r.game_id)).toEqual(['b', 'c']);
  expect(visibleRows(rows, 'review', all, true).map((r) => r.game_id)).toEqual(['b', 'c']);
});

test('人工改绑后自动退出需复核，列表成员不变', () => {
  const targetIds = ['a', 'b'];
  const before = mergeCatalog(GAMES, [item('b', 'ok', { bind_source: 'search' })], {});
  const after = mergeCatalog(GAMES, [item('b', 'ok', { bind_source: 'search' })], {
    b: { bind_source: 'existing', hit_title: '人工选中的作品' },
  });
  expect(visibleRows(before, 'review', targetIds, false).map((r) => r.game_id)).toEqual(['b']);
  expect(visibleRows(after, 'review', targetIds, false)).toEqual([]);
  expect(visibleRows(after, 'todo', targetIds, false).map((r) => r.game_id)).toEqual(
    visibleRows(before, 'todo', targetIds, false).map((r) => r.game_id),
  );
});

test('按 id 取选中行，空 id 返回 null', () => {
  const rows = mergeCatalog(GAMES, [], {});
  expect(pickRow(rows, 'b').name).toBe('乙');
  expect(pickRow(rows, 'nope')).toBeNull();
  expect(pickRow(rows, null)).toBeNull();
});
