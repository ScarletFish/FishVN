// Toast 状态与调用入口。渲染在 components/Toaster.svelte，挂在 App.svelte 里。
// 用 .svelte.js 后缀是因为要 $state —— .js 里编译不了 rune。

export const items = $state([]);
let counter = 0;

// 计时器簿记用普通 Map：不需要响应式，留在 $state 里会白白触发更新
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

  // 超出上限时挤掉最旧的。必须走退场动画而不是直接移除 ——
  // 正在 dismissing 的不计入，否则叠加上限会被退场中的条目绕过
  for (const old of items.filter((t) => !t.dismissing).slice(MAX)) dismiss(old.id);

  return id;
}

export function dismiss(id) {
  const item = items.find((t) => t.id === id);
  if (!item || item.dismissing) return;
  item.dismissing = true;
  stop(id);
  setTimeout(() => {
    // 退场结束才移除，元素才有动画可播。indexOf 可能是 -1，
    // splice(-1) 会误删最后一条，所以先判存在
    const at = items.indexOf(item);
    if (at !== -1) items.splice(at, 1);
  }, 300);
}

function start(item) {
  stop(item.id);
  const timer = setTimeout(() => dismiss(item.id), item.remaining);
  timers.set(item.id, { timer, at: Date.now() });
}

// 悬停暂停：进度条是 animation，暂停靠 CSS 的 animation-play-state，
// 但计时器在 JS 里，只能手动记下剩余时长
export function pause(item) {
  const rec = timers.get(item.id);
  if (!rec) return;
  clearTimeout(rec.timer);
  timers.delete(item.id);
  item.remaining = Math.max(0, item.remaining - (Date.now() - rec.at));
}

export function resume(item) {
  // !timers.has 防重复启动；remaining > 0 是因为悬停期间可能已耗尽，移开就不该再等
  if (!timers.has(item.id) && item.remaining > 0) start(item);
}

function stop(id) {
  const rec = timers.get(id);
  if (!rec) return;
  clearTimeout(rec.timer);
  timers.delete(id);
}
