const MERGE_PHASES = new Set(['images']);

export function mergeProgress(logs) {
  const out = [];
  for (const row of logs) {
    const ev = row.event;
    const prev = out[out.length - 1];
    const same =
      prev &&
      MERGE_PHASES.has(ev.phase) &&
      !ev.error &&
      prev.event.phase === ev.phase &&
      prev.event.game_id === ev.game_id;
    if (same) out[out.length - 1] = row;
    else out.push(row);
  }
  return out;
}
