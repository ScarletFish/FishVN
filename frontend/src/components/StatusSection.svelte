<script>
  import WindowedGrid from './WindowedGrid.svelte';

  let { label, count, items, variant = 'mylist', children } = $props();
</script>

<section class={variant === 'dashboard' ? 'dashboard-status-section' : 'mylist-section'}>
  <header class={variant === 'dashboard' ? 'dashboard-status-header' : 'mylist-section-header'}>
    <span class={variant === 'dashboard' ? 'dashboard-status-title' : 'mylist-section-title'}>{label}</span>
    <span class={variant === 'dashboard' ? 'dashboard-status-count' : 'mylist-section-count'}>{count ?? items.length}</span>
  </header>
  {#if items.length >= 60}
    <WindowedGrid {items} {children} />
  {:else}
    <div class="cover-grid">
      {#each items as item (item.id)}
        {@render children(item)}
      {/each}
    </div>
  {/if}
</section>