<script>
  let { label, count, items, children } = $props();
</script>

<section class="mylist-section">
  <header class="mylist-section-header">
    <span class="mylist-section-title">{label}</span>
    <span class="mylist-section-count">{count ?? items.length}</span>
  </header>
  <div class="cover-grid">
    {#each items as item (item.id)}
      {@render children(item)}
    {/each}
  </div>
</section>