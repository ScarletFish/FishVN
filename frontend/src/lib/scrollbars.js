import { OverlayScrollbars, ClickScrollPlugin } from 'overlayscrollbars';

OverlayScrollbars.plugin(ClickScrollPlugin);

const MAIN_OPTIONS = {
  scrollbars: {
    theme: 'os-theme-light',
    autoHide: 'never',
    clickScroll: true,
  },
};

const PANEL_OPTIONS = {
  scrollbars: {
    theme: 'os-theme-light',
    autoHide: 'leave',
    autoHideDelay: 400,
    clickScroll: true,
  },
};

const HIDE_DELAY = 800;
const LINGER_DELAY = 1000;
const EDGE_TOLERANCE = 12;

let mainInstance = null;

export function useScrollbars(target) {
  if (!target) return;
  mainInstance = OverlayScrollbars(target, MAIN_OPTIONS);
  const { scrollbarHorizontal, scrollbarVertical } = mainInstance.elements();
  scrollbarHorizontal.scrollbar.classList.add('is-main');
  scrollbarVertical.scrollbar.classList.add('is-main');
  watchRightEdge(target, mainInstance);
}

export function getScrollViewport() {
  return mainInstance?.elements().viewport;
}

export function panelScrollbars(node) {
  const inst = OverlayScrollbars(node, PANEL_OPTIONS);
  return {
    destroy() {
      try {
        inst.destroy();
      } catch {
      }
    },
  };
}

function watchRightEdge(target, instance) {
  let scrollbar = null;
  let edge = 0;
  let inside = false;
  let dragging = false;
  let lingering = false;
  let hideTimer = null;
  let lingerTimer = null;

  const engaged = () => inside || dragging || lingering;

  const sync = () => {
    scrollbar = instance.elements().scrollbarVertical.scrollbar;
    const right = target.getBoundingClientRect().right;
    edge = scrollbar ? Math.min(scrollbar.getBoundingClientRect().left, right - EDGE_TOLERANCE) : 0;
  };

  const show = () => {
    clearTimeout(hideTimer);
    hideTimer = null;
    scrollbar?.classList.add('is-visible');
  };

  const hide = () => {
    clearTimeout(hideTimer);
    clearTimeout(lingerTimer);
    hideTimer = null;
    lingerTimer = null;
    scrollbar?.classList.remove('is-visible');
  };

  const apply = () => {
    scrollbar?.classList.toggle('is-edge', engaged());
  };

  const scheduleHide = () => {
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (engaged()) return;
      hide();
    }, HIDE_DELAY);
  };

  const scheduleLinger = () => {
    clearTimeout(lingerTimer);
    lingering = true;
    apply();
    show();
    lingerTimer = setTimeout(() => {
      lingering = false;
      hide();
    }, LINGER_DELAY);
  };

  const cancelLinger = () => {
    clearTimeout(lingerTimer);
    lingerTimer = null;
    lingering = false;
  };

  const onScroll = () => {
    apply();
    show();
    scheduleHide();
  };

  const onPointerMove = (e) => {
    if (!scrollbar) sync();
    const next = e.clientX >= edge;
    if (next === inside) return;
    inside = next;
    if (inside) {
      cancelLinger();
      apply();
      show();
    } else {
      apply();
      if (dragging) hide();
      else scheduleLinger();
    }
  };

  const onPointerLeaveTarget = () => {
    if (!inside) return;
    inside = false;
    apply();
    scheduleLinger();
  };

  const onPointerDown = (e) => {
    if (e.target.closest('.os-scrollbar-handle')) {
      dragging = true;
      apply();
      show();
    }
  };

  const onDocLeave = () => {
    inside = false;
    dragging = false;
    cancelLinger();
    apply();
    hide();
  };

  const release = () => {
    if (!dragging) return;
    dragging = false;
    apply();
    if (inside) show();
    else scheduleLinger();
  };

  const onResize = () => {
    sync();
    inside = false;
    cancelLinger();
    apply();
  };

  const viewport = instance.elements().viewport;
  viewport.addEventListener('scroll', onScroll, { passive: true });
  target.addEventListener('pointermove', onPointerMove);
  target.addEventListener('pointerleave', onPointerLeaveTarget);
  target.addEventListener('pointerdown', onPointerDown);
  document.addEventListener('pointerleave', onDocLeave);
  window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', release);
  window.addEventListener('resize', onResize);

  sync();
}