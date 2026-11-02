import { expect, test } from 'vitest';
import {
  buildMetadataPatch,
  buildStatePatch,
  clampRating,
  cleanInteger,
  cleanRating,
  lengthMinutesRejected,
  normalizeInteger,
  normalizeRating,
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
    lengthMinutes: '',
    description: '',
    coverPath: '',
    ambiencePath: '',
    ambienceMode: 'soft',
    ambiencePlate: false,
    ambienceFocalX: 50,
    ambienceFocalY: 22,
    tags: [],
    developers: '',
    staff: [],
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

test('评分整数位最多三位，第四位进不来', () => {
  expect(cleanRating('9999')).toBe('999');
  expect(cleanRating('1234.5')).toBe('123.5');
  expect(cleanRating('100')).toBe('100');
});

test('整数框只留数字且限位数', () => {
  expect(cleanInteger('19,334')).toBe('19334');
  expect(cleanInteger(' 1820 分钟 ')).toBe('1820');
  expect(cleanInteger('12345678901', 7)).toBe('1234567');
  expect(cleanInteger('')).toBe('');
});

test('整数框离焦才钳范围，空着仍算空', () => {
  expect(normalizeInteger('')).toBe('');
  expect(normalizeInteger('  ')).toBe('');
  expect(normalizeInteger('1820', { max: 99999 })).toBe('1820');
  expect(normalizeInteger('999999999', { max: 99999, maxIntDigits: 9 })).toBe('99999');
  expect(normalizeInteger('-5')).toBe('5');
});

test('评分离焦规整：钳到 0–100，空着仍算空', () => {
  expect(normalizeRating('')).toBe('');
  expect(normalizeRating('.')).toBe('');
  expect(normalizeRating('8.65')).toBe('8.6');
  expect(normalizeRating('9999')).toBe('100');
  expect(normalizeRating('0')).toBe('0');
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

test('平均时长只拦负数与非数字，没填的不算错', () => {
  expect(lengthMinutesRejected('')).toBe(false);
  expect(lengthMinutesRejected('0')).toBe(false);
  expect(lengthMinutesRejected('1820')).toBe(false);
  expect(lengthMinutesRejected('-5')).toBe(true);
  expect(lengthMinutesRejected('abc')).toBe(true);
});

test('空白表单其余字段全 null，焦点回落默认值', () => {
  expect(buildMetadataPatch(blank())).toEqual({
    alias: null,
    released: null,
    rating: null,
    votecount: null,
    length_minutes: null,
    description: null,
    image_path: null,
    cover_focal_x: 50,
    cover_focal_y: 50,
    ambience_path: null,
    ambience_mode: 'soft',
    ambience_plate: false,
    ambience_focal_x: 50,
    ambience_focal_y: 22,
    tags: [],
    developers: [],
    staff: [],
  });
});

test('焦点随表单值写入并钳制范围', () => {
  const patch = buildMetadataPatch({ ...blank(), focusX: 130, focusY: -20 });
  expect(patch.cover_focal_x).toBe(100);
  expect(patch.cover_focal_y).toBe(0);
  expect(buildMetadataPatch({ ...blank(), focusX: 72, focusY: 31 })).toMatchObject({
    cover_focal_x: 72,
    cover_focal_y: 31,
  });
});

test('没有名字的 staff 行在写补丁时丢弃', () => {
  const fields = {
    ...blank(),
    staff: [
      { role: 'scenario', name: '  ', original: '打越 鋼太郎', note: null },
      { role: 'music', name: '', original: '', note: null },
      { role: 'art', name: 'Kazuaki', original: null, note: 'x' },
    ],
  };
  expect(buildMetadataPatch(fields).staff).toEqual([
    { role: 'scenario', name: null, original: '打越 鋼太郎', note: null },
    { role: 'art', name: 'Kazuaki', original: null, note: 'x' },
  ]);
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

/** 回车提交时输入框未失焦，blur 的钳制不会触发，所以写库前必须再钳一次 */
test('状态补丁钳制越界评分', () => {
  const base = { status: 'playing', notes: ' ' };
  expect(buildStatePatch({ ...base, userRating: 9999 }).user_rating).toBe(100);
  expect(buildStatePatch({ ...base, userRating: -3 }).user_rating).toBe(0);
  expect(buildStatePatch({ ...base, userRating: 87.64 }).user_rating).toBe(87.6);
  expect(buildStatePatch({ ...base, userRating: null }).user_rating).toBeNull();
  expect(buildStatePatch({ ...base, userRating: 0 }).user_rating).toBe(0);
});

test('状态补丁去空白，空笔记归 null', () => {
  expect(buildStatePatch({ status: 'wishlist', userRating: null, notes: ' 好 ' })).toEqual({
    status: 'wishlist',
    user_rating: null,
    notes: '好',
  });
  expect(buildStatePatch({ status: 'wishlist', userRating: 1, notes: '   ' }).notes).toBeNull();
});

/** 没传 total 时不能带 play_seconds：进行中的游玩结算会自己往上加，写回旧值等于倒退 */
test('总时长只在用户改过时才写', () => {
  expect(Object.keys(buildStatePatch({ status: 'playing', userRating: null, notes: '' }))).not.toContain(
    'play_seconds'
  );
  expect(buildStatePatch({ status: 'playing', userRating: null, notes: '', playSeconds: 4500 })).toMatchObject(
    { play_seconds: 4500 }
  );
  expect(buildStatePatch({ status: 'playing', playSeconds: -60 }).play_seconds).toBe(0);
  expect(buildStatePatch({ status: 'playing', playSeconds: 12.6 }).play_seconds).toBe(13);
});