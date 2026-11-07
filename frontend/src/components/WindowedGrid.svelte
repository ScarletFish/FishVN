<script>
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { cardWidth, computeColsFor } from '../lib/grid-metrics.js';

  let { items, children } = $props();

  let containerEl = $state(null);
  let scrollEl = $state(null);
  let cols = $state(1);
  let rowH = $state(0);
  let scrollMargin = $state(0);
  let colTemplate = $state('');
  let virtualizerStore = null;
  let virtualizer = null;
  let storeReady = $state(false);
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
      const gap = parseFloat(getComputedStyle(el).rowGap) || 0;
      cols = computeColsFor(el, { gap });
      const card = cardWidth(width, cols, gap);
      rowH = card * (4 / 3) + gap;
      colTemplate = `repeat(${cols}, minmax(0, 1fr))`;
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
    const options = {
      count: rows,
      estimateSize: () => rowH,
      scrollMargin,
    };
    if (!virtualizerStore) {
      virtualizerStore = createVirtualizer({
        ...options,
        getScrollElement: () => scrollEl,
        overscan: 3,
        getItemKey: (index) => index,
      });
      storeReady = true;
      return;
    }
    virtualizer?.setOptions(options);
  });

  $effect(() => {
    if (!storeReady) return;
    return virtualizerStore.subscribe((instance) => {
      virtualizer = instance;
      vItems = instance.getVirtualItems();
      vTotal = instance.getTotalSize();
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
