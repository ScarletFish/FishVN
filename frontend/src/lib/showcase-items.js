export const SHOWCASE_LIMIT = 30;

function num(value, fallback) {
  if (value === null || value === undefined || value === '') return fallback;
  const n = Number(value);
  return Number.isFinite(n) ? n : fallback;
}

export function compareShowcaseItems(a, b) {
  const played = num(b?.last_played_at, 0) - num(a?.last_played_at, 0);
  if (played !== 0) return played;
  const modified = num(b?.state_modified_at, 0) - num(a?.state_modified_at, 0);
  if (modified !== 0) return modified;
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
