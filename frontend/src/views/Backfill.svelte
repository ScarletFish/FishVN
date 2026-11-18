<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { notifyBackfillRefresh } from '../lib/task-events.js';
  import { formatDateTime } from '../lib/format.js';
  import { STATE_DOT } from '../lib/backfill-labels.js';
  import { mergeProgress } from '../lib/backfill-log.js';
  import { focusRows, mergeCatalog, pickOnClick, pickRow, prunePicks, visibleRows } from '../lib/backfill-list.js';
  import {
    backfillStore,
    watchBackfill,
    unwatchBackfill,
    reloadBackfill,
    mergeBackfillItem,
  } from '../lib/backfill-store.svelte.js';
  import BackfillPanel from '../components/backfill/BackfillPanel.svelte';
  import Seg from '../components/Seg.svelte';
  import SelectDd from '../components/SelectDd.svelte';
  import BackfillLog from '../components/backfill/BackfillLog.svelte';
  import { openForm } from '../lib/add-game.svelte.js';

  let { scroll = 0, onopendetail } = $props();

  const status = $derived(backfillStore.status);
  const rows = $derived(
    mergeCatalog(backfillStore.games, status?.items, backfillStore.localOverlay),
  );
  const focus = $derived(focusRows(rows));
  const scopeOn = $derived(backfillStore.scope === 'focus' && focus.size > 0);
  const scope = $derived(scopeOn ? focus : null);
  const items = $derived(visibleRows(rows, scope, backfillStore.onlyNoId));
  const selected = $derived(pickRow(rows, backfillStore.selected));
  const picked = $derived(backfillStore.picked);
  const pickedSet = $derived(new Set(backfillStore.picked));
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

  let starting = $state(false);
  let stopping = $state(false);
  let restored = $state(false);
  let runList = $state([]);
  let runPick = $state('current');
  let historyLogs = $state(null);
  let wasRunning = $state(false);
  let autoPicked = $state(false);

  onMount(async () => {
    await watchBackfill();
    loadRuns();
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

  function rowClick(it, ev) {
    if (running) return;
    const next = pickOnClick(
      {
        picked: backfillStore.picked,
        anchorId: backfillStore.anchorId,
        selected: backfillStore.selected,
      },
      items,
      it.game_id,
      { shift: ev.shiftKey, toggle: ev.metaKey || ev.ctrlKey },
    );
    backfillStore.picked = next.picked;
    backfillStore.anchorId = next.anchorId;
    backfillStore.selected = next.selected;
  }

  function pruneSelection() {
    const next = visibleRows(rows, scope, backfillStore.onlyNoId);
    const alive = new Set(next.map((it) => it.game_id));
    const kept = prunePicks(backfillStore, alive);
    const dropped = backfillStore.picked.length - kept.picked.length;
    backfillStore.picked = kept.picked;
    backfillStore.anchorId = kept.anchorId;
    backfillStore.selected = kept.selected;
    if (dropped > 0) toast(t('backfillPage.pickedCleared', { count: dropped }), 'info');
  }

  function toggleNoId() {
    backfillStore.onlyNoId = !backfillStore.onlyNoId;
    pruneSelection();
  }

  function setScope(next) {
    if (next === 'focus' && focus.size === 0) return;
    backfillStore.scope = next;
    pruneSelection();
  }

  async function launch(ids) {
    if (starting || running) return false;
    starting = true;
    try {
      await api.startBackfill(ids);
      notifyBackfillRefresh();
      await reloadBackfill();
      return true;
    } catch (e) {
      toast(e.message, 'error');
      return false;
    } finally {
      starting = false;
    }
  }

  async function start() {
    const ids = [...backfillStore.picked];
    if (await launch(ids.length ? ids : undefined)) backfillStore.picked = [];
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

  const failedRows = $derived(
    rows.filter(
      (row) =>
        (row.state === 'failed' || row.state === 'unmatched') &&
        (!scopeOn || focus.has(row.game_id)),
    ),
  );

  const failedCount = $derived(failedRows.length);

  async function retryFailed() {
    const ids = failedRows.map((it) => it.game_id);
    if (ids.length) await launch(ids);
  }

  const shownLogs = $derived(
    backfillStore.logs.filter((r) => {
      const ev = r.event;
      if (status?.run_id && ev.run_id && ev.run_id !== status.run_id) return false;
      if (ev.phase === 'backfill' && !ev.game_id) return false;
      return true;
    }),
  );

  const displayLogs = $derived(runPick === 'current' ? shownLogs : (historyLogs ?? []));
  const logRows = $derived(mergeProgress(displayLogs));

  const runOptions = $derived([
    ...(running || shownLogs.length
      ? [{ value: 'current', label: t('backfillPage.logCurrent') }]
      : []),
    ...runList.map((run) => ({
      value: String(run.id),
      label: t('backfillPage.logItem', {
        time: formatDateTime((run.started_at ?? 0) / 1000).slice(5),
        done: run.summary?.fetched ?? 0,
        total: run.summary?.total ?? 0,
      }),
    })),
  ]);

  const showLog = $derived(running || (!backfillStore.logHidden && !selected));
  const showPanel = $derived(
    running ||
      Boolean(selected) ||
      (!backfillStore.logHidden && (shownLogs.length > 0 || runList.length > 0)),
  );

  async function loadRuns() {
    try {
      runList = await api.backfillRuns(20);
    } catch {
      runList = [];
    }
  }

  async function pickRun(value) {
    runPick = value;
    if (value === 'current') {
      historyLogs = null;
      return;
    }
    historyLogs = [];
    try {
      const detail = await api.backfillRun(Number(value));
      historyLogs = (detail?.events ?? []).map((ev, i) => ({
        seq: i + 1,
        name: 'vndb-backfill',
        event: ev,
      }));
    } catch (e) {
      toast(e.message, 'error');
    }
  }

  $effect(() => {
    if (running) backfillStore.logHidden = false;
  });

  $effect(() => {
    if (running === wasRunning) return;
    wasRunning = running;
    if (running) {
      runPick = 'current';
      historyLogs = null;
      return;
    }
    loadRuns();
  });

  $effect(() => {
    if (autoPicked || runPick !== 'current' || running || shownLogs.length || !runList.length) {
      return;
    }
    autoPicked = true;
    pickRun(String(runList[0].id));
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
        {#if picked.length === 1}
          <span class="bf-shift-tip" data-tooltip={t('backfillPage.shiftHint')}>
            {t('backfillPage.shiftTip')}
          </span>
        {/if}
        {#if failedCount > 0}
          <button type="button" class="btn btn-ghost btn-sm" disabled={starting} onclick={retryFailed}>
            {t('backfillPage.retryFailed', { count: failedCount })}
          </button>
        {/if}
        <button
          type="button"
          class="btn btn-primary btn-sm"
          disabled={starting || (picked.length === 0 && focus.size === 0)}
          onclick={start}
        >
          {picked.length
            ? t('backfillPage.startPicked', { count: picked.length })
            : t('backfillPage.start')}
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
          {:else if stage === 'resolve'}
            {t('backfillPage.matchRunning', { done, total })}
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
            class:is-selected={pickedSet.has(it.game_id)}
            class:is-active={it.state === 'active'}
            aria-pressed={pickedSet.has(it.game_id)}
            disabled={running}
            onclick={(ev) => rowClick(it, ev)}
          >
            <span class="bf-row__dot {STATE_DOT[it.state]}"></span>
            <span class="bf-row__local" data-tooltip={it.name}>{it.name}</span>
            <span class="bf-row__bound" class:is-blank={!it.title}>
              <span class="bf-row__title" class:bf-row__title--muted={it.state === 'active'} data-tooltip={it.title}>
                {it.title}
              </span>
              <span class="bf-row__sub">
                {#if it.year}<span>{it.year}</span>{/if}
                {#if it.vndb_id}<span>{it.vndb_id}</span>{/if}
                {#if it.filled.length}<span>{t('backfillPage.filledCount', { count: it.filled.length })}</span>{/if}
                {#if it.bind_source === 'search'}
                  <span class="bf-row__src">
                    {t('backfillPage.sourceSearch')}{#if it.hit_count > 1} · {t('backfillPage.hits', { count: it.hit_count })}{/if}
                  </span>
                {:else if it.bind_source === 'existing'}
                  <span>{t('backfillPage.sourceExisting')}</span>
                {/if}
                {#if it.error}<span class="is-error">{it.error}</span>{/if}
              </span>
            </span>
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
                  {#if runOptions.length > 1}
                    <SelectDd
                      options={runOptions}
                      bind:value={runPick}
                      onchange={pickRun}
                      ariaLabel={t('backfillPage.logPick')}
                    />
                  {:else}
                    <span class="bf-panel__meta">{logRows.length}</span>
                  {/if}
                </span>
                {#if !running}
                  <button type="button" class="bf-panel__collapse" onclick={collapseLog}>
                    {t('backfillPage.logCollapse')}
                  </button>
                {/if}
              </div>
              <BackfillLog logs={displayLogs} status={runPick === 'current' ? status : null} />
            {:else}
              <BackfillPanel
                item={selected}
                {running}
                onchanged={mergeBackfillItem}
                onopenform={openForm}
                onopendetail={onopendetail}
                onclose={() => (backfillStore.selected = null)}
              />
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {/if}
</section>
