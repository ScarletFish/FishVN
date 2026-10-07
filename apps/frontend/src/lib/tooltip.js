// 全局提示气泡。替代原生 title。
// 用法：任意元素加 data-tooltip="文本"，hover 显示。
// 副作用模块：import 一次即生效。

const SHOW_DELAY = 500;
const HIDE_DELAY = 80;

const el = document.createElement('div');
el.id = 'globalTooltip';
document.body.appendChild(el);

let showTimer = null;
let hideTimer = null;
let current = null;

document.addEventListener('mouseover', (e) => {
  const target = e.target.closest('[data-tooltip]');
  if (!target || target === current) return;

  clearTimeout(showTimer);
  clearTimeout(hideTimer);
  current = target;

  // 记录进入时的光标位置，不跟随移动
  const entryX = e.clientX;
  const entryY = e.clientY;

  showTimer = setTimeout(() => {
    el.classList.toggle('is-rich', target.hasAttribute('data-tooltip-rich'));
    el.textContent = target.getAttribute('data-tooltip');
    position(entryX, entryY);
    el.classList.remove('is-exiting');
    el.classList.add('is-visible');
  }, SHOW_DELAY);
});

document.addEventListener('mouseout', (e) => {
  const target = e.target.closest('[data-tooltip]');
  if (!target) return;
  if (target !== current && !target.contains(current)) return;

  clearTimeout(showTimer);
  current = null;
  hideTimer = setTimeout(() => {
    el.classList.add('is-exiting');
    el.classList.remove('is-visible');
  }, HIDE_DELAY);
});

document.addEventListener(
  'scroll',
  () => {
    if (!el.classList.contains('is-visible')) return;
    clearTimeout(showTimer);
    clearTimeout(hideTimer);
    current = null;
    el.classList.add('is-exiting');
    el.classList.remove('is-visible');
  },
  true,
);

function position(cx, cy) {
  const tw = el.offsetWidth || 0;
  const th = el.offsetHeight || 30;
  const gap = 10;

  let left = cx + gap;
  let top = cy + gap + 4;

  if (tw && left + tw > window.innerWidth - 6) left = cx - tw - gap;
  if (top + th > window.innerHeight - 6) top = cy - th - gap;

  el.style.left = Math.max(6, left) + 'px';
  el.style.top = Math.max(6, top) + 'px';
}