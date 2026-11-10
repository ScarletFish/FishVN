<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { STATUS_KEYS, statusLabel } from '../lib/status.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { createScrollAnim } from '../lib/scroll-anim.js';
  import { captureHero } from '../lib/hero-transition.js';
  import {
    notifyAddMenuSummon,
    notifyLibraryChanged,
    notifyLibraryTagsChanged,
    onLibraryItemsThrottled,
    LIBRARY_CHANGED,
  } from '../lib/library-events.js';
  import { openForm } from '../lib/add-game.svelte.js';
  import { gameComparator, myListSort } from '../lib/list-sort.svelte.js';
  import SortSelect from '../components/SortSelect.svelte';
  import GameCard from '../components/GameCard.svelte';
  import StatusSection from '../components/StatusSection.svelte';
  import { neighborsFromSections } from '../lib/detail-nav.js';

  let { onopen, scroll = 0, filter = $bindable('all') } = $props();


  let games = $state([]);
  let loading = $state(true);
  let error = $state(null);
  let unwatchItems = null;

  async function deleteGame(item) {
    const ok = await confirm('deleteGame', { name: item.name });
    if (!ok) return;
    await api.deleteGame(item.id);
    notifyLibraryChanged();
    notifyLibraryTagsChanged();
    await load();
  }

  function openFromCard(item) {
    const card = document.querySelector(`.cover-card[data-game-id="${CSS.escape(String(item.id))}"]`);
    if (card) captureHero(card);
    onopen(item.id, { neighbors: neighborIds });
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
    if (!error) await restoreScroll();
    await tick();
    window.dispatchEvent(new CustomEvent('app:ready'));
  }

  async function reload() {
    api.invalidateGames();
    await load();
  }

  let pendingRestore = true;
  let restored = false;

  async function restoreScroll() {
    if (!pendingRestore) return;
    if (!scroll) {
      pendingRestore = false;
      return;
    }
    const viewport = getScrollViewport();
    if (!viewport) return;
    pendingRestore = false;
    await tick();
    viewport.scrollTop = scroll;
    restored = true;
  }

  onMount(() => {
    load();
    window.addEventListener(LIBRARY_CHANGED, reload);
    unwatchItems = onLibraryItemsThrottled(reload);
  });

  onDestroy(() => {
    window.removeEventListener(LIBRARY_CHANGED, load);
    unwatchItems?.();
  });

  const counts = $derived.by(() => {
    const all = { all: games.length };
    for (const s of STATUS_KEYS) all[s] = 0;
    for (const g of games) if (all[g.status] != null) all[g.status]++;
    return all;
  });

  const filtered = $derived(
    (filter === 'all' ? games : games.filter((g) => g.status === filter)).slice().sort(gameComparator(myListSort.by))
  );

  const sections = $derived(
    STATUS_KEYS.map((status) => ({
      status,
      label: statusLabel(status),
      items: filtered.filter((g) => g.status === status),
    })).filter((s) => s.items.length > 0)
  );

  const neighborIds = $derived(filtered.map((g) => g.id));

  const scrollAnim = createScrollAnim();

  const animSig = $derived(
    loading || error || filtered.length === 0
      ? null
      : `${filter}|${filtered.map((g) => g.id).join(',')}`
  );

  $effect(() => {
    const sig = animSig;
    if (!sig) return;
    tick().then(() => {
      if (animSig !== sig) return;
      if (restored) {
        restored = false;
        return;
      }
      scrollAnim.build({
        sections: document.querySelectorAll('.mylist-section'),
        cards: document.querySelectorAll('.cover-card'),
      });
    });
  });

  onDestroy(() => scrollAnim.kill());
</script>

<section class="view">
  {#if !loading}
    <div class="mylist-status-bar">
      <button
        type="button"
        class="mylist-status-item"
        class:active={filter === 'all'}
        onclick={() => (filter = 'all')}
      ><b>{counts.all}</b>{t('mylist.filter.all')}</button>
      {#each STATUS_KEYS as status (status)}
        <button
          type="button"
          class="mylist-status-item"
          class:active={filter === status}
          onclick={() => (filter = status)}
        ><b>{counts[status]}</b>{statusLabel(status)}</button>
      {/each}
    </div>
  {/if}

  <header class="view-header">
    <h1>{t('mylist.title')}</h1>
    <div class="view-header-right">
      <SortSelect bind:value={myListSort.by} />
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
      <p class="mylist-empty__hint">{t('dashboard.emptyHint')}</p>
      <button type="button" class="btn btn-primary" onclick={notifyAddMenuSummon}>
        {t('mylist.add')}
      </button>
    </div>
  {:else if filtered.length === 0}
    <div class="mylist-empty">
      <p>{t('mylist.emptyFiltered', { label: filter === 'all' ? t('mylist.filter.all') : statusLabel(filter) })}</p>
    </div>
  {:else}
    {#each sections as section (section.status)}
      <StatusSection label={section.label} items={section.items}>
        {#snippet children(item)}
          <GameCard {item} onopen={() => openFromCard(item)} onedit={openForm} ondelet={deleteGame} />
        {/snippet}
      </StatusSection>
    {/each}
  {/if}
</section>
