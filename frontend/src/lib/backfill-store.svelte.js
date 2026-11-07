import { api } from './api.js';
import { subscribe } from './events.js';
import { LIBRARY_CHANGED } from './library-events.js';
import { BACKFILL_REFRESH } from './task-events.js';
import { appendEvent, isItemDone, mergeLogRows, needsReload, patchItem, projectEvent } from './backfill-projection.js';

export const backfillStore = $state({
  status: null,
  games: [],
  targetIds: [],
  localOverlay: {},
  loading: true,
  error: null,
  filter: 'todo',
  onlyNoId: false,
  selected: null,
  logs: [],
  lastSeq: 0,
  logHidden: false,
});

let unsub = null;
let watchers = 0;
let timer = null;
let seq = 0;
let frozenRun = null;

async function load() {
  const mine = ++seq;
  try {
    const fresh = await api.backfillStatus();
    if (mine !== seq) return;
    backfillStore.status = fresh;
    backfillStore.error = null;
    if (frozenRun !== fresh.run_id) {
      frozenRun = fresh.run_id;
      backfillStore.targetIds = (fresh.items ?? []).map((it) => it.game_id);
    }
  } catch (e) {
    if (mine !== seq) return;
    backfillStore.error = e.message;
  } finally {
    if (mine === seq) backfillStore.loading = false;
  }
}

async function loadGames() {
  try {
    const games = await api.listGames();
    backfillStore.games = games;
    const alive = new Set(games.map((g) => g.id));
    const overlay = { ...backfillStore.localOverlay };
    for (const id of Object.keys(overlay)) {
      if (!alive.has(id)) delete overlay[id];
    }
    backfillStore.localOverlay = overlay;
    if (backfillStore.selected && !alive.has(backfillStore.selected)) {
      backfillStore.selected = null;
    }
  } catch (e) {
    backfillStore.error = e.message;
  }
}

function schedule() {
  if (timer) return;
  timer = setTimeout(() => {
    timer = null;
    load();
  }, 200);
}

async function pullLogs(after) {
  try {
    const rows = await api.eventsSince(after);
    if (!rows.length) return;
    backfillStore.logs = mergeLogRows(backfillStore.logs, rows);
    const last = rows[rows.length - 1]?.seq ?? 0;
    backfillStore.lastSeq = Math.max(backfillStore.lastSeq, last);
  } catch {
  }
}

function refreshItem(gameId) {
  const runId = backfillStore.status?.run_id;
  api
    .backfillItem(gameId)
    .then((item) => {
      if (backfillStore.status?.run_id !== runId) return;
      backfillStore.status = patchItem(backfillStore.status, item);
    })
    .catch(() => {});
}

function onEvent(ev) {
  if (!ev) return;
  backfillStore.status = projectEvent(backfillStore.status, ev);
  backfillStore.logs = appendEvent(backfillStore.logs, ev);
  if (Number.isFinite(ev.seq)) {
    backfillStore.lastSeq = Math.max(backfillStore.lastSeq, ev.seq);
  }
  if (isItemDone(ev)) refreshItem(ev.game_id);
  if (needsReload(ev)) schedule();
}

export async function watchBackfill() {
  watchers += 1;
  if (watchers === 1) {
    frozenRun = null;
    await load();
    await loadGames();
    const after = backfillStore.lastSeq;
    unsub = await subscribe('vndb-backfill', onEvent);
    await pullLogs(after);
    window.addEventListener(BACKFILL_REFRESH, schedule);
    window.addEventListener(LIBRARY_CHANGED, loadGames);
  }
}

export function unwatchBackfill() {
  watchers = Math.max(0, watchers - 1);
  if (watchers > 0) return;
  unsub?.();
  unsub = null;
  window.removeEventListener(BACKFILL_REFRESH, schedule);
  window.removeEventListener(LIBRARY_CHANGED, loadGames);
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
}

export { load as reloadBackfill };

export function mergeBackfillItem(patch) {
  const id = patch?.game_id;
  if (!id) return;
  const at = backfillStore.games.findIndex((g) => g.id === id);
  if (at < 0) return;
  if (Object.hasOwn(patch, 'vndb_id')) {
    backfillStore.games[at].vndb_id = patch.vndb_id;
  }
  backfillStore.localOverlay = {
    ...backfillStore.localOverlay,
    [id]: { ...(backfillStore.localOverlay[id] ?? {}), ...patch },
  };
}
