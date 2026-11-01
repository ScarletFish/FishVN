function pad(value) {
  return String(value).padStart(2, '0');
}

function seconds(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

export function buildCumulativeSeries(sessions, metric = 'foreground') {
  const rows = [];

  for (const session of sessions ?? []) {
    const at = Number(session?.started_at ?? 0);
    if (!Number.isFinite(at) || at <= 0) continue;
    rows.push({
      at,
      foreground: seconds(session?.foreground_seconds),
      alive: seconds(session?.alive_seconds),
      running: session?.ended_at == null,
    });
  }

  rows.sort((a, b) => a.at - b.at);

  const aliveMode = metric === 'alive';
  let foregroundTotal = 0;
  let aliveTotal = 0;

  const points = rows.map((row, index) => {
    foregroundTotal += row.foreground;
    aliveTotal += row.alive;
    return {
      index,
      at: row.at,
      running: row.running,
      step: aliveMode ? row.alive : row.foreground,
      total: aliveMode ? aliveTotal : foregroundTotal,
    };
  });

  return {
    points,
    total: aliveMode ? aliveTotal : foregroundTotal,
    first: points[0]?.at ?? null,
    last: points.at(-1)?.at ?? null,
  };
}

export function averageSeconds(totalSeconds, count) {
  const sessions = Math.max(0, Math.floor(Number(count) || 0));
  if (sessions === 0) return 0;
  return Math.max(0, Number(totalSeconds) || 0) / sessions;
}

export function shortDate(seconds) {
  const value = Number(seconds);
  if (!Number.isFinite(value) || value <= 0) return '';
  const at = new Date(value * 1000);
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}`;
}

export function monthDay(seconds) {
  const value = Number(seconds);
  if (!Number.isFinite(value) || value <= 0) return '';
  const at = new Date(value * 1000);
  return `${pad(at.getMonth() + 1)}-${pad(at.getDate())}`;
}
