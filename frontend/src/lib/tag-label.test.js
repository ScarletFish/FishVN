import { afterEach, expect, test, vi } from 'vitest';
import { loadTagDict, tagLabel, traitLabel } from './tag-label.svelte.js';
import { api } from './api.js';

vi.mock('./env.js', () => ({ DEV_HTTP: false, DEV_HTTP_BASE: '/dev-http' }));
vi.mock('./dev-backend.js', () => ({ ensureBackend: async () => {} }));

afterEach(() => {
  vi.restoreAllMocks();
});

test('词典加载后按 id 命中中文，未命中与无 id 回退原名', async () => {
  // 加载前：空表，全部回退原名
  expect(tagLabel({ id: 'g2', name: 'Fantasy' })).toBe('Fantasy');

  vi.spyOn(api, 'tagDictionary').mockResolvedValue({
    tags: { g2: '奇幻', g96: '恋爱' },
    traits: { i4: '黑色', i6: '棕色' },
  });
  await loadTagDict();

  expect(tagLabel({ id: 'g2', name: 'Fantasy' })).toBe('奇幻');
  expect(tagLabel({ id: 'g96', name: 'Romance' })).toBe('恋爱');
  expect(tagLabel({ id: 'g117', name: 'BDSM' })).toBe('BDSM');
  expect(tagLabel({ name: '手输标签' })).toBe('手输标签');
  expect(traitLabel({ id: 'i4', name: 'Black' })).toBe('黑色');
  expect(traitLabel({ id: 'i6', name: 'Brown' })).toBe('棕色');
  // 跨表不串位：trait 查不到 tag 表里的 g2
  expect(traitLabel({ id: 'g2', name: 'Hair Color' })).toBe('Hair Color');
});

test('词典加载失败时保持英文回退', async () => {
  vi.resetModules();
  const { api: freshApi } = await import('./api.js');
  const mod = await import('./tag-label.svelte.js');
  vi.spyOn(freshApi, 'tagDictionary').mockRejectedValueOnce(new Error('network'));

  await mod.loadTagDict();

  expect(mod.tagLabel({ id: 'g2', name: 'Fantasy' })).toBe('Fantasy');
  expect(mod.traitLabel({ id: 'i4', name: 'Black' })).toBe('Black');
});
