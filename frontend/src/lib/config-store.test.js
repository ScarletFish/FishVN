// @vitest-environment jsdom
import { beforeEach, describe, expect, test, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  getConfig: vi.fn(),
  updateConfig: vi.fn(),
  toast: vi.fn(),
}));

vi.mock('./api.js', () => ({
  api: { getConfig: mocks.getConfig, updateConfig: mocks.updateConfig },
}));

vi.mock('./toast.svelte.js', () => ({ toast: mocks.toast }));

import { DEFAULT_CHARACTER_SCOPE } from './generated/character-roles.js';
import { getSpoilerLevel, loadConfig, readConfig, saveConfig } from './config-store.svelte.js';

const STORED = {
  time_metric: 'alive',
  spoiler_level: 'show_all',
  excluded_tags: ['g1'],
  dashboard_layout: [{ id: 'stats', enabled: true }],
  vndb_sync_rating: false,
  has_vndb_token: true,
};

beforeEach(async () => {
  mocks.getConfig.mockReset();
  mocks.updateConfig.mockReset();
  mocks.toast.mockReset();
  mocks.getConfig.mockResolvedValue(STORED);
  await loadConfig();
});

describe('loadConfig', () => {
  test('后端给到的合法值原样落到快照', () => {
    const config = readConfig();
    expect(getSpoilerLevel()).toBe('show_all');
    expect(config.time_metric).toBe('alive');
    expect(config.excluded_tags).toEqual(['g1']);
    expect(config.vndb_sync_rating).toBe(false);
    expect(config.has_vndb_token).toBe(true);
  });

  test('重载时就地更新，早先拿到的快照引用仍然可读', async () => {
    const captured = readConfig();

    mocks.getConfig.mockResolvedValue({ ...STORED, spoiler_level: 'hide_major' });
    await loadConfig();

    expect(readConfig()).toBe(captured);
    expect(captured.spoiler_level).toBe('hide_major');
  });

  test('越界字段回落到默认，不把脏值带进界面', async () => {
    mocks.getConfig.mockResolvedValue({
      time_metric: 'weird',
      spoiler_level: 'nope',
      excluded_tags: 'g1',
      character_scope: 'nope',
      vndb_sync_mode: 'nope',
      backup_retention_days: undefined,
    });
    await loadConfig();

    const config = readConfig();
    expect(config.time_metric).toBe('foreground');
    expect(config.spoiler_level).toBe('hide_all');
    expect(config.excluded_tags).toEqual([]);
    expect(config.character_scope).toBe(DEFAULT_CHARACTER_SCOPE);
    expect(config.vndb_sync_mode).toBe('download');
    expect(config.backup_retention_days).toBe(null);
  });
});

describe('saveConfig', () => {
  test('先乐观写，再把补丁交给后端', async () => {
    mocks.updateConfig.mockResolvedValue(undefined);

    const pending = saveConfig({ hide_r18_tags: true });
    expect(readConfig().hide_r18_tags).toBe(true);

    const ok = await pending;
    expect(ok).toBe(true);
    expect(mocks.toast).not.toHaveBeenCalled();
    expect(mocks.updateConfig).toHaveBeenCalledWith({ hide_r18_tags: true });
  });

  test('写失败时回滚并提示', async () => {
    mocks.updateConfig.mockRejectedValue(new Error('boom'));

    await expect(saveConfig({ hide_r18_tags: true })).resolves.toBe(false);

    expect(readConfig().hide_r18_tags).toBe(false);
    expect(mocks.toast).toHaveBeenCalledWith('boom', 'error');
  });

  test('显式置空能穿过补丁落到快照', async () => {
    mocks.updateConfig.mockResolvedValue(undefined);

    await saveConfig({ backup_retention_days: null });

    expect(mocks.updateConfig).toHaveBeenCalledWith({ backup_retention_days: null });
  });
});
