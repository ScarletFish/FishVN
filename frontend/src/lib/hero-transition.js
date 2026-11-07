import { shouldReduceMotion } from './motion.js';

let pending = null;

function bgSrc(el) {
  const bg = el && getComputedStyle(el).backgroundImage;
  const m = bg && bg.match(/url\(["']?([^"')]+)["']?\)/);
  return m ? m[1] : null;
}

export function captureHero(sourceEl, opts = {}) {
  const gsap = globalThis.gsap;
  if (!gsap) return;
  if (shouldReduceMotion()) return;
  const img = sourceEl?.querySelector(opts.img ?? '.cover-card__media img');
  if (img && (!img.complete || img.naturalWidth === 0)) return;
  const bgEl = img ? null : sourceEl?.querySelector(opts.bg ?? '.continue-bg');
  const src = img ? img.currentSrc || img.src : bgSrc(bgEl);
  if (!src) return;
  const frameEl =
    opts.frame === 'self'
      ? sourceEl
      : opts.frame
        ? sourceEl.querySelector(opts.frame)
        : sourceEl.querySelector('.cover-card__frame');
  const box = frameEl || img || bgEl;
  if (!box) return;
  let { left, top, width, height } = box.getBoundingClientRect();
  if (!width || !height) return;
  if (opts.aspect) {
    const w = Math.min(width, height * opts.aspect);
    const h = w / opts.aspect;
    left += (width - w) / 2;
    top += (height - h) / 2;
    width = w;
    height = h;
  }
  pending = {
    src,
    rect: { left, top, width, height },
    objectPosition: img ? getComputedStyle(img).objectPosition : '',
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
    clone.style.cssText =
      'width:100%;height:100%;object-fit:cover;display:block;' +
      (data.objectPosition ? `object-position:${data.objectPosition};` : '');
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
