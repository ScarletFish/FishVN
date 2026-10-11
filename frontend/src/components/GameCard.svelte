<script>
  /** 列表卡片：封面缺失时用游戏名首字符占位 */
  let { item, onopen } = $props();

  const initial = $derived((item.name || '?')[0].toUpperCase());

  function open() {
    onopen?.(item.id);
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') open();
    else if (e.key === ' ') {
      e.preventDefault();
      open();
    }
  }
</script>

<div class="cover-card" role="button" tabindex="0" onclick={open} onkeydown={handleKeydown}>
  <div class="cover-card__media">
    {#if item.cover_path}
      <img src={item.cover_path} alt={item.name} loading="lazy" decoding="async" />
    {:else}
      <div class="cover-ph">{initial}</div>
    {/if}
  </div>

  <div class="cover-card__overlay">
    <h3 class="cover-card__title">{item.name}</h3>
  </div>
</div>