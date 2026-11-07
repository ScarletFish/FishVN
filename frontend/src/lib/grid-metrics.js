export function pxLength(el, raw, fallback = 0) {
  const v = parseFloat(raw);
  if (!Number.isFinite(v)) return fallback;
  return String(raw).trim().endsWith('rem') ? v * (parseFloat(getComputedStyle(el).fontSize) || 16) : v;
}

export function computeColsFor(el, { baselineW, gap } = {}) {
  const cs = getComputedStyle(el);
  const g = gap ?? (parseFloat(cs.gap) || parseFloat(cs.columnGap) || 12);
  const padX = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
  const contentW = Math.max(0, el.clientWidth - padX);
  const kept = parseInt(el.style.getPropertyValue('--cols'), 10) || 1;
  if (contentW <= 0) return kept;
  const baseline = baselineW ?? pxLength(el, cs.getPropertyValue('--card-w'), 320);
  if (!(baseline > 0)) return kept;
  return Math.max(1, Math.floor((contentW + g) / (Math.min(baseline, contentW) + g)));
}

export function cardWidth(contentW, cols, gap) {
  return (contentW - gap * (cols - 1)) / cols;
}

export function cardStep(contentW, cols, gap) {
  return cardWidth(contentW, cols, gap) + gap;
}
