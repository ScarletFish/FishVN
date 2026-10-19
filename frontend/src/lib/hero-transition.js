let pending = null;

export function captureHero(sourceEl) {
  const gsap = globalThis.gsap;
  if (!gsap) return;
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  const img = sourceEl?.querySelector('.cover-card__media img');
  if (!img || !img.complete || img.naturalWidth === 0) return;
  const box = sourceEl.querySelector('.cover-card__frame') || img;
  const rect = box.getBoundingClientRect();
  if (!rect.width || !rect.height) return;
  pending = {
    src: img.currentSrc || img.src,
    rect: { left: rect.left, top: rect.top, width: rect.width, height: rect.height },
  };
}

export function discardHero() {
  pending = null;
}

function settle(wrap, hero) {
  let done = false;
  let observer = null;
  const timer = setTimeout(finish, 2000);
  function finish() {
    if (done) return;
    done = true;
    observer?.disconnect();
    clearTimeout(timer);
    wrap.style.visibility = '';
    hero.remove();
  }
  function watch(img) {
    if (img.complete && img.naturalWidth > 0) {
      finish();
      return;
    }
    img.addEventListener('load', finish, { once: true });
    img.addEventListener('error', finish, { once: true });
  }
  const img = wrap.querySelector('img');
  if (img) {
    watch(img);
  } else {
    observer = new MutationObserver(() => {
      const el = wrap.querySelector('img');
      if (el) {
        observer.disconnect();
        observer = null;
        watch(el);
      }
    });
    observer.observe(wrap, { childList: true, subtree: true });
  }
}

export function playHero(targetWrap) {
  const data = pending;
  pending = null;
  const gsap = globalThis.gsap;
  if (!data || !targetWrap || !gsap) return;
  targetWrap.style.visibility = 'hidden';
  requestAnimationFrame(() => {
    const toRect = targetWrap.getBoundingClientRect();
    if (!toRect.width || !toRect.height) {
      targetWrap.style.visibility = '';
      return;
    }
    const hero = document.createElement('div');
    hero.style.cssText =
      'position:fixed;' +
      `left:${data.rect.left}px;top:${data.rect.top}px;` +
      `width:${data.rect.width}px;height:${data.rect.height}px;` +
      'z-index:var(--z-modal);pointer-events:none;overflow:hidden;' +
      'border:1px solid var(--border-default);' +
      'border-radius:var(--radius-md);background:var(--bg-surface);' +
      'box-shadow:var(--shadow-md);';
    const clone = document.createElement('img');
    clone.src = data.src;
    clone.alt = '';
    clone.style.cssText = 'width:100%;height:100%;object-fit:cover;display:block;';
    hero.appendChild(clone);
    document.body.appendChild(hero);
    gsap.set(hero, { transformOrigin: '0 0' });
    gsap.to(hero, {
      x: toRect.left - data.rect.left,
      y: toRect.top - data.rect.top,
      scaleX: toRect.width / data.rect.width,
      scaleY: toRect.height / data.rect.height,
      duration: 0.35,
      ease: 'power2.out',
      onComplete: () => settle(targetWrap, hero),
    });
  });
}
