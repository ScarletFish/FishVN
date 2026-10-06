<script>
  /** 状态分区容器：组头 + 网格。children 由调用方渲染成卡片 */
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