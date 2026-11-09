import { expect, test } from 'vitest';
import {
  CHARACTER_DRAFT_KEYS,
  CHARACTER_DRAFT_META,
  CHARACTER_FIELDS,
  CHARACTER_MERGE_FIELDS,
  CHARACTER_MERGE_KEYS,
  CHARACTER_NUMERIC_FIELDS,
  CHARACTER_PAYLOAD_KEYS,
} from './generated/character-fields.js';

test('角色规格覆盖全部字段且类别计数稳定', () => {
  expect(CHARACTER_FIELDS).toHaveLength(22);
  expect(CHARACTER_FIELDS.filter((f) => f.identity)).toHaveLength(3);
  expect(CHARACTER_FIELDS.filter((f) => f.joined)).toHaveLength(2);
  expect(CHARACTER_FIELDS.filter((f) => f.inDdl)).toHaveLength(20);
  expect(CHARACTER_FIELDS.filter((f) => f.inColumns)).toHaveLength(19);
  expect(CHARACTER_FIELDS.filter((f) => f.inStruct)).toHaveLength(21);
  expect(CHARACTER_FIELDS.filter((f) => f.inDraft)).toHaveLength(19);
  expect(CHARACTER_FIELDS.filter((f) => f.inMerge)).toHaveLength(19);
});

test('三套键集合由同一规格派生且互不重复', () => {
  const draftFields = CHARACTER_FIELDS.filter((f) => f.inDraft);
  expect(CHARACTER_PAYLOAD_KEYS).toEqual(draftFields.map((f) => f.key));
  expect(CHARACTER_DRAFT_KEYS).toEqual([
    ...draftFields.map((f) => f.draft),
    ...CHARACTER_DRAFT_META,
  ]);
  expect(CHARACTER_DRAFT_META).toEqual(['source']);
  expect(CHARACTER_MERGE_KEYS).toEqual(
    CHARACTER_FIELDS.filter((f) => f.inMerge).map((f) => f.key),
  );
  expect(new Set(CHARACTER_MERGE_KEYS).size).toBe(CHARACTER_MERGE_KEYS.length);
  for (const key of ['game_id', 'id', 'sort_order']) {
    expect(CHARACTER_MERGE_KEYS).not.toContain(key);
  }
});

test('合并字段都带判定类别，数值上限只落在六个数测字段', () => {
  const rules = ['text', 'number', 'list', 'listTrait', 'listVa'];
  expect(CHARACTER_MERGE_FIELDS).toHaveLength(19);
  for (const field of CHARACTER_MERGE_FIELDS) {
    expect(rules).toContain(field.same);
  }
  expect(Object.keys(CHARACTER_NUMERIC_FIELDS).sort()).toEqual(
    ['age', 'height', 'weight', 'bust', 'waist', 'hips'].sort(),
  );
  expect(CHARACTER_NUMERIC_FIELDS.age).toEqual({ maxIntDigits: 2, max: 99 });
  expect(CHARACTER_NUMERIC_FIELDS.height).toEqual({ maxIntDigits: 3, max: 300 });
});
