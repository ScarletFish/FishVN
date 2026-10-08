<script>
  /** 列表卡片：封面缺失时用游戏名首字符占位 */
  let { item } = $props();

  const initial = $derived((item.name || '?')[0].toUpperCase());
</script>

<article class="cover-card" tabindex="0">
  <div class="cover-card__media">
    {#if item.cover_path}
      <img src={item.cover_path} alt={item.name} loading="lazy" decoding="async" />
    {:else}
      <div class="cover-card__ph">{initial}</div>
    {/if}
  </div>

  <div class="cover-card__overlay">
    <h3 class="cover-card__title">{item.name}</h3>
  </div>
</article>