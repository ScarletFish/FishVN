export function staffDisplay(s) {
  return s?.displayName || s?.name || s?.original || s?.id || '';
}

export function staffOriginal(s) {
  return s?.displayOriginal || s?.original || '';
}

export function staffLabel(s) {
  const base = staffDisplay(s);
  const orig = staffOriginal(s);
  return orig && orig !== base ? `${base}（${orig}）` : base;
}
