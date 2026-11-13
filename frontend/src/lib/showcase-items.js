export const SHOWCASE_LIMIT = 30;

function num(value, fallback) {
  if (value === null || value === undefined || value === '') return fallback;
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}

function rank(item) {
  if (num(item?.last_played_at, 0) > 0) return 0;
  return item?.status === 'playing' ? 1 : 2;
}

export function compareShowcaseItems(a, b) {
  const ra = rank(a);
  const rb = rank(b);
  if (ra !== rb) return ra - rb;
  if (ra === 0) {
    const delta = num(b?.last_played_at, 0) - num(a?.last_played_at, 0);
    if (delta !== 0) return delta;
  }
  return String(a?.name ?? '').localeCompare(String(b?.name ?? ''), 'zh-CN');
}

export function buildShowcaseItems(rows, limit = SHOWCASE_LIMIT) {
  const cap = Math.max(0, num(limit, 0));
  if (cap === 0) return [];
  const seen = new Set();
  const out = [];
  for (const row of Array.isArray(rows) ? rows : []) {
    if (!row || row.id === null || row.id === undefined || seen.has(row.id)) continue;
    seen.add(row.id);
    out.push(row);
  }
  out.sort(compareShowcaseItems);
  return out.slice(0, cap);
}

export function stageDescriptor(item) {
  const custom = String(item?.ambience_path ?? '').trim();
  const path = custom || String(item?.cover_path ?? '').trim();
  const mode = item?.ambience_mode === 'clear' ? 'clear' : 'soft';
  return {
    path,
    mode,
    focal: `${num(item?.ambience_focal_x, 50)}% ${num(item?.ambience_focal_y, 22)}%`,
    coverFocal: `${num(item?.cover_focal_x, 50)}% ${num(item?.cover_focal_y, 50)}%`,
  };
}
