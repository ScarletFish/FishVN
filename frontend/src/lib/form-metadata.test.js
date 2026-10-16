import { expect, test } from 'vitest';
import {
  buildMetadataPatch,
  clampRating,
  cleanRating,
  ratingRejected,
  toInteger,
  toNumber,
  votecountRejected,
} from './form-metadata.js';

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

test('评分输入只留数字与一位小数，输入阶段不夹范围', () => {
  expect(cleanRating('8..5')).toBe('8.5');
  expect(cleanRating('8.6.3')).toBe('8.6');
  expect(cleanRating('10.99')).toBe('10.9');
  expect(cleanRating('10')).toBe('10');
  expect(cleanRating('abc')).toBe('');
});

test('失焦才夹到 0..100 并保留一位小数', () => {
  expect(clampRating(8.63)).toBe(8.6);
  expect(clampRating(8.65)).toBe(8.7);
  expect(clampRating(-5)).toBe(0);
  expect(clampRating(105)).toBe(100);
});

test('评分只拦越界与非数字，没填的不算错', () => {
  expect(ratingRejected('')).toBe(false);
  expect(ratingRejected('   ')).toBe(false);
  expect(ratingRejected('100')).toBe(false);
  expect(ratingRejected('86.3')).toBe(false);
  expect(ratingRejected('101')).toBe(true);
  expect(ratingRejected('-1')).toBe(true);
  expect(ratingRejected('abc')).toBe(true);
});

test('评分人数只拦负数与非数字，没填的不算错', () => {
  expect(votecountRejected('')).toBe(false);
  expect(votecountRejected('0')).toBe(false);
  expect(votecountRejected('19334')).toBe(false);
  expect(votecountRejected('19334.7')).toBe(false);
  expect(votecountRejected('-1')).toBe(true);
  expect(votecountRejected('abc')).toBe(true);
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