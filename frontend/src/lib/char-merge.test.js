import { expect, test } from 'vitest';
import { applyCharacters, charKey, planCharacters } from './char-merge.js';

function ch(over = {}) {
  return {
    id: 'local-a',
    name: '',
    original: null,
    aliases: null,
    description: null,
    gender: null,
    age: null,
    birthday: null,
    blood_type: null,
    height: null,
    weight: null,
    bust: null,
    waist: null,
    hips: null,
    cup: null,
    role: null,
    image_id: null,
    image_path: null,
    sort_order: 0,
    traits: null,
    va: null,
    ...over,
  };
}

const saber = (over = {}) =>
  ch({ id: 'c33', name: 'Saber', original: 'セイバー', role: 'main', ...over });

const names = (list) => list.map((c) => c.original || c.name);

test('配对键优先取原名', () => {
  expect(charKey(ch({ name: 'Saber', original: 'セイバー' }))).toBe('セイバー');
  expect(charKey(ch({ name: 'Saber', original: '' }))).toBe('Saber');
  expect(charKey(ch({ name: 'Saber', original: null }))).toBe('Saber');
  expect(charKey(ch())).toBe('');
});

test('配对键去掉首尾空白', () => {
  expect(charKey(ch({ original: '  セイバー  ' }))).toBe('セイバー');
  expect(charKey(ch({ name: ' Saber ' }))).toBe('Saber');
});

test('incoming 独有的进新增', () => {
  const plan = planCharacters([], [saber()]);
  expect(plan.added).toHaveLength(1);
  expect(plan.conflicts).toEqual([]);
  expect(plan.kept).toEqual([]);
});

test('本地独有的进保留', () => {
  const local = ch({ id: 'local-x', name: '自建角色' });
  const plan = planCharacters([local], []);
  expect(plan.added).toEqual([]);
  expect(plan.conflicts).toEqual([]);
  expect(plan.kept).toEqual([local]);
});

test('同名且有差异的进冲突', () => {
  const local = ch({ id: 'local-x', name: 'Saber', original: 'セイバー', image_path: 'mine.jpg' });
  const plan = planCharacters([local], [saber({ image_path: 'vndb.jpg' })]);
  expect(plan.added).toEqual([]);
  expect(plan.kept).toEqual([]);
  expect(plan.conflicts).toHaveLength(1);
  expect(plan.conflicts[0].key).toBe('セイバー');
  expect(plan.conflicts[0].current.image_path).toBe('mine.jpg');
  expect(plan.conflicts[0].incoming.image_path).toBe('vndb.jpg');
});

test('重名且完全相同不算冲突', () => {
  expect(planCharacters([saber()], [saber()]).conflicts).toEqual([]);
});

test('重名但性别数组顺序不同不算冲突', () => {
  const a = saber({ gender: ['f', 'm'] });
  const b = saber({ gender: ['m', 'f'] });
  expect(planCharacters([a], [b]).conflicts).toEqual([]);
});

test('重名但 sort_order 不同不算冲突', () => {
  expect(planCharacters([saber({ sort_order: 7 })], [saber({ sort_order: 3 })]).conflicts).toEqual([]);
});

test('重名但 image_path 不同算冲突', () => {
  const a = saber({ image_path: 'mine.jpg' });
  const b = saber({ image_path: 'vndb.jpg' });
  expect(planCharacters([a], [b]).conflicts).toHaveLength(1);
});

test('空串与 null 归一', () => {
  const a = saber({ description: '', birthday: '', cup: null });
  const b = saber({ description: null, birthday: undefined, cup: '' });
  expect(planCharacters([a], [b]).conflicts).toEqual([]);
});

test.each([
  ['年龄 80 与 80.0 判等', { age: '80' }, { age: 80.0 }, 0],
  ['体型数字与字符串判等', { height: '154', bust: 73 }, { height: 154.0, bust: '73.0' }, 0],
  ['空值与 0 不判等', { age: null }, { age: 0 }, 1],
])('%s', (desc, localOver, incomingOver, conflictCount) => {
  const conflicts = planCharacters([saber(localOver)], [saber(incomingOver)]).conflicts;
  expect(conflicts).toHaveLength(conflictCount);
});

