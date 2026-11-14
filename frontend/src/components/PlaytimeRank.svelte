<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { durationText, compactDuration } from '../lib/playtime.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';

  let { total = 0 } = $props();

  const stat = useStat(api.dashboardPlaytimeTop);
  const items = $derived(stat.data ?? []);

  const max = $derived(items.reduce((m, x) => Math.max(m, x.seconds), 0));
  const sum = $derived(items.reduce((s, x) => s + x.seconds, 0));
  const share = $derived(
    total > 0 && sum > 0 ? Math.min(100, Math.round((sum / total) * 100)) : 0,
  );

  function shortTime(seconds) {
    const value = Math.round(seconds);
    if (value < 60) return `${value}s`;
    return compactDuration(Math.round(value / 60));
  }
</script>

<StatCard
  title={t('dashboard.playtime.title')}
  loading={stat.loading}
  error={stat.error}
  empty={items.length === 0}
  emptyText={t('dashboard.playtime.empty')}
>
  <ul class="playtime-list" role="list">
    {#each items as item, i (item.id)}
      <li class="playtime-row" role="listitem">
        <span class="playtime-rank" class:playtime-rank--top={i === 0}>{i + 1}</span>
        <span class="playtime-name" title={item.name}>{item.name}</span>
        <span class="playtime-track">
          <span
            class="playtime-bar"
            style:width="{max > 0 ? Math.max(4, (item.seconds / max) * 100) : 0}%"
          ></span>
        </span>
        <span class="playtime-time">
          <span class="playtime-time-full">{durationText(t, item.seconds)}</span>
          <span class="playtime-time-short">{shortTime(item.seconds)}</span>
        </span>
      </li>
    {/each}
  </ul>

  {#if share > 0}
    <p class="playtime-sum">
      {t('dashboard.playtime.sum', {
        n: items.length,
        sum: durationText(t, sum),
        pct: share,
      })}
    </p>
  {/if}
</StatCard>

<style>
  .playtime-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    flex: 1;
    grid-template-columns: 24px minmax(0, 9rem) minmax(0, 1fr) auto;
    grid-auto-rows: minmax(32px, auto);
    align-content: start;
    align-items: center;
    column-gap: var(--space-3);
    row-gap: 2px;
    container-type: inline-size;
  }

  .playtime-row {
    display: contents;
  }

  .playtime-rank {
    color: var(--fg-muted);
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .playtime-rank--top {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-pill);
    background: var(--accent);
    color: var(--fg-on-accent);
  }

  .playtime-name {
    color: var(--fg-default);
    font-size: var(--text-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .playtime-track {
    height: 8px;
    border-radius: var(--radius-pill);
    background: var(--bg-hover);
    overflow: hidden;
  }

  .playtime-bar {
    display: block;
    height: 100%;
    border-radius: var(--radius-pill);
    background: var(--chart-main);
  }

  .playtime-time {
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    text-align: right;
    white-space: nowrap;
  }

  .playtime-time-short {
    display: none;
  }

  .playtime-sum {
    margin: var(--space-3) 0 0;
    color: var(--fg-muted);
    font-size: var(--text-xs);
  }

  @container (max-width: 420px) {
    .playtime-time-full {
      display: none;
    }

    .playtime-time-short {
      display: inline;
    }
  }
</style>
