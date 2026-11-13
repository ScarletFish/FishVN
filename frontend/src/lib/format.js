export function formatScore(rating) {
  return (rating / 10).toFixed(2);
}

export function formatScoreShort(rating) {
  const text = (rating / 10).toFixed(1);
  return text.endsWith('.0') ? text.slice(0, -2) : text;
}

export function formatDateTime(seconds) {
  const value = Number(seconds);
  if (!Number.isFinite(value) || value <= 0) return '';
  const at = new Date(value * 1000);
  if (Number.isNaN(at.getTime())) return '';
  const pad = (n) => String(n).padStart(2, '0');
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())} ${pad(at.getHours())}:${pad(at.getMinutes())}`;
}

const SIZE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];

export function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes < 0) return '';
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < SIZE_UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const decimals = unit === 1 ? 0 : 1;
  return `${value.toFixed(decimals)} ${SIZE_UNITS[unit]}`;
}
