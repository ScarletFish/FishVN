<script>
  import { t } from '../lib/i18n.js';

  let { title, sub = '', loading = false, error = null, empty = false, emptyText = '', children } = $props();
</script>

<div class="stat-card">
  <header class="stat-card-head">
    <h2>{title}</h2>
    {#if sub && !loading && !error && !empty}
      <span class="stat-card-sub">{sub}</span>
    {/if}
  </header>

  {#if loading}
    <div class="stat-card-body">
      <p class="stat-card-note">{t('common.loading')}</p>
    </div>
  {:else if error}
    <div class="stat-card-body">
      <p class="stat-card-note stat-card-note--error">{error}</p>
    </div>
  {:else if empty}
    <div class="stat-card-body">
      <p class="stat-card-note">{emptyText}</p>
    </div>
  {:else}
    <div class="stat-card-body">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .stat-card {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .stat-card-head {
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .stat-card-head h2 {
    margin: 0;
    color: var(--fg-default);
    font-size: var(--text-base);
    font-weight: var(--fw-bold);
    letter-spacing: -0.01em;
  }

  .stat-card-sub {
    color: var(--fg-muted);
    font-size: var(--text-sm);
  }

  .stat-card-note {
    margin: 0;
    color: var(--fg-muted);
    font-size: var(--text-sm);
  }

  .stat-card-note--error {
    color: var(--error);
  }

  .stat-card-body {
    position: relative;
    display: flex;
    flex: 1;
    flex-direction: column;
    justify-content: center;
    min-height: 0;
  }
</style>
