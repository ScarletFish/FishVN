<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import GameCard from '../components/GameCard.svelte';
  import GameForm from '../components/GameForm.svelte';
  import StatusSection from '../components/StatusSection.svelte';

  let { onopen, scroll = 0, onscroll } = $props();

  const STATUS_ORDER = ['playing', 'wishlist', 'finished', 'dropped'];

  let games = $state([]);
  let filter = $state('all');
  let loading = $state(true);
  let error = $state(null);
  let formOpen = $state(false);
  let formGameId = $state(null);

  function openForm(id) {
    formGameId = id;
    formOpen = true;
  }

  async function load() {
    try {
      games = await api.listGames();
      error = null;
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
    await restoreScroll();
    await tick();
    window.dispatchEvent(new CustomEvent('app:ready'));
  }

  async function restoreScroll() {
    if (!scroll) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    await tick();
    viewport.scrollTop = scroll;
  }

  onMount(load);

  onDestroy(() => {
    const viewport = getScrollViewport();
    if (viewport) onscroll?.(viewport.scrollTop);
  });

  const byName = (a, b) => a.name.localeCompare(b.name, 'zh-CN');

  const counts = $derived.by(() => {
    const all = { all: games.length };
    for (const s of STATUS_ORDER) all[s] = 0;
    for (const g of games) if (all[g.status] != null) all[g.status]++;
    return all;
  });

  const filtered = $derived(
    (filter === 'all' ? games : games.filter((g) => g.status === filter)).slice().sort(byName)
  );

  const sections = $derived(
    STATUS_ORDER.map((status) => ({
      status,
      label: t(`status.${status}`),
      items: filtered.filter((g) => g.status === status),
    })).filter((s) => s.items.length > 0)
  );
</script>

<section class="view">
  <div class="mylist-status-bar">
    <button
      type="button"
      class="mylist-status-item"
      class:active={filter === 'all'}
      onclick={() => (filter = 'all')}
    ><b>{counts.all}</b>{t('mylist.filter.all')}</button>
    {#each STATUS_ORDER as status (status)}
      <button
        type="button"
        class="mylist-status-item"
        class:active={filter === status}
        onclick={() => (filter = status)}
      ><b>{counts[status]}</b>{t(`status.${status}`)}</button>
    {/each}
  </div>

  <header class="view-header">
    <h1>{t('mylist.title')}</h1>
    <div class="view-header-right">
      <button
        type="button"
        class="mylist-add"
        onclick={() => openForm(null)}
        aria-label={t('mylist.add')}
        data-tooltip={t('mylist.add')}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    </div>
  </header>

  {#if loading}
    <p class="mylist-note">{t('common.loading')}</p>
  {:else if error}
    <p class="mylist-note">{t('mylist.error')}</p>
  {:else if games.length === 0}
    <div class="mylist-empty">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="7" height="7" rx="1"></rect>
        <rect x="14" y="3" width="7" height="7" rx="1"></rect>
        <rect x="3" y="14" width="7" height="7" rx="1"></rect>
        <rect x="14" y="14" width="7" height="7" rx="1"></rect>
      </svg>
      <p>{t('mylist.empty')}</p>
    </div>
  {:else if filtered.length === 0}
    <div class="mylist-empty">
      <p>{t('mylist.emptyFiltered', { label: t(`status.${filter}`) })}</p>
    </div>
  {:else}
    {#each sections as section (section.status)}
      <StatusSection label={section.label} items={section.items}>
        {#snippet children(item)}
          <GameCard {item} onopen={() => onopen(item.id)} onedit={openForm} />
        {/snippet}
      </StatusSection>
    {/each}
  {/if}
</section>

<GameForm bind:open={formOpen} bind:gameId={formGameId} onsaved={load} />
