<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { durationText } from '../lib/playtime.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';

  const stat = useStat(api.dashboardPlaytimeTop);
  const items = $derived(stat.data ?? []);

  const max = $derived(items.reduce((m, x) => Math.max(m, x.seconds), 0));

</script>

<StatCard
  title={t('dashboard.playtime.title')}
  sub={t('dashboard.playtime.sub')}
  loading={stat.loading}
  error={stat.error}
  empty={items.length === 0}
  emptyText={t('dashboard.playtime.empty')}
>
  <ul class="playtime-list">
    {#each items as item, i (item.id)}
      <li class="playtime-row">
        <span class="playtime-rank" class:playtime-rank--top={i === 0}>{i + 1}</span>
        <span class="playtime-name">{item.name}</span>
        <span class="playtime-track">
          <span
            class="playtime-bar"
            style:width="{max > 0 ? Math.max(4, (item.seconds / max) * 100) : 0}%"
          ></span>
        </span>
        <span class="playtime-time">{durationText(t, item.seconds)}</span>
      </li>
    {/each}
  </ul>
</StatCard>

<style>
  .playtime-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex: 1;
    flex-direction: column;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .playtime-row {
    display: grid;
    grid-template-columns: 24px minmax(0, 8.5rem) 1fr auto;
    align-items: center;
    gap: var(--space-3);
  }

  .playtime-rank {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-pill);
    background: var(--bg-page);
    color: var(--fg-muted);
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
    font-variant-numeric: tabular-nums;
  }

  .playtime-rank--top {
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
    height: 10px;
    border-radius: var(--radius-pill);
    background: var(--bg-page);
    overflow: hidden;
  }

  .playtime-bar {
    display: block;
    height: 100%;
    border-radius: var(--radius-pill);
    background: var(--accent);
    opacity: 0.85;
  }

  .playtime-time {
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
</style>
