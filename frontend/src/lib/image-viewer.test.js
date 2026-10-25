import { expect, test } from 'vitest';

import {
  VIEWER_DEFAULT_MAX_ZOOM,
  VIEWER_MIN_ZOOM,
  clampViewScale,
  fitView,
  hundredView,
  imageToScreen,
  isFitView,
  panView,
  screenToImage,
  zoomAtView,
} from './image-viewer.js';

test('竖图适应窗口：高度铺满、左右留空居中', () => {
  expect(fitView(600, 1600, 400, 600)).toEqual({
    scale: 0.375,
    tx: 87.5,
    ty: 0,
  });
});

test('横图适应窗口：宽度铺满、上下留空居中', () => {
  expect(fitView(1600, 600, 400, 600)).toEqual({
    scale: 0.25,
    tx: 0,
    ty: 225,
  });
});

test('空输入不炸，回落单位视图', () => {
  expect(fitView(0, 0, 400, 600)).toEqual({ scale: 1, tx: 0, ty: 0 });
  expect(fitView(600, 1600, 0, 0)).toEqual({ scale: 1, tx: 0, ty: 0 });
});

test('缩放保持指针下图内容不动', () => {
  const fit = fitView(600, 1600, 400, 600);
  const zoomed = zoomAtView(fit, 2, 200, 300);
  const anchor = screenToImage(fit, 200, 300);
  const after = imageToScreen(zoomed, anchor.x, anchor.y);
  expect(after.x).toBeCloseTo(200, 6);
  expect(after.y).toBeCloseTo(300, 6);
});

test('缩放夹在上下限之间', () => {
  const view = { scale: 1, tx: 0, ty: 0 };
  expect(zoomAtView(view, 100, 10, 10).scale).toBe(VIEWER_DEFAULT_MAX_ZOOM);
  expect(zoomAtView(view, 0.001, 10, 10).scale).toBe(VIEWER_MIN_ZOOM);
  expect(clampViewScale({ scale: 99, tx: 0, ty: 0 }).scale).toBe(
    VIEWER_DEFAULT_MAX_ZOOM
  );
  expect(clampViewScale({ scale: 0.001, tx: 0, ty: 0 }, 0.5).scale).toBe(0.5);
});

test('clampViewScale 在范围内时原样返回', () => {
  const view = { scale: 1.5, tx: 3, ty: 4 };
  expect(clampViewScale(view)).toBe(view);
});

test('屏幕坐标与图坐标往返一致', () => {
  const view = { scale: 0.75, tx: -25, ty: -300 };
  const img = screenToImage(view, 200, 300);
  const screen = imageToScreen(view, img.x, img.y);
  expect(screen.x).toBeCloseTo(200, 6);
  expect(screen.y).toBeCloseTo(300, 6);
});

test('1:1 视图把图中心摆到指定视口点', () => {
  const view = hundredView(600, 1600, 200, 300);
  const center = imageToScreen(view, 300, 800);
  expect(center).toEqual({ x: 200, y: 300 });
});

test('平移只改偏移不改缩放', () => {
  const view = panView({ scale: 0.5, tx: 10, ty: 20 }, -5, 15);
  expect(view).toEqual({ scale: 0.5, tx: 5, ty: 35 });
});

test('isFitView 只认适应窗口的缩放比', () => {
  expect(isFitView(fitView(600, 1600, 400, 600), 600, 1600, 400, 600)).toBe(true);
  expect(isFitView({ scale: 1, tx: 0, ty: 0 }, 600, 1600, 400, 600)).toBe(false);
  expect(isFitView({ scale: 0.376, tx: 0, ty: 0 }, 600, 1600, 400, 600)).toBe(
    false
  );
});
