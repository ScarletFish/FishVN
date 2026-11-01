<script>
  let {
    title,
    count = null,
    headerEl = $bindable(null),
    action = null,
    class: extra = '',
    children,
  } = $props();
</script>

<section class="detail-section section-card {extra}">
  <div class="detail-section-header" bind:this={headerEl}>
    <h3>{title}</h3>
    {#if count !== null}
      <span class="detail-section-count">{count}</span>
    {/if}
    {#if action}
      <span class="detail-section-action">{@render action()}</span>
    {/if}
  </div>
  {@render children()}
</section>
