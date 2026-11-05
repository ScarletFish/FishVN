<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { notifyBackfillRefresh } from '../lib/task-events.js';
  import { STATE_BADGE, STATE_DOT, STATE_LABEL } from '../lib/backfill-labels.js';
  import { mergeProgress } from '../lib/backfill-log.js';
  import {
    backfillStore,
    watchBackfill,
    unwatchBackfill,
    reloadBackfill,
    backfillCounts,
    backfillVisible,
    backfillItem,
  } from '../lib/backfill-store.svelte.js';
  import BackfillPanel from '../components/backfill/BackfillPanel.svelte';
  import BackfillLog from '../components/backfill/BackfillLog.svelte';

  let { scroll = 0 } = $props();

  const status = $derived(backfillStore.status);
  const counts = $derived(backfillCounts(status));
  const items = $derived(
    backfillVisible(status, backfillStore.filter, backfillStore.onlyNoId),
  );
  const selected = $derived(backfillItem(status, backfillStore.selected));
  const running = $derived(Boolean(status?.running));
  const paused = $derived(Boolean(status?.paused));
  const hasItems = $derived((status?.items?.length ?? 0) > 0);
  const total = $derived(status?.total ?? 0);
  const done = $derived(status?.done ?? 0);
  const ratio = $derived(total > 0 ? Math.min(1, done / total) : 0);
  const activeItem = $derived((status?.items ?? []).find((it) => it.state === 'active') ?? null);

  const FILTERS = [
    { key: 'all', label: 'backfillPage.filter.all' },
    { key: 'pending', label: 'backfillPage.filter.pending' },
    { key: 'active', label: 'backfillPage.filter.active' },
    { key: 'ok', label: 'backfillPage.filter.ok' },
    { key: 'unmatched', label: 'backfillPage.filter.unmatched' },
    { key: 'failed', label: 'backfillPage.filter.failed' },
  ];

  let starting = $state(false);
  let stopping = $state(false);
  let restored = $state(false);

  onMount(async () => {
    await watchBackfill();
  });

  onDestroy(() => {
    unwatchBackfill();
  });

  $effect(() => {
    if (restored || scroll <= 0 || backfillStore.loading) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    restored = true;
    const target = scroll;
    tick().then(() => {
      viewport.scrollTop = target;
    });
  });

  function pick(it) {
    backfillStore.selected = backfillStore.selected === it.game_id ? null : it.game_id;
  }

  function setFilter(key) {
    backfillStore.filter = key;
    if (!backfillStore.selected) return;
    const next = backfillVisible(status, key, backfillStore.onlyNoId);
    if (!next.some((it) => it.game_id === backfillStore.selected)) backfillStore.selected = null;
  }

  function toggleNoId() {
    backfillStore.onlyNoId = !backfillStore.onlyNoId;
    if (!backfillStore.selected) return;
    const next = backfillVisible(status, backfillStore.filter, backfillStore.onlyNoId);
    if (!next.some((it) => it.game_id === backfillStore.selected)) backfillStore.selected = null;
  }

  async function start() {
    if (starting || running) return;
    starting = true;
    try {
      await api.startBackfill();
      notifyBackfillRefresh();
      await reloadBackfill();
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      starting = false;
    }
  }

  async function togglePause() {
    try {
      if (paused) await api.resumeBackfill();
      else await api.pauseBackfill();
      await reloadBackfill();
    } catch (e) {
      toast(e.message, 'error');
    }
  }

  async function stop() {
    if (stopping) return;
    const ok = await confirm('backfillStop', { done, total });
    if (!ok) return;
    stopping = true;
    try {
      await api.cancelTask('vndb-backfill');
      await reloadBackfill();
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      stopping = false;
    }
  }

  async function retryFailed() {
    const ids = (status?.items ?? [])
      .filter((it) => it.state === 'failed' || it.state === 'unmatched')
      .map((it) => it.game_id);
    if (!ids.length || running) return;
    starting = true;
    try {
      await api.startBackfill(ids);
      notifyBackfillRefresh();
      await reloadBackfill();
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      starting = false;
    }
  }

  const failedCount = $derived(counts.failed + counts.unmatched);

  const shownLogs = $derived(
    backfillStore.logs.filter((r) => {
      const ev = r.event;
      if (status?.run_id && ev.run_id && ev.run_id !== status.run_id) return false;
      if (ev.phase === 'backfill' && !ev.game_id) return false;
      return true;
    }),
  );

  const logRows = $derived(mergeProgress(shownLogs));

  const showLog = $derived(running || (!backfillStore.logHidden && !selected));
  const showPanel = $derived(
    running || Boolean(selected) || (!backfillStore.logHidden && shownLogs.length > 0),
  );

  $effect(() => {
    if (running) backfillStore.logHidden = false;
  });

  function collapseLog() {
    backfillStore.logHidden = true;
  }

  function expandLog() {
    backfillStore.logHidden = false;
  }

  function retract(node) {
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) return { duration: 0 };
    const width = node.getBoundingClientRect().width;
    const gap = parseFloat(getComputedStyle(node.parentElement).columnGap) || 0;
    return {
      duration: 160,
      css: (t) =>
        `width: ${(width * t).toFixed(1)}px; margin-left: ${(-gap * (1 - t)).toFixed(1)}px; opacity: ${t.toFixed(2)};`,
    };
  }
