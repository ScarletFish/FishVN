import { expect, test } from 'vitest';
import { staffDiff, staffGroupText } from './staff.js';

const entry = (role, name, original = null, note = null) => ({ role, name, original, note });

test('staffDiff 只列出姓名集合不一致的职能，其余计数相同职能', () => {
  const current = [
    entry('scenario', '打越 鋼太郎', 'Uchikoshi Koutarou'),
    entry('music', 'Amao'),
    entry('director', '江城戸 由貴'),
  ];
  const incoming = [
    entry('scenario', '打越 鋼太郎', 'Uchikoshi Koutarou'),
    entry('music', 'Amao', null, 'OP 主题曲'),
    entry('director', '別の人物'),
    entry('art', 'ゆーげん'),
  ];
  expect(staffDiff(current, incoming)).toEqual({
    labels: ['导演', '原画'],
    rest: 2,
  });
});

test('staffDiff 的职能顺序按当前侧在前、新职能在后', () => {
  const current = [entry('director', 'A')];
  const incoming = [entry('music', 'B'), entry('director', 'A')];
  expect(staffDiff(current, incoming)).toEqual({
    labels: ['音乐'],
    rest: 1,
  });
});

test('staffDiff 对顺序无关的同名集合判为一致', () => {
  const current = [entry('music', 'A'), entry('music', 'B')];
  const incoming = [entry('music', 'B'), entry('music', 'A')];
  expect(staffDiff(current, incoming)).toEqual({ labels: [], rest: 1 });
});

test('staffDiff 在仅备注不同时返回空差异', () => {
  const current = [entry('music', 'Amao', null, '旧备注')];
  const incoming = [entry('music', 'Amao', null, '新备注')];
  expect(staffDiff(current, incoming)).toEqual({ labels: [], rest: 1 });
});

test('staffGroupText 按传入职能逐行渲染，缺失的一侧显示占位符', () => {
  const list = [entry('music', 'Amao'), entry('scenario', '打越 鋼太郎')];
  expect(staffGroupText(list, ['剧本', '原画'])).toBe('剧本：打越 鋼太郎\n原画：—');
});

test('staffGroupText 对空列表渲染全部占位符', () => {
  expect(staffGroupText([], ['剧本', '音乐'])).toBe('剧本：—\n音乐：—');
});
