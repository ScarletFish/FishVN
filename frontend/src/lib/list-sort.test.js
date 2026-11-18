import { describe, expect, test } from 'vitest';
import { gameComparator, groupBySeries } from './list-sort.svelte.js';

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


describe('groupBySeries', () => {
  const g = (name, series_name, sort_name, status = 'playing', extra = {}) => ({
    name, series_name, sort_name, status, ...extra,
  });

  test('系列成员聚到锚点位置，组内按排序名升序', () => {
    const items = [
      g('A', null, null),
      g('壳之少女', '之少女', '01'),
      g('B', null, null),
      g('虚之少女', '之少女', '02'),
      g('C', null, null),
      g('天之少女', '之少女', '03'),
    ];
    const out = groupBySeries(items);
    expect(out.map((x) => x.name)).toEqual(['A', '壳之少女', '虚之少女', '天之少女', 'B', 'C']);
  });

  test('组内排序名空值排最后', () => {
    const items = [
      g('壳', 'S', '01'),
      g('无序号', 'S', null),
      g('虚', 'S', '02'),
    ];
    expect(groupBySeries(items).map((x) => x.name)).toEqual(['壳', '虚', '无序号']);
  });

  test('同系列两部及以上才聚组', () => {
    const items = [
      g('A', null, null),
      g('壳', 'S', '01', 'playing'),
      g('B', null, null),
      g('虚', 'S', '02', 'playing'),
    ];
    expect(groupBySeries(items).map((x) => x.name)).toEqual(['A', '壳', '虚', 'B']);
  });

  test('组内只有一部时不成组，留在原位', () => {
    const items = [
      g('A', null, null),
      g('壳', 'S', '01', 'playing'),
      g('B', null, null),
      g('别处的虚', 'S', '02', 'finished'),
    ];
    const inPlaying = items.filter((x) => x.status === 'playing');
    expect(groupBySeries(inPlaying).map((x) => x.name)).toEqual(['A', '壳', 'B']);
  });

  test('当前组内有两部就聚，不需要全部成员都在', () => {
    const items = [
      g('A', null, null),
      g('壳', 'S', '01', 'playing'),
      g('B', null, null),
      g('虚', 'S', '02', 'playing'),
      g('别处的天', 'S', '03', 'finished'),
    ];
    const inPlaying = items.filter((x) => x.status === 'playing');
    expect(groupBySeries(inPlaying).map((x) => x.name)).toEqual(['A', '壳', '虚', 'B']);
  });
});
