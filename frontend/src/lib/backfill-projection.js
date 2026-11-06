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
