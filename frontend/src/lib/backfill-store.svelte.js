import { api } from './api.js';
import { subscribe } from './events.js';
import { BACKFILL_REFRESH } from './task-events.js';

export const backfillStore = $state({
  status: null,
  loading: true,
  error: null,
  filter: 'all',
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
let pulling = false;
let pullAgain = false;

async function load() {
  const mine = ++seq;
  try {
    const fresh = await api.backfillStatus();
    if (mine !== seq) return;
    backfillStore.status = fresh;
    backfillStore.error = null;
    if (backfillStore.selected && !fresh.items.some((it) => it.game_id === backfillStore.selected)) {
      backfillStore.selected = null;
    }
  } catch (e) {
    if (mine !== seq) return;
    backfillStore.error = e.message;
  } finally {
    if (mine === seq) backfillStore.loading = false;
  }
}

function schedule() {
  if (timer) return;
  timer = setTimeout(() => {
    timer = null;
    load();
    pullLogs();
  }, 200);
}

async function pullLogs() {
  if (pulling) {
    pullAgain = true;
    return;
  }
  pulling = true;
  try {
    const rows = await api.eventsSince(backfillStore.lastSeq);
    if (rows.length) {
      backfillStore.lastSeq = rows[rows.length - 1].seq;
      backfillStore.logs = [...backfillStore.logs, ...rows].slice(-400);
    }
  } catch {
  } finally {
    pulling = false;
    if (pullAgain) {
      pullAgain = false;
      if (watchers > 0) pullLogs();
    }
  }
}

function onEvent(ev) {
  if (!ev) return;
  schedule();
}

export async function watchBackfill() {
  watchers += 1;
  if (watchers === 1) {
    await load();
    await pullLogs();
    unsub = await subscribe('vndb-backfill', onEvent);
    window.addEventListener(BACKFILL_REFRESH, schedule);
  }
}

export function unwatchBackfill() {
  watchers = Math.max(0, watchers - 1);
  if (watchers > 0) return;
  unsub?.();
  unsub = null;
  window.removeEventListener(BACKFILL_REFRESH, schedule);
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  pullAgain = false;
}

export { load as reloadBackfill };

export function backfillCounts(status) {
  const items = status?.items ?? [];
  const counts = { all: items.length, pending: 0, active: 0, ok: 0, unmatched: 0, failed: 0 };
  for (const it of items) counts[it.state] = (counts[it.state] ?? 0) + 1;
  return counts;
}

export function backfillVisible(status, filter, onlyNoId) {
  let list = status?.items ?? [];
  if (filter !== 'all') list = list.filter((it) => it.state === filter);
  if (onlyNoId) list = list.filter((it) => it.bind_source === 'search');
  return list;
}

export function backfillItem(status, gameId) {
  if (!gameId) return null;
  return (status?.items ?? []).find((it) => it.game_id === gameId) ?? null;
}
