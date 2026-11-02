export const FOCAL_DEFAULT = 50;
export const FOCAL_MIN = 0;
export const FOCAL_MAX = 100;
export const COVER_ASPECT = 3 / 4;
export const BANNER_ASPECT = 1200 / 335;
export const BANNER_FOCAL = { x: 50, y: 22 };

export function clampFocal(value) {
  if (!Number.isFinite(value)) return FOCAL_DEFAULT;
  return Math.min(FOCAL_MAX, Math.max(FOCAL_MIN, Math.round(value)));
}

export function frameSize(iw, ih, aspect = COVER_ASPECT) {
  if (!iw || !ih || !aspect) return { w: 0, h: 0 };
  const byWidth = { w: iw, h: iw / aspect };
  return byWidth.h <= ih ? byWidth : { w: ih * aspect, h: ih };
}

export function focalToFrame(x, y, iw, ih, aspect = COVER_ASPECT) {
  const f = frameSize(iw, ih, aspect);
  const rangeX = iw - f.w;
  const rangeY = ih - f.h;
  return {
    fx: rangeX > 0 ? (x / 100) * rangeX : 0,
    fy: rangeY > 0 ? (y / 100) * rangeY : 0,
  };
}

export function frameToFocal(fx, fy, iw, ih, aspect = COVER_ASPECT) {
  const f = frameSize(iw, ih, aspect);
  const rangeX = iw - f.w;
  const rangeY = ih - f.h;
  return {
    x: clampFocal(rangeX > 0 ? (fx / rangeX) * 100 : FOCAL_DEFAULT),
    y: clampFocal(rangeY > 0 ? (fy / rangeY) * 100 : FOCAL_DEFAULT),
  };
}

export function grabOffset(px, py, fx, fy, fw, fh, inside) {
  return inside
    ? { ox: fx - px, oy: fy - py }
    : { ox: -fw / 2, oy: -fh / 2 };
}

export function focalFromPointer(px, py, grab, iw, ih, aspect = COVER_ASPECT) {
  return frameToFocal(px + grab.ox, py + grab.oy, iw, ih, aspect);
}
