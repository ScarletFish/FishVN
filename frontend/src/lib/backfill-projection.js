const TERMINAL_PHASES = new Set([
  'finished',
  'failed',
  'unmatched',
  'resolve',
  'rate-limited',
  'paused',
]);

export function needsReload(ev) {
  if (!ev) return false;
  return Boolean(ev.error) || TERMINAL_PHASES.has(ev.phase);
}

export function projectEvent(status, ev) {
  if (!status || !ev) return status;
  const done = ev.done ?? status.done;
  const total = ev.total ?? status.total;
  let items = status.items ?? [];
  if (ev.game_id) {
    items = items.map((item) =>
      item.game_id === ev.game_id && item.state === 'pending'
        ? { ...item, state: 'active' }
        : item,
    );
  }
  if (done === status.done && total === status.total && items === (status.items ?? [])) {
    return status;
  }
  return { ...status, done, total, items };
}

const LOG_LIMIT = 400;

export function mergeLogRows(current, incoming) {
  const bySeq = new Map(
    current
      .filter((row) => Number.isFinite(row.seq) && row.seq > 0)
      .map((row) => [row.seq, row]),
  );
  for (const row of incoming) {
    if (!Number.isFinite(row.seq) || row.seq <= 0) continue;
    bySeq.set(row.seq, row);
  }
  return [...bySeq.values()].sort((a, b) => a.seq - b.seq).slice(-LOG_LIMIT);
}

export function appendEvent(logs, ev, name = 'vndb-backfill') {
  if (!ev || !Number.isFinite(ev.seq) || ev.seq <= 0) return logs;
  return mergeLogRows(logs, [{ seq: ev.seq, name, event: ev }]);
}
