import { titleChain } from './generated/display-titles.js';

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

function shownTitle(game, row) {
  const titles = game.titles ?? [];
  const chain = titleChain({
    titles,
    alttitle: '',
    olang: '',
    latin: titles.find((title) => title.latin)?.latin ?? '',
  });
  return chain[0] ?? game.alias ?? row.hit_title ?? game.vndb_id ?? '';
}

export function mergeCatalog(games, statusItems, localOverlay) {
  const byId = new Map();
  for (const item of statusItems ?? []) byId.set(item.game_id, item);
  for (const [id, patch] of Object.entries(localOverlay ?? {})) {
    byId.set(id, { ...(byId.get(id) ?? {}), ...patch });
  }
  return (games ?? []).map((game) => {
    const row = { ...emptyOverlay(), ...(byId.get(game.id) ?? {}) };
    return {
      ...row,
      game_id: game.id,
      name: game.name,
      vndb_id: game.vndb_id,
      has_metadata: Boolean(game.has_metadata),
      cover_path: game.cover_path ?? null,
      title: shownTitle(game, row),
      year: (game.released ?? row.hit_released ?? '').slice(0, 4),
    };
  });
}

export function focusRows(rows) {
  return new Set(
    (rows ?? []).filter((row) => !row.has_metadata).map((row) => row.game_id),
  );
}

export function visibleRows(rows, inScope, onlyNoId) {
  return (rows ?? []).filter((row) => {
    if (inScope && !inScope.has(row.game_id)) return false;
    if (onlyNoId && row.vndb_id) return false;
    return true;
  });
}

export function rangePick(items, anchorId, id) {
  const ids = (items ?? []).map((it) => it.game_id);
  const to = ids.indexOf(id);
  if (to < 0) return [];
  const from = anchorId ? ids.indexOf(anchorId) : -1;
  if (from < 0) return [id];
  const [lo, hi] = from <= to ? [from, to] : [to, from];
  return ids.slice(lo, hi + 1);
}

export function pickOnClick(state, items, id, mods = {}) {
  const picked = state?.picked ?? [];
  const anchorId = state?.anchorId ?? null;
  const selected = state?.selected ?? null;
  if (!id) return { picked, anchorId, selected };

  if (mods.shift) {
    const span = rangePick(items, anchorId, id);
    const next = mods.toggle ? [...new Set([...picked, ...span])] : span;
    const anchored = (items ?? []).some((it) => it.game_id === anchorId);
    return { picked: next, anchorId: anchored ? anchorId : id, selected: id };
  }

  if (mods.toggle) {
    if (!picked.includes(id)) return { picked: [...picked, id], anchorId: id, selected: id };
    const next = picked.filter((one) => one !== id);
    const lead = selected === id ? (next[next.length - 1] ?? null) : selected;
    return { picked: next, anchorId: id, selected: lead };
  }

  if (picked.length === 1 && picked[0] === id) {
    return { picked: [], anchorId, selected: null };
  }
  return { picked: [id], anchorId: id, selected: id };
}

export function prunePicks(state, alive) {
  const picked = (state?.picked ?? []).filter((id) => alive.has(id));
  const keep = (id) => (id && alive.has(id) ? id : null);
  return {
    picked,
    anchorId: keep(state?.anchorId),
    selected: keep(state?.selected),
  };
}

export function pickRow(rows, gameId) {
  if (!gameId) return null;
  return (rows ?? []).find((row) => row.game_id === gameId) ?? null;
}
