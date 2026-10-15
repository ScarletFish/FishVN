import { expect, test } from 'vitest';
import {
  blankCharacter,
  displayName,
  moveCharacters,
  namedOnly,
  toDraft,
  toPayload,
} from './characters.js';

/** VNDB 同步来的一行：带着本地表单不露出来的那些字段 */
function syncedRow() {
  return {
    id: 'c33',
    name: 'Saber',
    original: 'セイバー',
    gender: ['f', 'f'],
    age: 17,
    height: 154,
    weight: 42,
    role: 'main',
    image_path: 'data/covers/vndb/ch34.jpg',
    description: '描述',
    traits: [{ id: 'i1', name: 'Long Hair', group_name: 'Hair' }],
    bust: 73,
    waist: 53,
    hips: 76,
    blood_type: 'a',
    va: [{ staff_id: 's1', name: '川澄綾子', original: null, note: '' }],
  };
}

test('取回的行按表单口径铺开', () => {
  const draft = toDraft(syncedRow());

  expect(draft.name).toBe('Saber');
  expect(draft.age).toBe('17');
  // 性别在库里是每 release 一条的数组，表单只有一个下拉
  expect(draft.gender).toBe('f');
  expect(draft.va).toHaveLength(1);
  expect(draft.va[0].staffId).toBe('s1');
});

test('表单没露出的字段原样带回去', () => {
  const draft = toDraft(syncedRow());
  draft.name = '改成中文名';

  const payload = toPayload(draft);

  expect(payload.name).toBe('改成中文名');
  expect(payload.traits).toEqual(syncedRow().traits);
  expect(payload.bust).toBe(73);
  expect(payload.blood_type).toBe('a');
});

test('新建角色没有来源行也能出 payload', () => {
  const payload = toPayload(blankCharacter());

  expect(payload.id.startsWith('local-')).toBe(true);
  expect(payload.name).toBeNull();
  expect(payload.gender).toBeNull();
  expect(payload.va).toEqual([]);
});

test('留空的字段写回 null 而不是空串', () => {
  const draft = toDraft(syncedRow());
  draft.original = '   ';
  draft.age = '';
  draft.height = '不是数字';

  const payload = toPayload(draft);

  expect(payload.original).toBeNull();
  expect(payload.age).toBeNull();
  expect(payload.height).toBeNull();
});

test('已有的声优编号不被换掉，新加的自己生成', () => {
  const draft = toDraft(syncedRow());
  draft.va.push({ key: 'k', staffId: '', name: '新声优', note: '' });

  const [kept, added] = toPayload(draft).va;

  expect(kept.staff_id).toBe('s1');
  expect(added.staff_id.startsWith('local-')).toBe(true);
  expect(added.name).toBe('新声优');
  expect(added.note).toBe('');
});

test('顺序位移越界时原样返回', () => {
  const list = [{ id: 'a' }, { id: 'b' }];

  expect(moveCharacters(list, 0, 1).map((c) => c.id)).toEqual(['b', 'a']);
  expect(moveCharacters(list, 0, -1)).toBe(list);
  expect(moveCharacters(list, 1, 1)).toBe(list);
});

test('卡片名优先级与详情页一致', () => {
  expect(displayName({ original: '遠坂凛', name: 'Rin' })).toBe('遠坂凛');
  expect(displayName({ original: '  ', name: 'Rin' })).toBe('Rin');
  expect(displayName({ original: '', name: '' })).toBe('');
});

test('无名角色被滤掉，有名的按原顺序留下', () => {
  const blank = blankCharacter();
  const list = [
    blank,
    { ...blank, id: 'n', name: 'Saber' },
    { ...blank, id: 'o', original: 'セイバー' },
    { ...blank, id: 'w', name: '  ', original: '\t\n' },
    { ...blank, id: 'm', name: 'Mordred', original: 'モルドレッド' },
  ];

  expect(namedOnly(list).map((c) => c.id)).toEqual(['n', 'o', 'm']);
});
