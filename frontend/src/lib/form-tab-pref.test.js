import { expect, test } from 'vitest';
import { resolveFormTab } from './form-tab-pref.js';

test.each([
  ['合法 tab basic 原样返回', 'basic', 'basic'],
  ['合法 tab profile 原样返回', 'profile', 'profile'],
  ['合法 tab tags 原样返回', 'tags', 'tags'],
  ['合法 tab installations 原样返回', 'installations', 'installations'],
  ['合法 tab characters 原样返回', 'characters', 'characters'],
  ['null 回落到基本页', null, 'basic'],
  ['undefined 回落到基本页', undefined, 'basic'],
  ['空串回落到基本页', '', 'basic'],
  ['未知字符串 garbage 回落到基本页', 'garbage', 'basic'],
])('%s', (desc, input, expected) => {
  expect(resolveFormTab(input)).toBe(expected);
});
