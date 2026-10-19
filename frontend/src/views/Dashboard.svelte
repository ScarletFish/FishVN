<script>
  import { onMount } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { durationText } from '../lib/playtime.js';
  import { DASHBOARD_MODULES } from '../lib/dashboard-layout.js';
  import { CONFIG_CHANGED } from '../lib/config-events.js';

  let { onopen } = $props();

  let layout = $state([]);
  let stats = $state(null);
  let loading = $state(true);
  let error = $state(null);

  const moduleLabel = (id) => {
    const known = DASHBOARD_MODULES.find((m) => m.id === id);
    return known ? t(known.label) : id;
  };

  async function load() {
    try {
      const [config, value] = await Promise.all([api.getConfig(), api.dashboardStats()]);
      layout = config.dashboard_layout;
      stats = value;
      error = null;
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
    window.dispatchEvent(new CustomEvent('app:ready'));
  }

  onMount(() => {
    load();
    window.addEventListener(CONFIG_CHANGED, load);
    return () => window.removeEventListener(CONFIG_CHANGED, load);
  });
</script>

<section class="view">
  <header class="view-header">
    <h1>{t('dashboard.title')}</h1>
  </header>

  {#if loading}
    <p class="dashboard-note">{t('common.loading')}</p>
  {:else if error}
    <p class="dashboard-note">{t('dashboard.error')}</p>
  {:else}
    {#each layout as module (module.id)}
      {#if module.enabled && module.id === 'stats' && stats}
        <div class="dashboard-section">
          <div class="dashboard-stats">
            <div class="dashboard-stat">
              <b>{stats.total}</b>
              <span>{t('dashboard.statTotal')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{stats.playing}</b>
              <span>{t('dashboard.statPlaying')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{stats.wishlist}</b>
              <span>{t('dashboard.statWishlist')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{stats.finished}</b>
              <span>{t('dashboard.statFinished')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{stats.dropped}</b>
              <span>{t('dashboard.statDropped')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{durationText(t, stats.total_seconds)}</b>
              <span>{t('dashboard.statTotalTime')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{durationText(t, stats.week_seconds)}</b>
              <span>{t('dashboard.statWeekTime')}</span>
            </div>
            <div class="dashboard-stat">
              <b>{stats.play_count}</b>
              <span>{t('dashboard.statPlayCount')}</span>
            </div>
          </div>
        </div>
      {:else if module.enabled && module.id === 'continue_play'}
        <div class="dashboard-section">
          <div class="dashboard-section-title">{moduleLabel(module.id)}</div>
          <p class="dashboard-note">{t('dashboard.comingSoon')}</p>
        </div>
      {/if}
    {/each}
  {/if}
</section>
