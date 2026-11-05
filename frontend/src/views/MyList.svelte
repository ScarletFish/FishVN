<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { STATUS_KEYS, statusLabel } from '../lib/status.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { pickDirectory, pickExecutable } from '../lib/pick-path.js';
  import { createScrollAnim } from '../lib/scroll-anim.js';
  import { captureHero } from '../lib/hero-transition.js';
  import {
    notifyLibraryTagsChanged,
    onLibraryItemsThrottled,
    LIBRARY_CHANGED,
  } from '../lib/library-events.js';
  import GameCard from '../components/GameCard.svelte';
  import GameForm from '../components/GameForm.svelte';
  import StatusSection from '../components/StatusSection.svelte';
  import Menu from '../components/Menu.svelte';
  import ScanAddDialog from '../components/ScanAddDialog.svelte';

  let { onopen, scroll = 0, filter = $bindable('all') } = $props();


  let games = $state([]);
  let loading = $state(true);
  let error = $state(null);
  let unwatchItems = null;
  let formOpen = $state(false);
  let formGameId = $state(null);
  let prefill = $state(null);
  let addMenuOpen = $state(false);
  let scanOpen = $state(false);
  let scanPath = $state('');

  function openForm(id) {
    prefill = null;
    formGameId = id;
    formOpen = true;
  }

  async function deleteGame(item) {
    const ok = await confirm('deleteGame', { name: item.name });
    if (!ok) return;
    await api.deleteGame(item.id);
    notifyLibraryTagsChanged();
    await load();
  }

  function openFromCard(item) {
    const card = document.querySelector(`.cover-card[data-game-id="${CSS.escape(String(item.id))}"]`);
    if (card) captureHero(card);
    onopen(item.id);
  }

  async function addFromFile() {
    const picked = await pickExecutable(t('mylist.pickExecutable'));
    if (!picked) return;
    await openPrefilled(picked);
  }

  async function addFromScan() {
    const dir = await pickDirectory(t('mylist.pickDirectory'));
    if (!dir) return;
    scanPath = dir;
    scanOpen = true;
  }

  async function openPrefilled(path) {
    try {
      const info = await api.inspectPath(path);
      prefill = {
        name: info.name,
        installations: [{ dir_path: info.dir_path, entry_path: info.entry_path ?? '' }],
      };
      formGameId = null;
      formOpen = true;
    } catch (e) {
      toast(e.message, 'error');
    }
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
    window.addEventListener(LIBRARY_CHANGED, load);
    unwatchItems = onLibraryItemsThrottled(load);
  });

  onDestroy(() => {
    window.removeEventListener(LIBRARY_CHANGED, load);
    unwatchItems?.();
  });

  const byName = (a, b) => a.name.localeCompare(b.name, 'zh-CN');

  const counts = $derived.by(() => {
    const all = { all: games.length };
    for (const s of STATUS_KEYS) all[s] = 0;
    for (const g of games) if (all[g.status] != null) all[g.status]++;
    return all;
  });

  const filtered = $derived(
    (filter === 'all' ? games : games.filter((g) => g.status === filter)).slice().sort(byName)
  );

  const sections = $derived(
    STATUS_KEYS.map((status) => ({
      status,
      label: statusLabel(status),
      items: filtered.filter((g) => g.status === status),
    })).filter((s) => s.items.length > 0)
  );

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
      <Menu
        bind:open={addMenuOpen}
        label={t('mylist.add')}
        items={[
          { label: t('mylist.addManual'), onclick: () => openForm(null) },
          { label: t('mylist.addFromFile'), onclick: addFromFile },
          { label: t('mylist.addScan'), onclick: addFromScan },
        ]}
      >
        {#snippet trigger()}
          <button
            type="button"
            class="mylist-add"
            aria-haspopup="menu"
            aria-expanded={addMenuOpen}
            aria-label={t('mylist.add')}
            data-tooltip={t('mylist.add')}
            onclick={() => (addMenuOpen = !addMenuOpen)}
            onblur={(e) => {
              if (!e.currentTarget.parentElement.contains(e.relatedTarget)) addMenuOpen = false;
            }}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
        {/snippet}
      </Menu>
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

<GameForm bind:open={formOpen} bind:gameId={formGameId} prefill={prefill} />

<ScanAddDialog bind:open={scanOpen} path={scanPath} ondone={load} />
