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
  import { countsOf, focusRows, mergeCatalog, pickRow, visibleRows } from '../lib/backfill-list.js';
  import {
    backfillStore,
    watchBackfill,
    unwatchBackfill,
    reloadBackfill,
    mergeBackfillItem,
  } from '../lib/backfill-store.svelte.js';
  import BackfillPanel from '../components/backfill/BackfillPanel.svelte';
  import Seg from '../components/Seg.svelte';
  import BackfillLog from '../components/backfill/BackfillLog.svelte';
  import { openForm } from '../lib/add-game.svelte.js';

  let { scroll = 0 } = $props();

  const status = $derived(backfillStore.status);
  const rows = $derived(
    mergeCatalog(backfillStore.games, status?.items, backfillStore.localOverlay),
  );
  const focus = $derived(focusRows(rows, backfillStore.pendingIds));
  const scopeOn = $derived(backfillStore.scope === 'focus' && focus.size > 0);
  const scope = $derived(scopeOn ? focus : null);
  const counts = $derived(countsOf(rows, scope));
  const items = $derived(visibleRows(rows, backfillStore.filter, scope, backfillStore.onlyNoId));
  const selected = $derived(pickRow(rows, backfillStore.selected));
  const running = $derived(Boolean(status?.running));
  const paused = $derived(Boolean(status?.paused));
  const hasItems = $derived(rows.length > 0);
  const total = $derived(status?.total ?? 0);
  const done = $derived(status?.done ?? 0);
  const ratio = $derived(total > 0 ? Math.min(1, done / total) : 0);
  const stage = $derived(status?.stage ?? '');
  const imagesDone = $derived(status?.images_done ?? 0);
  const imagesTotal = $derived(status?.images_total ?? 0);
  const imagesRatio = $derived(imagesTotal > 0 ? Math.min(1, imagesDone / imagesTotal) : 0);
  const activeItem = $derived(rows.find((it) => it.state === 'active') ?? null);

  const FILTERS = [
    { key: 'all', label: 'backfillPage.filter.all' },
    { key: 'ok', label: 'backfillPage.filter.ok' },
    { key: 'review', label: 'backfillPage.filter.review' },
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

  function clearHiddenSelection() {
    if (!backfillStore.selected) return;
    const next = visibleRows(rows, backfillStore.filter, scope, backfillStore.onlyNoId);
    if (!next.some((it) => it.game_id === backfillStore.selected)) backfillStore.selected = null;
  }

  function setFilter(key) {
    backfillStore.filter = key;
    clearHiddenSelection();
  }

  function toggleNoId() {
    backfillStore.onlyNoId = !backfillStore.onlyNoId;
    clearHiddenSelection();
  }

  function setScope(next) {
    if (next === 'focus' && focus.size === 0) return;
    backfillStore.scope = next;
    clearHiddenSelection();
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
      stopping = false;
      toast(e.message, 'error');
    }
  }

  $effect(() => {
    if (!running) stopping = false;
  });

  async function retryFailed() {
    const ids = rows
      .filter((it) => it.state === 'failed' || it.state === 'unmatched')
      .filter((it) => !scopeOn || focus.has(it.game_id))
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
          disabled={starting || counts.todo === 0}
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
          {:else if stage === 'images'}
            {t('backfillPage.imagesRunning', { done: imagesDone, total: imagesTotal })}
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
      <div
        class="bf-progress"
        role="progressbar"
        aria-valuenow={Math.round((stage === 'images' ? imagesRatio : ratio) * 100)}
      >
        <span
          class="bf-progress__bar"
          style:width={`${(stage === 'images' ? imagesRatio : ratio) * 100}%`}
        ></span>
      </div>
    </div>
  {/if}

  {#if hasItems}
    <div class="bf-filters">
      <div class="bf-filters__left">
        <Seg
          label={t('backfillPage.scopeLabel')}
          value={scopeOn ? 'focus' : 'all'}
          onchange={(v) => setScope(v)}
          options={[
            { value: 'focus', label: t('backfillPage.scopeFocus'), count: focus.size, disabled: focus.size === 0 },
            { value: 'all', label: t('backfillPage.scopeAll'), count: rows.length },
          ]}
        />
        <Seg
          role="tablist"
          value={backfillStore.filter}
          onchange={(v) => setFilter(v)}
          options={FILTERS.map((f) => ({
            value: f.key,
            label: t(f.label),
            count: counts[f.key] ?? 0,
            tooltip: f.key === 'review' ? t('backfillPage.reviewHint') : undefined,
          }))}
        />
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
          disabled={backfillStore.filter === 'review'}
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
                    <span class="bf-row__src">
                      {t('backfillPage.sourceSearch')}{#if it.hit_count > 1} · {t('backfillPage.hits', { count: it.hit_count })}{/if}
                    </span>
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
            {#if it.state}
              <span class="bf-badge {STATE_BADGE[it.state]}">{t(STATE_LABEL[it.state])}</span>
            {/if}
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
                onchanged={mergeBackfillItem}
                onopenform={openForm}
                onclose={() => (backfillStore.selected = null)}
              />
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {/if}
</section>
