<script>
  import Modal from '../Modal.svelte';
  import ImageViewer from '../ImageViewer.svelte';
  import { t } from '../../lib/i18n.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import {
    COVER_ASPECT,
    clampFocal,
    focalFromPointer,
    focalToFrame,
    frameSize,
    frameToFocal,
    grabOffset,
  } from '../../lib/cover-focal.js';

  let {
    open = $bindable(false),
    imagePath = '',
    focusX = 50,
    focusY = 50,
    oncommit,
    onclose,
  } = $props();

  let rootEl = $state(null);
  let viewer = $state(null);
  let url = $state(null);
  let broken = $state(false);
  let iw = $state(0);
  let ih = $state(0);
  let draftX = $state(50);
  let draftY = $state(50);
  let frameDrag = null;

  const hasImage = $derived(Boolean(imagePath));
  const validX = $derived(Boolean(iw && ih && frameSize(iw, ih, COVER_ASPECT).w < iw));
  const validY = $derived(Boolean(iw && ih && frameSize(iw, ih, COVER_ASPECT).h < ih));

  $effect(() => {
    if (!open) return;
    draftX = clampFocal(focusX);
    draftY = clampFocal(focusY);
    broken = false;
    if (!imagePath) {
      url = null;
      return;
    }
    let live = true;
    coverSrc(imagePath)
      .then((u) => {
        if (live) url = u;
      })
      .catch(() => {
        if (live) broken = true;
      });
    return () => {
      live = false;
    };
  });

  $effect(() => {
    if (!open) return;
    const raf = requestAnimationFrame(() => rootEl?.focus());
    return () => cancelAnimationFrame(raf);
  });

  function onReady(w, h) {
    iw = w;
    ih = h;
  }

  function onTap(p) {
    if (!iw || !ih) return;
    const size = frameSize(iw, ih, COVER_ASPECT);
    const next = frameToFocal(p.x - size.w / 2, p.y - size.h / 2, iw, ih, COVER_ASPECT);
    draftX = next.x;
    draftY = next.y;
  }

  function onFrameDown(e, toImage) {
    if (!iw || !ih) return;
    e.stopPropagation();
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    const p = toImage(e.clientX, e.clientY);
    const size = frameSize(iw, ih, COVER_ASPECT);
    const f = focalToFrame(draftX, draftY, iw, ih, COVER_ASPECT);
    frameDrag = {
      id: e.pointerId,
      toImage,
      grab: grabOffset(p.x, p.y, f.fx, f.fy, size.w, size.h, true),
    };
  }

  function onFrameMove(e, toImage) {
    if (!frameDrag || e.pointerId !== frameDrag.id) return;
    e.stopPropagation();
    const p = toImage(e.clientX, e.clientY);
    const next = focalFromPointer(p.x, p.y, frameDrag.grab, iw, ih, COVER_ASPECT);
    draftX = next.x;
    draftY = next.y;
  }

  function onFrameUp(e) {
    if (frameDrag && e.pointerId === frameDrag.id) frameDrag = null;
  }

  function onKeyDown(e) {
    const step = e.shiftKey ? 10 : 2;
    let handled = true;
    if (e.key === 'ArrowLeft' && validX) draftX = clampFocal(draftX - step);
    else if (e.key === 'ArrowRight' && validX) draftX = clampFocal(draftX + step);
    else if (e.key === 'ArrowUp' && validY) draftY = clampFocal(draftY - step);
    else if (e.key === 'ArrowDown' && validY) draftY = clampFocal(draftY + step);
    else handled = false;
    if (handled) e.preventDefault();
  }

  function quick(kind) {
    if (kind === 'center') {
      draftX = 50;
      draftY = 50;
    } else if (kind === 'top') draftY = 0;
    else if (kind === 'bottom') draftY = 100;
    else if (kind === 'left') draftX = 0;
    else if (kind === 'right') draftX = 100;
  }

  function confirm() {
    oncommit?.(draftX, draftY);
    onclose?.();
  }
</script>

