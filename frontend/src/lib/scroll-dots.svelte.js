
export function computeColsFor(el, { baselineW = 320, gap } = {}) {
  const cs = getComputedStyle(el);
  const g = gap ?? (parseFloat(cs.gap) || parseFloat(cs.columnGap) || 12);
  const padX = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
  const contentW = Math.max(0, el.clientWidth - padX);
  if (contentW <= 0) return parseInt(el.style.getPropertyValue('--cols'), 10) || 1;
  return Math.max(1, Math.floor((contentW + g) / (baselineW + g)));
}

export function cardStep(contentW, cols, gap) {
  return (contentW - gap * (cols - 1)) / cols + gap;
}

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