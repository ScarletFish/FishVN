<script>
  import Icon from '../components/Icon.svelte';
  import { onMount, tick } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { HOME_STATUS_KEYS, statusLabel } from '../lib/status.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { durationText } from '../lib/playtime.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { captureHero } from '../lib/hero-transition.js';
  import { DASHBOARD_MODULES } from '../lib/dashboard-layout.js';
  import { CONFIG_CHANGED } from '../lib/config-events.js';
  import {
    notifyAddMenuSummon,
    notifyLibraryChanged,
    notifyLibraryTagsChanged,
    onLibraryItemsThrottled,
    LIBRARY_CHANGED,
  } from '../lib/library-events.js';
  import { openForm } from '../lib/add-game.svelte.js';
  import { dashboardSort, dashboardGroup, gameComparator } from '../lib/list-sort.svelte.js';
  import { neighborsFromSections } from '../lib/detail-nav.js';
  import SortSelect from '../components/SortSelect.svelte';
  import Seg from '../components/Seg.svelte';
  import GameCard from '../components/GameCard.svelte';
  import StatusSection from '../components/StatusSection.svelte';
  import WindowedGrid from '../components/WindowedGrid.svelte';

  let { onopen, scroll = 0 } = $props();


  let layout = $state([]);
  let stats = $state(null);
  let recent = $state([]);
  let games = $state([]);
  let includeUninstalled = $state(true);
  let covers = $state({});
  let loading = $state(true);
  let error = $state(null);
  let scrollEl = $state(null);
  let headEl = $state(null);

  const moduleLabel = (id) => {
    const known = DASHBOARD_MODULES.find((m) => m.id === id);
    return known ? t(known.label) : id;
  };

  const localSections = $derived(
    HOME_STATUS_KEYS.map((status) => ({
      status,
      label: statusLabel(status),
      items: games
        .filter(
          (game) => game.status === status && (includeUninstalled || game.installation_count > 0)
        )
        .sort(gameComparator(dashboardSort.by)),
    })).filter((section) => section.items.length > 0)
  );

  const localNeighborIds = $derived(neighborsFromSections(localSections));
  const recentNeighborIds = $derived(recent.map((g) => g.id));
  const localFlatItems = $derived(localSections.flatMap((s) => s.items));

  function openLocal(item) {
    const card = document.querySelector(
      `.cover-card[data-game-id="${CSS.escape(String(item.id))}"]`
    );
    if (card) captureHero(card);
    onopen(item.id, { neighbors: localNeighborIds });
  }

  async function deleteGame(item) {
    const ok = await confirm('deleteGame', { name: item.name });
    if (!ok) return;
    await api.deleteGame(item.id);
    notifyLibraryChanged();
    notifyLibraryTagsChanged();
    await load();
  }

  function continueOpen(item, card) {
    captureHero(card, { frame: 'self', bg: '.continue-bg', aspect: 3 / 4 });
    onopen(item.id, { autostart: true, neighbors: recentNeighborIds });
  }

  async function load() {
    try {
      const [config, value, items, list] = await Promise.all([
        api.getConfig(),
        api.dashboardStats(),
        api.dashboardRecent().catch(() => []),
        api.listGames(),
      ]);
      layout = config.dashboard_layout;
      includeUninstalled = config.dashboard_include_uninstalled !== false;
      stats = value;
      recent = items;
      games = list;
      error = null;
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
    if (!error) await restoreScroll();
    await tick();
    window.dispatchEvent(new CustomEvent('app:ready'));
  }

  async function reload() {
    api.invalidateGames();
    await load();
  }

  let pendingRestore = true;

  async function restoreScroll() {
    if (!pendingRestore) return;
    pendingRestore = false;
    if (!scroll) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    await tick();
    viewport.scrollTop = scroll;
  }

  $effect(() => {
    for (const item of recent) {
      if (!item.cover_path || covers[item.id]) continue;
      coverSrc(item.cover_path)
        .then((url) => {
          if (url) covers = { ...covers, [item.id]: url };
        })
        .catch(() => {});
    }
  });

  $effect(() => {
    if (!scrollEl || !headEl || recent.length === 0) return;
    return initScrollDots({ scroll: scrollEl, total: recent.length, dotsParent: headEl });
  });

  onMount(() => {
    load();
    window.addEventListener(CONFIG_CHANGED, load);
    window.addEventListener(LIBRARY_CHANGED, reload);
    const unwatchItems = onLibraryItemsThrottled(reload);
    return () => {
      window.removeEventListener(CONFIG_CHANGED, load);
      window.removeEventListener(LIBRARY_CHANGED, load);
      unwatchItems();
    };
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
      {#if module.enabled && module.id === 'continue_play' && recent.length > 0}
        <div class="dashboard-section">
          <div class="dashboard-module-header" bind:this={headEl}>
            <span class="dashboard-section-title">{moduleLabel(module.id)}</span>
          </div>
          <div class="hscroll-section continue-scroll" bind:this={scrollEl}>
            {#each recent as item (item.id)}
              <div
                class="hscroll-card continue-card"
                role="button"
                tabindex="0"
                onclick={(e) => continueOpen(item, e.currentTarget)}
                onkeydown={(e) => e.key === 'Enter' && continueOpen(item, e.currentTarget)}
              >
                {#if covers[item.id]}
                  <div
                    class="continue-bg"
                    style:background-image={`url(${covers[item.id]})`}
                    style:background-position={`${item.cover_focal_x ?? 50}% ${item.cover_focal_y ?? 50}%`}
                  ></div>
                {:else}
                  <div class="continue-ph">{item.name.charAt(0)}</div>
                {/if}
                <div class="continue-overlay"></div>
                <div class="continue-content">
                  <span class="continue-eyebrow">{moduleLabel(module.id)}</span>
                  <span class="continue-title">{item.name}</span>
                </div>
                {#if item.total_seconds > 0}
                  <span class="continue-meta">{durationText(t, item.total_seconds)}</span>
                {/if}
                <button
                  type="button"
                  class="continue-play"
                  data-tooltip={t('dashboard.continueStart')}
                  aria-label={t('dashboard.continueStart')}
                  onclick={(e) => {
                    e.stopPropagation();
                    continueOpen(item, e.currentTarget.closest('.continue-card'));
                  }}
                >
                  <Icon name="play" size="lg" />
                </button>
              </div>
            {/each}
          </div>
        </div>
      {:else if module.enabled && module.id === 'stats' && stats && stats.total > 0}
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
      {:else if module.enabled && module.id === 'local_library' && localSections.length > 0}
        <div class="dashboard-section">
          <div class="dashboard-module-header">
            <span class="dashboard-section-title">{moduleLabel(module.id)}</span>
            <div class="dashboard-module-controls">
              <Seg
                label={t('dashboard.groupToggle')}
                options={[
                  { value: 'status', label: t('dashboard.groupStatus') },
                  { value: 'flat', label: t('dashboard.groupFlat') },
                ]}
                bind:value={dashboardGroup.by}
              />
              <SortSelect bind:value={dashboardSort.by} />
            </div>
          </div>
          {#if dashboardGroup.by === 'flat'}
            {#if localFlatItems.length >= 60}
              <WindowedGrid items={localFlatItems}>
                {#snippet children(item)}
                  <GameCard {item} onopen={() => openLocal(item)} onedit={openForm} ondelet={deleteGame} />
                {/snippet}
              </WindowedGrid>
            {:else}
              <div class="cover-grid">
                {#each localFlatItems as item (item.id)}
                  <GameCard {item} onopen={() => openLocal(item)} onedit={openForm} ondelet={deleteGame} />
                {/each}
              </div>
            {/if}
          {:else}
            {#each localSections as section (section.status)}
              <StatusSection variant="dashboard" label={section.label} items={section.items}>
                {#snippet children(item)}
                  <GameCard {item} onopen={() => openLocal(item)} onedit={openForm} ondelet={deleteGame} />
                {/snippet}
              </StatusSection>
            {/each}
          {/if}
        </div>
      {/if}
    {/each}

    {#if games.length === 0}
      <div class="dashboard-empty">
        <Icon name="tray" size="xl" />
        <p>{t('dashboard.emptyTitle')}</p>
        <p class="dashboard-empty__hint">{t('dashboard.emptyHint')}</p>
        <button type="button" class="btn btn-primary" onclick={notifyAddMenuSummon}>
          {t('mylist.add')}
        </button>
      </div>
    {/if}
  {/if}
</section>
