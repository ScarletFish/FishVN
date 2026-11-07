import { describe, expect, test } from 'vitest';
import { gameComparator } from './list-sort.svelte.js';

const sorted = (by, games) => games.slice().sort(gameComparator(by));

describe('gameComparator', () => {
  test('name 按 zh-CN localeCompare，空名视为空串排最前', () => {
    const games = [{ name: 'ATRI' }, { name: '' }, { name: 'CLANNAD' }, { name: null }];
    expect(sorted('name', games).map((g) => g.name)).toEqual(['', null, 'ATRI', 'CLANNAD']);
  });

  test('vndb 降序，无评分排后且之间按原序稳定', () => {
    const games = [
      { name: 'a', vndb_rating: 7.5 },
      { name: 'b', vndb_rating: null },
      { name: 'c', vndb_rating: 8.9 },
      { name: 'd', vndb_rating: null },
    ];
    expect(sorted('vndb', games).map((g) => g.name)).toEqual(['c', 'a', 'b', 'd']);
  });

  test('user 降序，无评分排后', () => {
    const games = [
      { name: 'a', user_rating: 8 },
      { name: 'b', user_rating: null },
      { name: 'c', user_rating: 9 },
    ];
    expect(sorted('user', games).map((g) => g.name)).toEqual(['c', 'a', 'b']);
  });

  test('added 按创建时间新的在前', () => {
    const games = [
      { name: 'a', created_at: 100 },
      { name: 'b', created_at: 300 },
      { name: 'c', created_at: 200 },
    ];
    expect(sorted('added', games).map((g) => g.name)).toEqual(['b', 'c', 'a']);
  });
});
