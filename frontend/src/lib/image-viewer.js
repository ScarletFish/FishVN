export const VIEWER_MIN_ZOOM = 0.1;
export const VIEWER_DEFAULT_MAX_ZOOM = 4;

export function fitView(iw, ih, vw, vh) {
  if (!iw || !ih || !vw || !vh) return { scale: 1, tx: 0, ty: 0 };
  const scale = Math.min(vw / iw, vh / ih);
  return {
    scale,
    tx: (vw - iw * scale) / 2,
    ty: (vh - ih * scale) / 2,
  };
}

export function clampViewScale(
  view,
  min = VIEWER_MIN_ZOOM,
  max = VIEWER_DEFAULT_MAX_ZOOM
) {
  const scale = Math.min(max, Math.max(min, view.scale));
  return scale === view.scale ? view : { ...view, scale };
}

export function zoomAtView(
  view,
  factor,
  px,
  py,
  min = VIEWER_MIN_ZOOM,
  max = VIEWER_DEFAULT_MAX_ZOOM
) {
  const scale = Math.min(max, Math.max(min, view.scale * factor));
  const k = scale / view.scale;
  return {
    scale,
    tx: px - (px - view.tx) * k,
    ty: py - (py - view.ty) * k,
  };
}

export function panView(view, dx, dy) {
  return { scale: view.scale, tx: view.tx + dx, ty: view.ty + dy };
}

export function screenToImage(view, sx, sy) {
  return {
    x: (sx - view.tx) / view.scale,
    y: (sy - view.ty) / view.scale,
  };
}

export function imageToScreen(view, ix, iy) {
  return {
    x: ix * view.scale + view.tx,
    y: iy * view.scale + view.ty,
  };
}

export function hundredView(iw, ih, cx, cy) {
  return { scale: 1, tx: cx - iw / 2, ty: cy - ih / 2 };
}

export function isFitView(view, iw, ih, vw, vh, epsilon = 0.001) {
  const fit = fitView(iw, ih, vw, vh);
  return Math.abs(view.scale - fit.scale) < epsilon;
}