test('别名与 traits 顺序不同不算冲突', () => {
  const a = saber({
    aliases: ['B', 'A'],
    traits: [{ id: 'i1', name: 'Long Hair' }, { id: 'i2', name: 'Brave' }],
  });
  const b = saber({
    aliases: ['A', 'B'],
    traits: [{ id: 'i2', name: 'Brave' }, { id: 'i1', name: 'Long Hair' }],
  });
  expect(planCharacters([a], [b]).conflicts).toEqual([]);
});

test('va 集合相同（顺序不同）不算冲突', () => {
  const a = saber({
    va: [
      { staff_id: 's1', original: null, note: '' },
      { staff_id: 's2', original: null, note: '少年版' },
    ],
  });
  const b = saber({
    va: [
      { staff_id: 's2', original: null, note: '少年版' },
      { staff_id: 's1', original: null, note: '' },
    ],
  });
  expect(planCharacters([a], [b]).conflicts).toEqual([]);
});

test('va 的备注变了算冲突', () => {
  const a = saber({ va: [{ staff_id: 's1', original: null, note: '' }] });
  const b = saber({ va: [{ staff_id: 's1', original: null, note: '后来补的' }] });
  expect(planCharacters([a], [b]).conflicts).toHaveLength(1);
});

test('只选中的冲突取 VNDB 的整角色', () => {
  const a = saber({ id: 'local-a', image_path: 'mine.jpg', height: 150 });
  const b = saber({ image_path: 'vndb.jpg', height: 154 });
  const next = applyCharacters([a], [b], new Set(['セイバー']));
  expect(next).toHaveLength(1);
  expect(next[0].image_path).toBe('vndb.jpg');
  expect(next[0].height).toBe(154);
});

test('未选中的冲突保留本地', () => {
  const a = saber({ id: 'local-a', image_path: 'mine.jpg' });
  const b = saber({ image_path: 'vndb.jpg' });
  const next = applyCharacters([a], [b], new Set());
  expect(next[0].image_path).toBe('mine.jpg');
  expect(next[0].id).toBe('local-a');
});

test('本地独有的角色即使 key 在 adopted 里也不被覆盖', () => {
  const local = ch({ id: 'local-x', name: '自建角色', role: 'side' });
  const next = applyCharacters([local], [], new Set(['自建角色']));
  expect(next).toHaveLength(1);
  expect(next[0].id).toBe('local-x');
});

test('新增的角色直接进结果', () => {
  const next = applyCharacters([], [saber()], new Set());
  expect(next).toHaveLength(1);
  expect(next[0].id).toBe('c33');
});

test('按 role 四档排序并重编号 sort_order', () => {
  const local = [
    ch({ id: 'l1', name: 'appears', role: 'appears' }),
    ch({ id: 'l2', name: 'side', role: 'side' }),
  ];
  const incoming = [
    ch({ id: 'v1', name: 'primary', role: 'primary' }),
    ch({ id: 'v2', name: 'main', role: 'main' }),
    ch({ id: 'v3', name: '空档' }),
  ];
  const next = applyCharacters(local, incoming, new Set());
  expect(names(next)).toEqual(['main', 'primary', 'side', 'appears', '空档']);
  expect(next.map((c) => c.sort_order)).toEqual([0, 1, 2, 3, 4]);
});

test('同档内保持各自来源的原有顺序', () => {
  const local = [
    ch({ id: 'l1', name: 'A', role: 'main' }),
    ch({ id: 'l2', name: 'B', role: 'main' }),
  ];
  const incoming = [
    ch({ id: 'v1', name: 'C', role: 'main' }),
    ch({ id: 'v2', name: 'D', role: 'main' }),
  ];
  expect(names(applyCharacters(local, incoming, new Set()))).toEqual(['A', 'B', 'C', 'D']);
});

test('未列入四档的 role 排到最后', () => {
  const next = applyCharacters(
    [ch({ id: 'l1', name: '怪角色', role: 'background' }), ch({ id: 'l2', name: '主角', role: 'main' })],
    [],
    new Set()
  );
  expect(names(next)).toEqual(['主角', '怪角色']);
});

test('不改动传入的角色对象', () => {
  const a = saber({ image_path: 'mine.jpg' });
  applyCharacters([a], [saber({ image_path: 'vndb.jpg' })], new Set(['セイバー']));
  expect(a.image_path).toBe('mine.jpg');
});