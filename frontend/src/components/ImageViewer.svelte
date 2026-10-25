<script>
  import {
    VIEWER_DEFAULT_MAX_ZOOM,
    fitView,
    hundredView,
    imageToScreen,
    isFitView,
    panView,
    screenToImage,
    zoomAtView,
  } from '../lib/image-viewer.js';

  let {
    src = '',
    alt = '',
    maxZoom = VIEWER_DEFAULT_MAX_ZOOM,
    onready,
    ontap,
    children,
  } = $props();

  let stageEl = $state(null);
  let imgEl = $state(null);
  let iw = $state(0);
  let ih = $state(0);
  let view = $state({ scale: 1, tx: 0, ty: 0 });
  let fitMode = $state(true);
  let pan = null;
  let moved = false;

  const zoomPct = $derived(Math.round(view.scale * 100));

  export function resetView() {
    fitMode = true;
    refit();
  }

  function refit() {
    const stage = stageEl;
    if (!stage || !iw || !ih) return;
    view = fitView(iw, ih, stage.clientWidth, stage.clientHeight);
  }

  $effect(() => {
    const stage = stageEl;
    if (!stage || !iw || !ih) return;
    if (fitMode) refit();
    const observer = new ResizeObserver(() => {
      if (fitMode) refit();
    });
    observer.observe(stage);
    return () => observer.disconnect();
  });

  $effect(() => {
    const stage = stageEl;
    if (!stage) return;
    const onWheel = (e) => {
      if (!iw || !ih) return;
      e.preventDefault();
      const rect = stage.getBoundingClientRect();
      const factor = Math.exp(-e.deltaY * 0.0015);
      view = zoomAtView(
        view,
        factor,
        e.clientX - rect.left,
        e.clientY - rect.top,
        undefined,
        maxZoom
      );
      fitMode = false;
    };
    stage.addEventListener('wheel', onWheel, { passive: false });
    return () => stage.removeEventListener('wheel', onWheel);
  });

  function onImgLoad() {
    iw = imgEl?.naturalWidth ?? 0;
    ih = imgEl?.naturalHeight ?? 0;
    if (iw && ih) onready?.(iw, ih);
  }

  function onDown(e) {
    if (!iw || !ih) return;
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    pan = { id: e.pointerId, x: e.clientX, y: e.clientY };
    moved = false;
  }

  function onMove(e) {
    if (!pan || e.pointerId !== pan.id) return;
    const dx = e.clientX - pan.x;
    const dy = e.clientY - pan.y;
    if (!moved) {
      if (Math.hypot(dx, dy) < 4) return;
      moved = true;
    }
    pan = { id: e.pointerId, x: e.clientX, y: e.clientY };
    view = panView(view, dx, dy);
    fitMode = false;
  }

  function onUp(e) {
    if (!pan || e.pointerId !== pan.id) return;
    if (!moved) {
      const p = toImage(e.clientX, e.clientY);
      ontap?.({ x: p.x, y: p.y });
    }
    pan = null;
  }

  function onDblClick(e) {
    if (!iw || !ih) return;
    const stage = stageEl;
    if (!stage) return;
    const rect = stage.getBoundingClientRect();
    if (isFitView(view, iw, ih, stage.clientWidth, stage.clientHeight)) {
      view = hundredView(iw, ih, e.clientX - rect.left, e.clientY - rect.top);
      fitMode = false;
    } else {
      resetView();
    }
  }

  function toImage(clientX, clientY) {
    const stage = stageEl;
    if (!stage) return { x: 0, y: 0 };
    const rect = stage.getBoundingClientRect();
    return screenToImage(view, clientX - rect.left, clientY - rect.top);
  }

  function toScreen(ix, iy) {
    return imageToScreen(view, ix, iy);
  }
</script>

<div
  class="viewer-stage"
  bind:this={stageEl}
  onpointerdown={onDown}
  onpointermove={onMove}
  onpointerup={onUp}
  onpointercancel={onUp}
  ondblclick={onDblClick}
  role="application"
  aria-label={alt || '图片查看器'}
>
  {#if src}
    <img
      bind:this={imgEl}
      {src}
      {alt}
      draggable="false"
      onload={onImgLoad}
      style:width={iw ? `${iw}px` : undefined}
      style:height={ih ? `${ih}px` : undefined}
      style:transform={`translate(${view.tx}px, ${view.ty}px) scale(${view.scale})`}
    />
  {/if}
  {@render children?.({ toImage, toScreen, view })}
  {#if iw && ih}
    <span class="viewer-zoom">{zoomPct}%</span>
  {/if}
</div>

<style>
  .viewer-stage {
    position: relative;
    flex: 1;
    min-width: 0;
    min-height: 0;
    height: 100%;
    overflow: hidden;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
    cursor: grab;
  }

  .viewer-stage:active {
    cursor: grabbing;
  }

  .viewer-stage img {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: 0 0;
    pointer-events: none;
  }

  .viewer-zoom {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
    padding: 2px var(--space-2);
    background: var(--bg-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: 11px;
    color: var(--fg-muted);
    pointer-events: none;
  }
</style>
