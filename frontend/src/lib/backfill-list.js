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

export function visibleRows(rows, filter, targetIds, onlyNoId) {
  const targets = new Set(targetIds ?? []);
  return (rows ?? []).filter((row) => {
    if (onlyNoId && row.vndb_id) return false;
    if (filter === 'todo') return targets.has(row.game_id);
    if (filter === 'ok') return row.state === 'ok';
    if (filter === 'unmatched') return row.state === 'unmatched';
    if (filter === 'failed') return row.state === 'failed';
    return true;
  });
}

export function countsOf(rows, targetIds) {
  const targets = new Set(targetIds ?? []);
  const counts = { todo: 0, all: 0, ok: 0, unmatched: 0, failed: 0 };
  for (const row of rows ?? []) {
    counts.all += 1;
    if (targets.has(row.game_id)) counts.todo += 1;
    if (Object.hasOwn(counts, row.state)) counts[row.state] += 1;
  }
  return counts;
}

export function pickRow(rows, gameId) {
  if (!gameId) return null;
  return (rows ?? []).find((row) => row.game_id === gameId) ?? null;
}
