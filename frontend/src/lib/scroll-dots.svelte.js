// 水平滚动分页圆点指示器
// 布局模型：容器上写 --card-w（基准卡宽）与 --cols（整数列数，本模块按
// 基准 + 容器宽 纯函数计算并写回），卡片用 calc 弹性均分，每屏恰好整数张、
// 铺满无空隙。不量测任何渲染尺寸、无时序耦合：隐藏/显示的先后顺序不影响结果，
// 窗口尺寸一变（ResizeObserver）即重算。

/**
 * 纯函数：按基准卡宽 + 容器宽算整数列数。与容器上 --cols 的 calc 均分契约同源。
 * @param {HTMLElement} el - 横向滚动容器
 * @param {{baselineW?: number, gap?: number}} [opts] - baselineW 取容器 --card-w，gap 为卡间距
 * @returns {number} 整数列数
 */
export function computeColsFor(el, { baselineW = 320, gap } = {}) {
  const cs = getComputedStyle(el);
  const g = gap ?? (parseFloat(cs.gap) || parseFloat(cs.columnGap) || 12);
  const padX = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
  const contentW = Math.max(0, el.clientWidth - padX);
  if (contentW <= 0) return parseInt(el.style.getPropertyValue('--cols'), 10) || 1;
  return Math.max(1, Math.floor((contentW + g) / (baselineW + g)));
}

/** 每张卡的渲染宽按列数均分容器，滚动步长再加一个卡间距 */
export function cardStep(contentW, cols, gap) {
  return (contentW - gap * (cols - 1)) / cols + gap;
}

/**
 * 为水平滚动容器初始化分页圆点。
 * @param {object} opts
 * @param {HTMLElement} opts.scroll - 水平滚动容器
 * @param {number} opts.total - 卡片总数
 * @param {HTMLElement} opts.dotsParent - 放圆点的父元素（section 头部）
 * @param {number} [opts.initialIndex] - 初始滚动到的索引
 * @returns {() => void} 解绑函数（移除 scroll/resize 监听与圆点节点）
 */
export function initScrollDots({ scroll, total, dotsParent, initialIndex }) {
  if (!scroll || !dotsParent || total === 0) return () => {};

  const getBaselineW = () => parseFloat(getComputedStyle(scroll).getPropertyValue('--card-w')) || 320;
  const getGap = () => {
    const cs = getComputedStyle(scroll);
    return parseFloat(cs.gap) || parseFloat(cs.columnGap) || 12;
  };
  const getContentW = () => {
    const cs = getComputedStyle(scroll);
    const padX = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
    return Math.max(0, scroll.clientWidth - padX);
  };
  const computeCols = () => {
    if (getContentW() <= 0) return parseInt(scroll.style.getPropertyValue('--cols'), 10) || 1;
    const cols = computeColsFor(scroll, { baselineW: getBaselineW(), gap: getGap() });
    const prev = scroll.style.getPropertyValue('--cols');
    if (String(cols) !== prev) scroll.style.setProperty('--cols', String(cols));
    return cols;
  };
  let visible = computeCols();
  const getCardStep = () => cardStep(getContentW(), visible, getGap());

  let dotsEl = dotsParent.querySelector('.scroll-dots');
  if (!dotsEl) {
    dotsEl = document.createElement('div');
    dotsEl.className = 'scroll-dots';
    dotsParent.appendChild(dotsEl);
  }

  let dotCount = 0;
  const rebuildDots = () => {
    const newCount = Math.max(1, total - visible + 1);
    if (newCount === dotCount) return;
    dotCount = newCount;
    if (dotCount <= 1) {
      dotsEl.innerHTML = '';
      return;
    }
    dotsEl.innerHTML = Array.from({ length: dotCount }, (_, i) =>
      `<button type="button" class="scroll-dot${i === 0 ? ' active' : ''}" data-index="${i}" aria-label="第 ${i + 1} 页"></button>`
    ).join('');
  };
  rebuildDots();

  const setActive = (idx) => {
    dotsEl.querySelectorAll('.scroll-dot').forEach((d, i) => d.classList.toggle('active', i === idx));
  };

  let ticking = false;
  const updateActiveDot = () => {
    const cols = computeCols();
    if (cols !== visible) {
      visible = cols;
      rebuildDots();
    }
    if (dotCount <= 1) return;
    const step = getCardStep();
    if (!step) return;
    setActive(Math.max(0, Math.min(dotCount - 1, Math.round(scroll.scrollLeft / step))));
  };

  const onScroll = () => {
    if (ticking) return;
    requestAnimationFrame(() => {
      updateActiveDot();
      ticking = false;
    });
    ticking = true;
  };
  const onClick = (e) => {
    const dot = e.target.closest('.scroll-dot');
    if (!dot) return;
    const idx = parseInt(dot.dataset.index, 10);
    scroll.scrollTo({ left: idx * getCardStep(), behavior: 'smooth' });
    setActive(idx);
  };
  const ro = new ResizeObserver(() => requestAnimationFrame(updateActiveDot));
  ro.observe(scroll);

  scroll.addEventListener('scroll', onScroll, { passive: true });
  dotsEl.addEventListener('click', onClick);
  if (initialIndex != null) scroll.scrollLeft = Math.max(0, initialIndex * getCardStep());
  requestAnimationFrame(updateActiveDot);

  return () => {
    scroll.removeEventListener('scroll', onScroll);
    dotsEl.removeEventListener('click', onClick);
    ro.disconnect();
    dotsEl.remove();
  };
}