import { expect, test } from 'vitest';

import {
  BANNER_ASPECT,
  clampFocal,
  focalFromPointer,
  focalToFrame,
  frameSize,
  frameToFocal,
  grabOffset,
} from './cover-focal.js';

test('竖图取景框占满宽度，只剩上下可移', () => {
  expect(frameSize(600, 1600, 3 / 4)).toEqual({ w: 600, h: 800 });
});

test('横图取景框占满高度，只剩左右可移', () => {
  expect(frameSize(1600, 600, 3 / 4)).toEqual({ w: 450, h: 600 });
});

test('竖图的 x 轴与横图的 y 轴不产生位移', () => {
  expect(focalToFrame(30, 0, 600, 1600).fx).toBe(0);
  expect(focalToFrame(30, 0, 1600, 600).fy).toBe(0);
});

test('焦点与取景框位置往返一致', () => {
  for (const [x, y] of [
    [50, 0],
    [50, 100],
    [50, 25],
    [50, 75],
    [50, 50],
  ]) {
    const p = focalToFrame(x, y, 600, 1600);
    expect(frameToFocal(p.fx, p.fy, 600, 1600)).toEqual({ x, y });
  }
  for (const [x, y] of [
    [0, 50],
    [100, 50],
    [25, 50],
    [75, 50],
    [50, 50],
  ]) {
    const p = focalToFrame(x, y, 1600, 600);
    expect(frameToFocal(p.fx, p.fy, 1600, 600)).toEqual({ x, y });
  }
});

test('无效轴上的焦点归一为默认值', () => {
  expect(frameToFocal(0, 400, 600, 1600).x).toBe(50);
  expect(frameToFocal(0, 300, 1600, 600).y).toBe(50);
});

test('框内按下：抓取偏移保持抓住的点不动', () => {
  expect(grabOffset(100, 300, 0, 200, 600, 800, true)).toEqual({ ox: -100, oy: -100 });
});

test('框外按下：取景框中心对准指针', () => {
  expect(grabOffset(300, 700, 0, 200, 600, 800, false)).toEqual({ ox: -300, oy: -400 });
});

test('框跟手：指针同向移动，焦点同向增减', () => {
  const grab = { ox: 0, oy: 0 };
  // 竖图：指针 y 400 → 500（下移 100），焦点 50 → 63（rangeY 800）
  expect(focalFromPointer(300, 500, grab, 600, 1600)).toEqual({ x: 50, y: 63 });
  // 指针 y 400 → 300（上移 100），焦点 50 → 38
  expect(focalFromPointer(300, 300, grab, 600, 1600)).toEqual({ x: 50, y: 38 });
  // 横图：指针 x 575 → 675（右移 100），焦点 50 → 59（rangeX 1150）
  expect(focalFromPointer(675, 300, grab, 1600, 600).x).toBe(59);
  expect(focalFromPointer(475, 300, grab, 1600, 600).x).toBe(41);
});

test('框内抓取后原地不动，焦点不变', () => {
  const grab = grabOffset(100, 300, 0, 200, 600, 800, true);
  expect(focalFromPointer(100, 300, grab, 600, 1600)).toEqual({ x: 50, y: 25 });
});

test('拖到图外：焦点贴边不穿透', () => {
  const grab = { ox: 0, oy: 0 };
  expect(focalFromPointer(-500, -500, grab, 600, 1600)).toEqual({ x: 50, y: 0 });
  expect(focalFromPointer(9999, 9999, grab, 600, 1600)).toEqual({ x: 50, y: 100 });
});

test('无效轴上指针任意位置，焦点锁默认值', () => {
  const grab = { ox: 0, oy: 0 };
  expect(focalFromPointer(9999, 400, grab, 600, 1600)).toEqual({ x: 50, y: 50 });
  expect(focalFromPointer(800, 9999, grab, 1600, 600).y).toBe(50);
});

test('焦点钳制在 0 到 100，异常值回落默认', () => {
  expect(clampFocal(-30)).toBe(0);
  expect(clampFocal(140)).toBe(100);
  expect(clampFocal(49.6)).toBe(50);
  expect(clampFocal(Number.NaN)).toBe(50);
  expect(clampFocal('abc')).toBe(50);
});

test('横幅画幅把 16:9 横图裁成一条横带', () => {
  expect(frameSize(1920, 1080, BANNER_ASPECT)).toEqual({ w: 1920, h: 536 });
});

test('横幅画幅下竖版封面只剩极窄的一段可移', () => {
  const size = frameSize(600, 900, BANNER_ASPECT);
  expect(size.h).toBeLessThan(170);
  expect(frameToFocal(0, 900 - size.h, 600, 900, BANNER_ASPECT).y).toBe(100);
});
