// 渲染在 components/Toaster.svelte，挂在 App.svelte 里。
// 用 .svelte.js 后缀：要 $state，.js 里编译不了 rune。

export const items = $state([]);
let counter = 0;

// 计时器簿记不要放 $state，否则每次读写都触发更新
const timers = new Map();

const DURATIONS = {
  success: 4000,
  error: 6000,
  warning: 5000,
  info: 3500,
  silent: 2500,
};

const MAX = 5;

/**
 * 弹一条通知。
 * @param {string|{title: string, desc?: string}} msg 标题，或标题+说明两行
 * @param {'success'|'error'|'warning'|'info'|'silent'} [type]
 * @param {{duration?: number}} [opts] 覆盖按类型取的停留时长
 * @returns {number} 这条通知的 id，可传给 dismiss
 */
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

// 暂停要手动记剩余时长：进度条的暂停靠 CSS animation-play-state，
// 计时器在 JS 里，两者不通
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
