<script>
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import { getScrollViewport } from '../lib/scrollbars.js';

  let { items, children } = $props();

  let containerEl = $state(null);
  let scrollEl = $state(null);
  let cols = $state(1);
  let rowH = $state(0);
  let scrollMargin = $state(0);
  let colTemplate = $state('');
  let virtualizer = $state(null);
  let vItems = $state([]);
  let vTotal = $state(0);

  const rows = $derived(cols > 0 ? Math.ceil(items.length / cols) : 0);

  $effect(() => {
    const el = containerEl;
    if (!el) return;
    const viewport = getScrollViewport() ?? el.closest('.shell__main');
    if (!viewport) return;
    scrollEl = viewport;

    let lastWidth = 0;
    const measure = () => {
      const width = el.clientWidth;
      if (width <= 0 || width === lastWidth) return;
      lastWidth = width;
      const style = getComputedStyle(el);
      const gap = parseFloat(style.rowGap) || 0;
      const raw = style.getPropertyValue('--card-w').trim();
      const base = parseFloat(style.fontSize) || 16;
      let card = parseFloat(raw);
      if (raw.endsWith('rem')) card *= base;
      if (!Number.isFinite(card) || card <= 0) return;
      card = Math.min(card, width);
      cols = Math.max(1, Math.floor((width + gap) / (card + gap)));
      rowH = Math.round(card * (4 / 3) + gap);
      colTemplate = `repeat(${cols}, min(var(--card-w), 100%))`;
      scrollMargin = Math.round(
        el.getBoundingClientRect().top - viewport.getBoundingClientRect().top + viewport.scrollTop
      );
    };

    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!containerEl || !scrollEl || rowH <= 0) return;
    if (!virtualizer) {
      virtualizer = createVirtualizer({
        count: rows,
        getScrollElement: () => scrollEl,
        estimateSize: () => rowH,
        overscan: 3,
        getItemKey: (index) => index,
        scrollMargin,
      });
    } else {
      virtualizer.setOptions({
        count: rows,
        estimateSize: () => rowH,
        scrollMargin,
      });
    }
  });

  $effect(() => {
    const v = virtualizer;
    if (!v) return;
    return v.subscribe(() => {
      vItems = v.getVirtualItems();
      vTotal = v.getTotalSize();
    });
  });
</script>

<div class="virtual-grid" bind:this={containerEl} style:height={`${vTotal}px`}>
  {#each vItems as row (row.key)}
    {@const start = row.index * cols}
    <div
      class="virtual-row"
      style:top={`${row.start - scrollMargin}px`}
      style:grid-template-columns={colTemplate}
    >
      {#each items.slice(start, start + cols) as item (item.id)}
        {@render children(item)}
      {/each}
    </div>
  {/each}
</div>

<style>
  .virtual-grid {
    position: relative;
    width: 100%;
  }

  .virtual-row {
    position: absolute;
    left: 0;
    right: 0;
    display: grid;
    gap: var(--space-4);
    align-items: flex-start;
  }
</style>
