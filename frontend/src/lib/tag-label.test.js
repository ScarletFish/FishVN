import { afterEach, expect, test, vi } from 'vitest';
import { loadTagDict, tagKey, tagLabel, tagKeyLabel, traitLabel } from './tag-label.svelte.js';
import { api } from './api.js';

vi.mock('./env.js', () => ({ DEV_HTTP: false, DEV_HTTP_BASE: '/dev-http' }));
vi.mock('./dev-backend.js', () => ({ ensureBackend: async () => {} }));

afterEach(() => {
  vi.restoreAllMocks();
});

test('词典加载后按 id 命中中文，未命中与本地标签回退原名', async () => {
  expect(tagLabel({ id: 'g2', name: 'Fantasy' })).toBe('Fantasy');

  vi.spyOn(api, 'tagDictionary').mockResolvedValue({
    tags: {
      g2: { name: '奇幻', category: 'cont', vn_count: 1 },
      g96: { name: '恋爱', category: 'cont', vn_count: 2 },
    },
    traits: {
      i4: { name: '黑色', group: 'Hair' },
      i6: { name: '棕色', group: 'Hair' },
    },
  });
  await loadTagDict();

  expect(tagLabel({ id: 'g2', name: 'Fantasy' })).toBe('奇幻');
  expect(tagLabel({ id: 'g96', name: 'Romance' })).toBe('恋爱');
  expect(tagLabel({ id: 'g117', name: 'BDSM' })).toBe('BDSM');
  expect(tagLabel({ id: 'u1', name: '自造' })).toBe('自造');
  expect(traitLabel({ id: 'i4', name: 'Black' })).toBe('黑色');
  expect(traitLabel({ id: 'u1', name: '自造特征' })).toBe('自造特征');
  expect(traitLabel({ id: 'g2', name: 'Hair Color' })).toBe('Hair Color');
});

test('键就是标签 id', () => {
  expect(tagKey({ id: 'g2', name: 'Fantasy' })).toBe('g2');
  expect(tagKey({ id: 'u7', name: '自造' })).toBe('u7');
  expect(tagKey(null)).toBe('');
});

test('胶囊按 id 反查显示名', async () => {
  vi.spyOn(api, 'tagDictionary').mockResolvedValue({
    tags: { g96: { name: '恋爱', category: 'cont', vn_count: 2 } },
    traits: {},
  });
  await loadTagDict();

  expect(tagKeyLabel('g96')).toBe('恋爱');
  expect(tagKeyLabel('u9')).toBe('u9');
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
