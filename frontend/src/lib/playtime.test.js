import { expect, test } from 'vitest';
import { compactDuration, durationInput, durationText, parseDuration } from './playtime.js';

const t = (key) =>
  ({ 'common.unitHour': '小时', 'common.unitMinute': '分' })[key];

test('时长文案按分钟四舍五入', () => {
  expect(durationText(t, 0)).toBe('0');
  expect(durationText(t, 20)).toBe('<1 分');
  expect(durationText(t, 34 * 60)).toBe('34 分');
  expect(durationText(t, 3600)).toBe('1 小时');
  expect(durationText(t, 12 * 3600 + 34 * 60 + 56)).toBe('12 小时 35 分');
});

test('紧凑时长小时进位带两位分钟，不足一小时为 m', () => {
  expect(compactDuration(0)).toBe('0m');
  expect(compactDuration(45)).toBe('45m');
  expect(compactDuration(59)).toBe('59m');
  expect(compactDuration(60)).toBe('1h00m');
  expect(compactDuration(1820)).toBe('30h20m');
  expect(compactDuration(1949.7)).toBe('32h29m');
});

test('输入框回填整秒不丢精度', () => {
  expect(durationInput(0)).toBe('');
  expect(durationInput(34 * 60)).toBe('0:34');
  expect(durationInput(12 * 3600 + 34 * 60)).toBe('12:34');
  expect(durationInput(12 * 3600 + 34 * 60 + 56)).toBe('12:34:56');
});

test('冒号写法按小时分钟解析', () => {
  expect(parseDuration('12:34')).toBe(12 * 3600 + 34 * 60);
  expect(parseDuration('0:34')).toBe(34 * 60);
  expect(parseDuration('1:02:03')).toBe(3600 + 2 * 60 + 3);
});

test('带单位的写法按单位解析', () => {
  expect(parseDuration('12h30m')).toBe(12 * 3600 + 30 * 60);
  expect(parseDuration('1.5h')).toBe(5400);
  expect(parseDuration('12 小时 30 分')).toBe(12 * 3600 + 30 * 60);
  expect(parseDuration('45秒')).toBe(45);
  expect(parseDuration('2h')).toBe(7200);
});

test('英文长单位不会被短单位抢先截断', () => {
  expect(parseDuration('12min')).toBe(12 * 60);
  expect(parseDuration('2minutes')).toBe(120);
  expect(parseDuration('1.5hours')).toBe(5400);
  expect(parseDuration('12hr')).toBe(12 * 3600);
  expect(parseDuration('45sec')).toBe(45);
  expect(parseDuration('1h30min')).toBe(5400);
  expect(parseDuration('12 小时 30 分钟')).toBe(12 * 3600 + 30 * 60);
});

test('纯数字按秒解析', () => {
  expect(parseDuration('45000')).toBe(45000);
  expect(parseDuration('0')).toBe(0);
  expect(parseDuration(' 90 ')).toBe(90);
});

test('认不出来的输入返回空', () => {
  expect(parseDuration('')).toBeNull();
  expect(parseDuration('   ')).toBeNull();
  expect(parseDuration('abc')).toBeNull();
  expect(parseDuration('12:75')).toBeNull();
  expect(parseDuration('12x')).toBeNull();
});
