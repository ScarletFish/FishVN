<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { durationText } from '../lib/playtime.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import ActivityChart from '../components/ActivityChart.svelte';
  import PlaytimeRank from '../components/PlaytimeRank.svelte';
  import EraChart from '../components/EraChart.svelte';
  import TagCloud from '../components/TagCloud.svelte';

  const stat = useStat(api.dashboardStats);
  const stats = $derived(stat.data);

  const kpis = $derived.by(() => {
    if (!stats) return [];
    return [
      { value: stats.total, label: t('dashboard.statTotal') },
      { value: stats.playing, label: t('dashboard.statPlaying') },
      { value: stats.wishlist, label: t('dashboard.statWishlist') },
      { value: stats.finished, label: t('dashboard.statFinished') },
      { value: stats.stalled, label: t('dashboard.statStalled') },
      { value: stats.dropped, label: t('dashboard.statDropped') },
      { value: durationText(t, stats.total_seconds), label: t('dashboard.statTotalTime') },
      { value: durationText(t, stats.week_seconds), label: t('dashboard.statWeekTime') },
      { value: stats.play_count, label: t('dashboard.statPlayCount') },
    ];
  });

</script>

<section class="view">
  <header class="view-header">
    <h1>{t('dashboard.module.stats')}</h1>
    {#if stats}
      <span class="view-header-right">{t('stats.summary', { count: stats.total })}</span>
    {/if}
  </header>

  {#if stat.loading}
    <p class="stats-note">{t('common.loading')}</p>
  {:else if stat.error}
    <p class="stats-note stats-note--error">{stat.error}</p>
  {:else}
    <div class="dashboard-stats stats-kpi">
      {#each kpis as kpi (kpi.label)}
        <div class="dashboard-stat">
          <b>{kpi.value}</b>
          <span>{kpi.label}</span>
        </div>
      {/each}
    </div>

    <div class="stats-panels">
      <div class="stats-slot"><ActivityChart /></div>

      <div class="stats-row">
        <div class="stats-slot"><PlaytimeRank /></div>
        <div class="stats-slot"><EraChart /></div>
      </div>

      <div class="stats-slot"><TagCloud /></div>
    </div>
  {/if}
</section>

<style>
  .stats-panels {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .stats-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .stats-slot {
    height: 300px;
  }

  .stats-slot > :global(*) {
    height: 100%;
  }

  .stats-note {
    margin: var(--space-5) 0;
    color: var(--fg-muted);
    font-size: var(--text-sm);
  }

  .stats-note--error {
    color: var(--error);
  }
</style>