<div
  class="focal-root"
  bind:this={rootEl}
  role="slider"
  aria-label={t('gameForm.coverFocal.title')}
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={Math.round(draftX)}
  aria-valuetext="{Math.round(draftX)}%, {Math.round(draftY)}%"
  tabindex="0"
  onkeydown={onKeyDown}
>
  <Modal {open} variant="modal--focal" title={t('gameForm.coverFocal.title')} onclose={() => onclose?.()}>
    <div class="focal-body">
      <ImageViewer
        bind:this={viewer}
        src={url && !broken ? url : ''}
        alt=""
        onready={onReady}
        ontap={onTap}
      >
        {#snippet children({ toImage, toScreen, view })}
          {#if iw && ih}
            {@const size = frameSize(iw, ih, COVER_ASPECT)}
            {@const f = focalToFrame(draftX, draftY, iw, ih, COVER_ASPECT)}
            {@const tl = toScreen(f.fx, f.fy)}
            {@const w = size.w * view.scale}
            {@const h = size.h * view.scale}
            <div
              class="focal-mask"
              style:left="{tl.x}px"
              style:top="{tl.y}px"
              style:width="{w}px"
              style:height="{h}px"
            ></div>
            <div
              class="focal-frame"
              role="application"
              aria-label={t('gameForm.coverFocal.title')}
              style:left="{tl.x}px"
              style:top="{tl.y}px"
              style:width="{w}px"
              style:height="{h}px"
              onpointerdown={(e) => onFrameDown(e, toImage)}
              onpointermove={(e) => onFrameMove(e, toImage)}
              onpointerup={onFrameUp}
              onpointercancel={onFrameUp}
            >
              <span class="focal-frame__tag">3:4</span>
            </div>
          {/if}
        {/snippet}
      </ImageViewer>
    </div>
    <div class="focal-quick">
      <button type="button" class="btn" onclick={() => quick('center')} disabled={!hasImage}>
        {t('gameForm.coverFocal.center')}
      </button>
      <button type="button" class="btn" onclick={() => quick('left')} disabled={!hasImage || !validX}>
        {t('gameForm.coverFocal.left')}
      </button>
      <button type="button" class="btn" onclick={() => quick('right')} disabled={!hasImage || !validX}>
        {t('gameForm.coverFocal.right')}
      </button>
      <button type="button" class="btn" onclick={() => quick('top')} disabled={!hasImage || !validY}>
        {t('gameForm.coverFocal.top')}
      </button>
      <button type="button" class="btn" onclick={() => quick('bottom')} disabled={!hasImage || !validY}>
        {t('gameForm.coverFocal.bottom')}
      </button>
      <button type="button" class="btn" onclick={() => viewer?.resetView()} disabled={!hasImage}>
        {t('gameForm.coverFocal.fitWindow')}
      </button>
    </div>
    <div class="modal-actions">
      <button type="button" class="btn" onclick={() => onclose?.()}>
        {t('common.cancel')}
      </button>
      <button type="button" class="btn btn-primary" onclick={confirm} disabled={!hasImage}>
        {t('gameForm.coverFocal.confirm')}
      </button>
    </div>
  </Modal>
</div>

<style>
  .focal-root {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    outline: none;
  }

  .focal-body {
    display: flex;
    flex: 1 1 auto;
    min-height: 0;
    min-width: 0;
    margin-top: var(--space-4);
    padding: 0 var(--space-6);
  }

  .focal-mask {
    position: absolute;
    box-shadow: 0 0 0 9999px var(--scrim);
    pointer-events: none;
  }

  .focal-frame {
    position: absolute;
    border: 2px solid var(--fg-on-accent);
    border-radius: var(--radius-sm);
    cursor: move;
  }

  .focal-frame__tag {
    position: absolute;
    top: var(--space-1);
    left: var(--space-1);
    padding: 0 var(--space-1);
    background: var(--bg-raised);
    border-radius: var(--radius-sm);
    font-size: 11px;
    color: var(--fg-default);
    pointer-events: none;
  }

  .focal-quick {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-3);
    padding: 0 var(--space-6);
  }
</style>
