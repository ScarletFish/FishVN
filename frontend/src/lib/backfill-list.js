function emptyOverlay() {
  return {
    state: null,
    bind_source: null,
    hit_count: 0,
    hit_title: null,
    hit_released: null,
    candidates: [],
    filled: [],
    characters: 0,
    error: null,
  };
}

export function mergeCatalog(games, statusItems, localOverlay) {
  const byId = new Map();
  for (const item of statusItems ?? []) byId.set(item.game_id, item);
  for (const [id, patch] of Object.entries(localOverlay ?? {})) {
    byId.set(id, { ...(byId.get(id) ?? {}), ...patch });
  }
  return (games ?? []).map((game) => ({
    ...emptyOverlay(),
    ...(byId.get(game.id) ?? {}),
    game_id: game.id,
    name: game.name,
    vndb_id: game.vndb_id,
  }));
}

export function focusRows(rows, pendingIds) {
  const pending = new Set(pendingIds ?? []);
  return new Set(
    (rows ?? [])
      .filter(
        (row) => pending.has(row.game_id) || (row.state === 'ok' && row.bind_source === 'search'),
      )
      .map((row) => row.game_id),
  );
}

export function visibleRows(rows, filter, inScope, onlyNoId) {
  return (rows ?? []).filter((row) => {
    if (inScope && !inScope.has(row.game_id)) return false;
    if (filter === 'review') return row.state === 'ok' && row.bind_source === 'search';
    if (onlyNoId && row.vndb_id) return false;
    if (filter === 'ok') return row.state === 'ok';
    if (filter === 'unmatched') return row.state === 'unmatched';
    if (filter === 'failed') return row.state === 'failed';
    return true;
  });
}

export function countsOf(rows, inScope) {
  const counts = { all: 0, ok: 0, review: 0, unmatched: 0, failed: 0 };
  for (const row of rows ?? []) {
    if (inScope && !inScope.has(row.game_id)) continue;
    counts.all += 1;
    if (row.state === 'ok' && row.bind_source === 'search') counts.review += 1;
    if (Object.hasOwn(counts, row.state)) counts[row.state] += 1;
  }
  return counts;
}

export function pickRow(rows, gameId) {
  if (!gameId) return null;
  return (rows ?? []).find((row) => row.game_id === gameId) ?? null;
}
