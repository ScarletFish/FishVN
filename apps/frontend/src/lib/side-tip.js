// 侧栏提示气泡。锚定按钮右缘，带指向箭头。
// 用法：<button data-tip="库" use:sideTip>
// 与 lib/tooltip.js 的全局气泡并存：定位与外观不同，互不干扰。

const SHOW_DELAY = 400;
const HIDE_DELAY = 120;

export function sideTip(node) {
  const label = node.getAttribute('data-tip');
  if (!label) return;

  const tip = document.createElement('div');
  tip.className = 'sidebar-tooltip';

  const text = document.createElement('div');
  text.className = 'sidebar-tooltip-text';
  text.textContent = label;

  const arrow = document.createElement('div');
  arrow.className = 'sidebar-tooltip-arrow';

  tip.append(text, arrow);
  document.body.appendChild(tip);

  let showTimer = null;
  let hideTimer = null;

  const cancelShow = () => {
    if (showTimer) {
      clearTimeout(showTimer);
      showTimer = null;
    }
  };

  const cancelHide = () => {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  };

  const show = () => {
    const box = node.getBoundingClientRect();
    tip.style.top = box.top + box.height / 2 + 'px';
    tip.classList.add('is-visible');
  };

  const hide = () => {
    cancelShow();
    hideTimer = setTimeout(() => tip.classList.remove('is-visible'), HIDE_DELAY);
  };

  const onEnter = () => {
    cancelHide();
    cancelShow();
    showTimer = setTimeout(show, SHOW_DELAY);
  };

  node.addEventListener('mouseenter', onEnter);
  node.addEventListener('mouseleave', hide);

  return {
    destroy() {
      cancelShow();
      cancelHide();
      node.removeEventListener('mouseenter', onEnter);
      node.removeEventListener('mouseleave', hide);
      tip.remove();
    },
  };
}