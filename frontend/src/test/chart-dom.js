import { flushSync, mount, unmount } from 'svelte';

const SIZE = {
  x: 0,
  y: 0,
  width: 700,
  height: 220,
  top: 0,
  left: 0,
  right: 700,
  bottom: 220,
  toJSON() {},
};

function waitForFirstPath(target, timeout = 2000) {
  if (target.querySelector('svg path')) return Promise.resolve();
  return new Promise((resolve) => {
    const observer = new MutationObserver(() => {
      if (target.querySelector('svg path')) finish();
    });
    const timer = setTimeout(finish, timeout);
    function finish() {
      observer.disconnect();
      clearTimeout(timer);
      resolve();
    }
    observer.observe(target, { childList: true, subtree: true });
  });
}

export async function mountChart(Component, props = {}) {
  const saved = {
    resizeObserver: globalThis.ResizeObserver,
    rect: Element.prototype.getBoundingClientRect,
    bbox: SVGElement.prototype.getBBox,
    textLength: SVGElement.prototype.getComputedTextLength,
    transform: Object.getOwnPropertyDescriptor(SVGElement.prototype, 'transform'),
    raf: globalThis.requestAnimationFrame,
    caf: globalThis.cancelAnimationFrame,
  };

  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
  globalThis.requestAnimationFrame ??= (cb) => setTimeout(cb, 0);
  globalThis.cancelAnimationFrame ??= clearTimeout;
  Element.prototype.getBoundingClientRect = () => ({ ...SIZE });
  SVGElement.prototype.getBBox = () => ({ x: 0, y: 0, width: 40, height: 14 });
  SVGElement.prototype.getComputedTextLength = () => 40;
  Object.defineProperty(SVGElement.prototype, 'transform', {
    configurable: true,
    get() {
      return { baseVal: { consolidate: () => null, numberOfItems: 0 } };
    },
  });

  const target = document.createElement('div');
  document.body.appendChild(target);
  const app = mount(Component, { target, props });
  flushSync();
  await waitForFirstPath(target);
  flushSync();

  return {
    target,
    cleanup() {
      unmount(app);
      target.remove();
      globalThis.ResizeObserver = saved.resizeObserver;
      Element.prototype.getBoundingClientRect = saved.rect;
      SVGElement.prototype.getBBox = saved.bbox;
      SVGElement.prototype.getComputedTextLength = saved.textLength;
      if (saved.transform) {
        Object.defineProperty(SVGElement.prototype, 'transform', saved.transform);
      }
      globalThis.requestAnimationFrame = saved.raf;
      globalThis.cancelAnimationFrame = saved.caf;
    },
  };
}
