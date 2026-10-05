<script>
  import { t } from '../lib/i18n.js';

  // 占位数据：验证网格列数与卡宽，接入后端后替换
  const PLACEHOLDER = Array.from({ length: 24 }, (_, i) => String.fromCharCode(65 + (i % 26)));
</script>

<section class="view">
  <header class="view-header">
    <h1>{t('library.title')}</h1>
    <div class="view-header-right">
      <span class="lib-count">{PLACEHOLDER.length}</span>
    </div>
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