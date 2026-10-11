import { OverlayScrollbars, ClickScrollPlugin } from 'overlayscrollbars';

// clickScroll 不显式注册插件就静默失效
OverlayScrollbars.plugin(ClickScrollPlugin);

// autoHide: 'never' —— 库的计时器会盖掉竖带命中时的常驻。
// 淡出时长在 CSS 里（--duration-normal），theme 只作结构钩子，
// 尺寸与色值由 components/scrollbars.css 全量覆盖。
const OPTIONS = {
  scrollbars: {
    theme: 'os-theme-light',
    autoHide: 'never',
    clickScroll: true,
  },
};

const HIDE_DELAY = 800;
const LINGER_DELAY = 1000;
const EDGE_TOLERANCE = 12;

let instance = null;

export function useScrollbars(target) {
  if (!target) return;
  instance = OverlayScrollbars(target, OPTIONS);
  watchRightEdge(target, instance);
}

export function getScrollViewport() {
  return instance?.elements().viewport;
}

/**
 * 命中判定必须按坐标做，不能走滚动条的 :hover —— 库给滚动条整体设了
 * pointer-events: none，放开命中后落在带上的滚轮会进库自己的 wheel 监听
 * 并被 preventDefault，竖带上就滚不动。
 */
function watchRightEdge(target, instance) {
  let scrollbar = null;
  let edge = 0;
  let inside = false;
  let dragging = false;
  let lingering = false;
  let hideTimer = null;
  let lingerTimer = null;

  const engaged = () => inside || dragging || lingering;

  // 取竖带所在的最右侧一段：竖带右缘到容器右缘之间还有死区，一并算进来
  const sync = () => {
    scrollbar = target.querySelector('.os-scrollbar-vertical');
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
    // 不调 apply()：收细那一下会读成「先缩一下再消失」的两步动作
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

  // 库接管后 scroll 事件在内部 viewport 上，target 上收不到
  const viewport = instance.elements().viewport;
  viewport.addEventListener('scroll', () => {
    apply();
    show();
    scheduleHide();
  }, { passive: true });

  target.addEventListener('pointermove', (e) => {
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
      // 拖拽中移出不进缓冲
      if (dragging) hide();
      else scheduleLinger();
    }
  });

  target.addEventListener('pointerleave', () => {
    if (!inside) return;
    inside = false;
    apply();
    scheduleLinger();
  });

  // 竖带上只有手柄可命中，别处的按下是穿透到内容的，不算拖拽
  target.addEventListener('pointerdown', (e) => {
    if (e.target.closest('.os-scrollbar-handle')) {
      dragging = true;
      apply();
      show();
    }
  });

  // 指针离开整个窗口：立即收起并清状态。抬手发生在窗口外时
  // pointerup 收不到，不清 dragging 会卡在加粗状态
  document.addEventListener('pointerleave', () => {
    inside = false;
    dragging = false;
    cancelLinger();
    apply();
    hide();
  });

  // 抬手可能落在容器外
  const release = () => {
    if (!dragging) return;
    dragging = false;
    apply();
    if (inside) show();
    else scheduleLinger();
  };
  window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', release);

  window.addEventListener('resize', () => {
    sync();
    inside = false;
    cancelLinger();
    apply();
  });
}