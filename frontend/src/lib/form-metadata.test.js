import { expect, test } from 'vitest';
import { buildMetadataPatch, toInteger, toNumber } from './form-metadata.js';

/** 空白表单：字段与编辑态的初值同构 */
function blank() {
  return {
    alias: '',
    released: '',
    rating: '',
    votecount: '',
    description: '',
    coverPath: '',
    tags: [],
    developers: '',
  };
}

test('空串与非数字都归 null', () => {
  expect(toNumber('')).toBeNull();
  expect(toNumber('   ')).toBeNull();
  expect(toNumber('abc')).toBeNull();
  expect(toNumber('Infinity')).toBeNull();
  expect(toNumber('8.63')).toBe(8.63);
});

test('评分人数取整', () => {
  expect(toInteger('19334')).toBe(19334);
  expect(toInteger('19334.7')).toBe(19334);
  expect(toInteger('')).toBeNull();
});

test('空白表单写出的全是 null', () => {
  expect(buildMetadataPatch(blank())).toEqual({
    alias: null,
    released: null,
    rating: null,
    votecount: null,
    description: null,
    image_path: null,
    tags: [],
    developers: [],
  });
});

test('开发商按逗号切分并去空白', () => {
  const fields = { ...blank(), developers: ' TYPE-MOON ,同人社团 ,, ' };
  expect(buildMetadataPatch(fields).developers).toEqual(['TYPE-MOON', '同人社团']);
});

/** 只给 name 的标签往返后其余键仍在，重建成全 null 会让排序键与剧透等级失效 */
test('标签原样透传', () => {
  const tags = [{ id: 'g1', name: 'Gore', category: 'cont', rating: 3, vn_count: 640, spoiler: 0 }];
  expect(buildMetadataPatch({ ...blank(), tags }).tags).toEqual(tags);
});

/** titles 与 image_id 不在键里：双层 Option 缺外层才是「不动」，
 *  写成 null 会被后端当成显式置空 */
test('不写 titles 与 image_id', () => {
  const patch = buildMetadataPatch(blank());
  expect(Object.keys(patch)).not.toContain('titles');
  expect(Object.keys(patch)).not.toContain('image_id');
});

test('只有空白与全角空格的字段算空', () => {
  const fields = { ...blank(), alias: '  ', description: '　' };
  const patch = buildMetadataPatch(fields);
  expect(patch.alias).toBeNull();
  expect(patch.description).toBeNull();
});