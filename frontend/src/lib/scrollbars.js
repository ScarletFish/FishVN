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

// 只记主视图：MyList 靠 getScrollViewport 还原列表滚动位置，弹窗实例写进来会把它顶掉
let mainInstance = null;

export function useScrollbars(target) {
  if (!target) return;
  mainInstance = OverlayScrollbars(target, OPTIONS);
  watchRightEdge(target, mainInstance);
}

export function getScrollViewport() {
  return mainInstance?.elements().viewport;
}

// 弹窗表单的滚动容器：不挂 watchRightEdge —— 那套 edge/inside 坐标判定
// 在弹窗里会把滚动条卡在加粗态回不去，加粗反馈只保留在主视图。
export function attachScrollbars(target) {
  if (!target) return null;
  return OverlayScrollbars(target, OPTIONS);
}

export function destroyScrollbars(inst) {
  if (!inst) return;
  try {
    inst.destroy();
  } catch {
    /* 已销毁则忽略 */
  }
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
      // 拖拽中移出不进缓冲
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

  // 指针离开整个窗口：立即收起并清状态。抬手发生在窗口外时
  // pointerup 收不到，不清 dragging 会卡在加粗状态
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

  // 先算一次竖带位置，避免首次移动指针时 edge=0 导致到处都判定为「在边缘」
  sync();
}