import { OverlayScrollbars } from 'overlayscrollbars';

// 滚动条默认隐藏；滚动或指针移入内容区时淡入。
// autoHideDelay 是最后一次滚动/移入到开始淡出的等待；淡出时长在 CSS 里（--duration-normal）。
// theme 只作为库的结构钩子，尺寸与色值由 components/scrollbars.css 全量覆盖。
const OPTIONS = {
  scrollbars: {
    theme: 'os-theme-light',
    autoHide: 'move',
    autoHideDelay: 800,
  },
};

/** 把一个滚动容器交给 overlayscrollbars */
export function useScrollbars(target) {
  if (!target) return;
  OverlayScrollbars(target, OPTIONS);
  watchRightEdge(target);
}

/**
 * 指针进入右侧竖带时挂 is-edge，CSS 据此加粗手柄。拖拽期间不看指针位置。
 *
 * 命中判定按坐标做，不走滚动条的 :hover —— 库给滚动条整体设了 pointer-events: none，
 * 放开命中后落在带上的滚轮会进库自己的 wheel 监听并被 preventDefault，竖带上就滚不动。
 * 判定线取竖带自身的左边界，竖带宽度变了不用改这里。
 */
function watchRightEdge(target) {
  let scrollbar = null;
  let edge = 0;
  let inside = false;
  let dragging = false;

  const sync = () => {
    scrollbar = target.querySelector('.os-scrollbar-vertical');
    edge = scrollbar ? scrollbar.getBoundingClientRect().left : 0;
  };

  const apply = () => {
    scrollbar?.classList.toggle('is-edge', inside || dragging);
  };

  target.addEventListener('pointermove', (e) => {
    if (!scrollbar) sync();
    const next = e.clientX >= edge;
    if (next === inside) return;
    inside = next;
    apply();
  });

  target.addEventListener('pointerleave', () => {
    if (!inside) return;
    inside = false;
    apply();
  });

  // 竖带上只有手柄可命中，别处的按下是穿透到内容的，不算拖拽
  target.addEventListener('pointerdown', (e) => {
    if (e.target.closest('.os-scrollbar-handle')) {
      dragging = true;
      apply();
    }
  });

  // 抬手可能落在容器外
  const release = () => {
    if (!dragging) return;
    dragging = false;
    apply();
  };
  window.addEventListener('pointerup', release);
  window.addEventListener('pointercancel', release);

  window.addEventListener('resize', () => {
    sync();
    inside = false;
    apply();
  });
}
