
export const items = $state([]);
let counter = 0;

const timers = new Map();

const DURATIONS = {
  success: 4000,
  error: 6000,
  warning: 5000,
  info: 3500,
  silent: 2500,
};

const MAX = 5;

export function toast(msg, type = 'info', opts = {}) {
  const title = typeof msg === 'string' ? msg : (msg?.title ?? '');
  const desc = typeof msg === 'object' && msg?.desc ? msg.desc : '';
  const duration = opts.duration ?? DURATIONS[type] ?? DURATIONS.info;

  const id = ++counter;
  const item = { id, type, title, desc, duration, remaining: duration, dismissing: false };

  items.unshift(item);
  start(item);

  for (const old of items.filter((t) => !t.dismissing).slice(MAX)) dismiss(old.id);

  return id;
}

export function dismiss(id) {
  const item = items.find((t) => t.id === id);
  if (!item || item.dismissing) return;
  item.dismissing = true;
  stop(id);
  setTimeout(() => {
    const at = items.indexOf(item);
    if (at !== -1) items.splice(at, 1);
  }, 300);
}

function start(item) {
  stop(item.id);
  const timer = setTimeout(() => dismiss(item.id), item.remaining);
  timers.set(item.id, { timer, at: Date.now() });
}

export function pause(item) {
  const rec = timers.get(item.id);
  if (!rec) return;
  clearTimeout(rec.timer);
  timers.delete(item.id);
  item.remaining = Math.max(0, item.remaining - (Date.now() - rec.at));
}

export function resume(item) {
  if (!timers.has(item.id) && item.remaining > 0) start(item);
}

function stop(id) {
  const rec = timers.get(id);
  if (!rec) return;
  clearTimeout(rec.timer);
  timers.delete(id);
}