</script>

<section class="view view--backfill">
  <header class="view-header">
    <div class="bf-title">
      <h1>{t('backfillPage.title')}</h1>
      <span class="bf-hint">{t('backfillPage.hint')}</span>
    </div>
    <div class="view-header-right">
      {#if running}
        <button type="button" class="btn btn-ghost btn-sm" onclick={togglePause}>
          {paused ? t('mylist.backfillResume') : t('mylist.backfillPause')}
        </button>
        <button
          type="button"
          class="btn btn-ghost btn-sm bf-stop"
          disabled={stopping}
          onclick={stop}
        >
          {stopping ? t('backfillPage.stopping') : t('mylist.backfillCancel')}
        </button>
      {:else}
        {#if failedCount > 0}
          <button type="button" class="btn btn-ghost btn-sm" disabled={starting} onclick={retryFailed}>
            {t('backfillPage.retryFailed', { count: failedCount })}
          </button>
        {/if}
        <button
          type="button"
          class="btn btn-primary btn-sm"
          disabled={starting || !hasItems}
          onclick={start}
        >
          {t('backfillPage.start')}
        </button>
      {/if}
    </div>
  </header>

  {#if running}
    <div class="bf-running">
      <div class="bf-running__line">
        <span class="bf-running__text">
          {#if status?.rate_limited}
            {t('backfillPage.rateLimited')}
          {:else if stopping}
            {t('backfillPage.stopping')}
          {:else}
            {t('backfillPage.running', { done, total })}
          {/if}
        </span>
        {#if activeItem}
          <span class="bf-running__current">{activeItem.name}</span>
        {/if}
        {#if paused}
          <span class="bf-running__tag">{t('backfillPage.paused')}</span>
        {/if}
      </div>
      <div class="bf-progress" role="progressbar" aria-valuenow={Math.round(ratio * 100)}>
        <span class="bf-progress__bar" style:width={`${ratio * 100}%`}></span>
      </div>
    </div>
  {/if}

  {#if hasItems}
    <div class="bf-filters">
      <div class="bf-seg" role="tablist">
        {#each FILTERS as f (f.key)}
          <button
            type="button"
            class="bf-seg__item"
            class:is-active={backfillStore.filter === f.key}
            role="tab"
            aria-selected={backfillStore.filter === f.key}
            onclick={() => setFilter(f.key)}
          >
            {t(f.label)}<b>{counts[f.key] ?? 0}</b>
          </button>
        {/each}
      </div>
      <div class="bf-filters__right">
        {#if !running && backfillStore.logHidden && logRows.length}
          <button type="button" class="bf-toggle" onclick={expandLog}>
            {t('backfillPage.logTitle')} {logRows.length}
          </button>
        {/if}
        <button
          type="button"
          class="bf-toggle"
          class:is-active={backfillStore.onlyNoId}
          aria-pressed={backfillStore.onlyNoId}
          data-tooltip={t('backfillPage.onlyNoIdHint')}
          onclick={toggleNoId}
        >
          {t('backfillPage.onlyNoId')}
        </button>
      </div>
    </div>
  {/if}

  {#if backfillStore.loading}
    <p class="bf-note">{t('common.loading')}</p>
  {:else if backfillStore.error}
    <p class="bf-note">{backfillStore.error}</p>
  {:else if !hasItems}
    <div class="bf-empty">
      <p>{t('backfillPage.empty')}</p>
      <p class="bf-empty__hint">{t('backfillPage.emptyHint')}</p>
    </div>
  {:else}
    <div class="bf-body">
      <div class="bf-list">
        {#if items.length === 0}
          <p class="bf-note">{t('backfillPage.filterEmpty')}</p>
        {/if}
        {#each items as it (it.game_id)}
          <button
            type="button"
            class="bf-row"
            class:is-selected={backfillStore.selected === it.game_id}
            class:is-active={it.state === 'active'}
            disabled={running}
            onclick={() => pick(it)}
          >
            <span class="bf-row__dot {STATE_DOT[it.state]}"></span>
            <span class="bf-row__local" data-tooltip={it.name}>{it.name}</span>
            <span class="bf-row__bound" class:is-blank={it.state === 'pending' && !it.vndb_id}>
              {#if it.state === 'ok'}
                <span class="bf-row__title" data-tooltip={it.hit_title ?? it.vndb_id}>
                  {it.hit_title ?? it.vndb_id ?? ''}
                </span>
                <span class="bf-row__sub">
                  {#if it.vndb_id}<span>{it.vndb_id}</span>{/if}
                  {#if it.hit_released}<span>{it.hit_released.slice(0, 4)}</span>{/if}
                  {#if it.filled.length}<span>{t('backfillPage.filledCount', { count: it.filled.length })}</span>{/if}
                  {#if it.bind_source === 'search'}
                    <span class="bf-row__src">{t('backfillPage.sourceSearch')}</span>
                  {:else if it.bind_source === 'existing'}
                    <span>{t('backfillPage.sourceExisting')}</span>
                  {/if}
                </span>
              {:else if it.state === 'unmatched'}
                <span class="bf-row__sub">{it.error ?? t('backfillPage.filter.unmatched')}</span>
              {:else if it.state === 'failed'}
                <span class="bf-row__title bf-row__title--error">{it.error ?? t('backfillPage.filter.failed')}</span>
                <span class="bf-row__sub">
                  {#if it.vndb_id}<span>{it.vndb_id}</span>{/if}
                </span>
              {:else if it.state === 'active'}
                <span class="bf-row__title bf-row__title--muted">{t('backfillPage.filter.active')}</span>
              {:else}
                <span class="bf-row__sub">{it.vndb_id ?? ''}</span>
              {/if}
            </span>
            <span class="bf-badge {STATE_BADGE[it.state]}">{t(STATE_LABEL[it.state])}</span>
          </button>
        {/each}
      </div>

      {#if showPanel}
        <aside class="bf-panel-slot" out:retract>
          <div class="bf-panel" class:is-log={showLog}>
            {#if showLog}
              <div class="bf-panel__head">
                <span class="bf-panel__title-row">
                  <span class="bf-panel__title">{t('backfillPage.logTitle')}</span>
                  <span class="bf-panel__meta">{logRows.length}</span>
                </span>
                {#if !running}
                  <button type="button" class="bf-panel__collapse" onclick={collapseLog}>
                    {t('backfillPage.logCollapse')}
                  </button>
                {/if}
              </div>
              <BackfillLog logs={shownLogs} {status} />
            {:else}
              <BackfillPanel
                item={selected}
                {running}
                onretried={reloadBackfill}
                onclose={() => (backfillStore.selected = null)}
              />
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {/if}
</section>
