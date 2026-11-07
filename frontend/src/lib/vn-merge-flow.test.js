import { beforeEach, expect, test, vi } from 'vitest';

const m = vi.hoisted(() => ({
  listGames: vi.fn(),
  fetchVnMetadata: vi.fn(),
  cancelTask: vi.fn(),
  subscribe: vi.fn(),
  off: vi.fn(),
  toast: vi.fn(),
  conflicts: [{ key: 'name' }],
  charConflicts: [{ key: 'c1' }],
  handler: null,
}));

vi.mock('./api.js', () => ({
  api: {
    listGames: m.listGames,
    fetchVnMetadata: m.fetchVnMetadata,
    cancelTask: m.cancelTask,
  },
}));
vi.mock('./events.js', () => ({ subscribe: m.subscribe }));
vi.mock('./toast.svelte.js', () => ({ toast: m.toast }));
vi.mock('./i18n.js', () => ({ t: (key) => key }));
vi.mock('./vn-merge.js', () => ({
  conflictRows: () => m.conflicts,
  applyIncoming: (current, incoming, picked) => ({ ...current, picked: [...picked] }),
}));
vi.mock('./char-merge.js', () => ({
  planCharacters: () => ({ conflicts: m.charConflicts }),
  applyCharacters: (current, incoming) => incoming,
}));

import { createVnMergeFlow } from './vn-merge-flow.svelte.js';

function load(ownId = 'g1') {
  return vi.fn().mockResolvedValue({ ownId, current: { name: '本地' }, characters: [] });
}

beforeEach(() => {
  vi.clearAllMocks();
  m.conflicts = [{ key: 'name' }];
  m.charConflicts = [{ key: 'c1' }];
  m.handler = null;
  m.listGames.mockResolvedValue([]);
  m.fetchVnMetadata.mockResolvedValue({ name: '远端', characters: [{ id: 'c1' }] });
  m.cancelTask.mockResolvedValue(undefined);
  m.subscribe.mockImplementation(async (_channel, fn) => {
    m.handler = fn;
    return m.off;
  });
});

test('选中候选后算出两组冲突，订阅与拉取状态都被清理', async () => {
  const flow = createVnMergeFlow();
  const ok = await flow.pick('v1', load());

  expect(ok).toBe(true);
  expect(m.subscribe).toHaveBeenCalledWith('vndb-sync', expect.any(Function));
  expect(m.fetchVnMetadata).toHaveBeenCalledWith('v1');
  expect(flow.selectedId).toBe('v1');
  expect(flow.rows).toEqual([{ key: 'name' }]);
  expect(flow.charRows).toEqual([{ key: 'c1' }]);
  expect(flow.pulling).toBe(false);
  expect(flow.progress).toBeNull();
  expect(m.off).toHaveBeenCalledOnce();
});

test('accept 只计算不写入，并保留合并前的字段供宿主比较', async () => {
  const flow = createVnMergeFlow();
  await flow.pick('v1', load());

  const result = flow.accept(new Set(['name']), new Set());
  expect(result.id).toBe('v1');
  expect(result.previous).toEqual({ name: '本地' });
  expect(result.fields.picked).toEqual(['name']);
  expect(result.characters).toEqual([{ id: 'c1' }]);
  expect(flow.accept(new Set(), new Set())).toBeNull();
});

test('vndb_id 已被别的作品占用时直接拒绝，不发起拉取', async () => {
  m.listGames.mockResolvedValue([{ id: 'g2', name: '别的作品', vndb_id: 'v1' }]);
  const flow = createVnMergeFlow();

  expect(await flow.pick('v1', load())).toBeNull();
  expect(m.fetchVnMetadata).not.toHaveBeenCalled();
  expect(m.toast).toHaveBeenCalledWith('gameForm.error.vndbTaken', 'error');
  expect(flow.pulling).toBe(false);
});

test('拉取抛错时提示并复位，不留下半开的订阅', async () => {
  m.fetchVnMetadata.mockRejectedValue(new Error('网络断了'));
  const flow = createVnMergeFlow();

  expect(await flow.pick('v1', load())).toBeNull();
  expect(m.toast).toHaveBeenCalledWith('网络断了', 'error');
  expect(m.off).toHaveBeenCalledOnce();
  expect(flow.pulling).toBe(false);
});

test('无差异时不打开取舍面板', async () => {
  m.conflicts = [];
  m.charConflicts = [];
  const flow = createVnMergeFlow();

  expect(await flow.pick('v1', load())).toBeNull();
  expect(m.toast).toHaveBeenCalledWith('backfillPage.fixNone');
  expect(flow.rows).toEqual([]);
});

test('run 的事件过滤与错误投影，且同一时刻只允许一次拉取', async () => {
  const flow = createVnMergeFlow();
  const result = await flow.run(
    'vndb-backfill',
    async () => {
      m.handler({ game_id: 'g2', phase: 'meta' });
      expect(flow.progress).toBeNull();
      m.handler({ game_id: 'g1', phase: 'meta', done: 1 });
      expect(flow.progress).toMatchObject({ phase: 'meta' });
      m.handler({ error: '限流' });
      expect(flow.notice).toBe('限流');
      expect(await flow.run('vndb-sync', async () => 'second')).toBeNull();
      return 'first';
    },
    (ev) => !ev.game_id || ev.game_id === 'g1',
  );

  expect(result).toBe('first');
  expect(m.off).toHaveBeenCalledOnce();
  expect(flow.pulling).toBe(false);
});
