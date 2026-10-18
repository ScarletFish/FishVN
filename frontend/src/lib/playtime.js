const MINUTE = 60;
const HOUR = 60 * MINUTE;

function pad(value) {
  return String(value).padStart(2, '0');
}

export function durationText(t, seconds) {
  const minutes = Math.round(Math.max(0, Number(seconds) || 0) / MINUTE);
  if (minutes <= 0) return '0';
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours <= 0) return `${rest} ${t('common.unitMinute')}`;
  if (rest <= 0) return `${hours} ${t('common.unitHour')}`;
  return `${hours} ${t('common.unitHour')} ${rest} ${t('common.unitMinute')}`;
}

export function compactDuration(minutes) {
  const value = Math.max(0, Math.floor(Number(minutes) || 0));
  const hours = Math.floor(value / 60);
  const rest = value % 60;
  if (hours <= 0) return `${rest}m`;
  return `${hours}h${pad(rest)}m`;
}

export function durationInput(seconds) {
  const total = Math.max(0, Math.floor(Number(seconds) || 0));
  if (total <= 0) return '';
  const rest = total % MINUTE;
  const text = `${Math.floor(total / HOUR)}:${pad(Math.floor((total % HOUR) / MINUTE))}`;
  return rest > 0 ? `${text}:${pad(rest)}` : text;
}

const UNITS = [
  [/^(\d+(?:\.\d+)?)(?:h|hr|hours?|小时|時)/i, HOUR],
  [/^(\d+(?:\.\d+)?)(?:m|min|minutes?|分钟|分)/i, MINUTE],
  [/^(\d+(?:\.\d+)?)(?:s|sec|seconds?|秒)/i, 1],
];

export function parseDuration(text) {
  const raw = String(text ?? '').replace(/\s+/g, '');
  if (!raw) return null;

  const clock = raw.match(/^(\d+):([0-5]?\d)(?::([0-5]?\d))?$/);
  if (clock) {
    return Number(clock[1]) * HOUR + Number(clock[2]) * MINUTE + Number(clock[3] ?? 0);
  }

  if (/^\d+(?:\.\d+)?$/.test(raw)) return Math.round(Number(raw));

  let rest = raw;
  let total = 0;
  while (rest) {
    const unit = UNITS.find(([pattern]) => pattern.test(rest));
    if (!unit) return null;
    const match = rest.match(unit[0]);
    total += Number(match[1]) * unit[1];
    rest = rest.slice(match[0].length);
  }
  return Math.round(total);
}
