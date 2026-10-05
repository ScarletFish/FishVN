<script>
  // 占位数据：验证网格列数与卡宽，接入后端后替换
  const PLACEHOLDER = Array.from({ length: 24 }, (_, i) => String.fromCharCode(65 + (i % 26)));
</script>

<section class="view">
  <header class="lib-head">
    <h1>库</h1>
    <span class="lib-count">{PLACEHOLDER.length}</span>
  </header>

  <div class="cover-grid">
    {#each PLACEHOLDER as letter (letter)}
      <article class="cover-card">
        <div class="cover-card__ph">{letter}</div>
      </article>
    {/each}
  </div>
</section>

<style>
  .lib-head {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  h1 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--fw-semibold);
    letter-spacing: -0.01em;
    color: var(--fg-default);
  }

  .lib-count {
    font-size: var(--text-compact-md);
    font-weight: var(--fw-semibold);
    font-variant-numeric: tabular-nums;
    color: var(--fg-muted);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    padding: var(--space-025) var(--space-2);
  }
</style>