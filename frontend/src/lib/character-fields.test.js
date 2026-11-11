import { expect, test } from 'vitest';
import {
  CHARACTER_DRAFT_META,
  CHARACTER_MERGE_FIELDS,
  CHARACTER_MERGE_KEYS,
  CHARACTER_NUMERIC_FIELDS,
} from './generated/character-fields.js';

test('草稿元信息只含 source，合并键互不重复且不含主键', () => {
  expect(CHARACTER_DRAFT_META).toEqual(['source']);
  expect(new Set(CHARACTER_MERGE_KEYS).size).toBe(CHARACTER_MERGE_KEYS.length);
  for (const key of ['game_id', 'id', 'sort_order']) {
    expect(CHARACTER_MERGE_KEYS).not.toContain(key);
  }
});

test('合并字段都带判定类别，数值上限只落在六个数测字段', () => {
  const rules = ['text', 'number', 'list', 'listTrait', 'listVa'];
  for (const field of CHARACTER_MERGE_FIELDS) {
    expect(rules).toContain(field.same);
  }
  expect(Object.keys(CHARACTER_NUMERIC_FIELDS).sort()).toEqual(
    ['age', 'height', 'weight', 'bust', 'waist', 'hips'].sort(),
  );
  expect(CHARACTER_NUMERIC_FIELDS.age).toEqual({ maxIntDigits: 2, max: 99 });
  expect(CHARACTER_NUMERIC_FIELDS.height).toEqual({ maxIntDigits: 3, max: 300 });
});
