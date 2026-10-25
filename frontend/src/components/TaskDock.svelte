<script>
  import { onMount, onDestroy } from 'svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { t } from '../lib/i18n.js';
  import { notifyLibraryChanged, notifyLibraryItemUpdated, notifyLibraryTagsChanged } from '../lib/library-events.js';
  import { portal } from '../lib/portal.js';
  import { BACKFILL_REFRESH } from '../lib/task-events.js';
  import { backfillSummaryText } from '../lib/backfill-summary.js';
  import { toast } from '../lib/toast.svelte.js';
  import {
    PHASE_I18N,
    createDockState,
    reduce,
    seedFromStatus,
    markFinished,
    dockView,
    currentIndex,
  } from '../lib/dock-state.js';

  let { hidden = false, visible = $bindable(false) } = $props();

  let collapsedRun = $state(0);
  let closedRun = $state(0);
  let dock = $state(createDockState());
  let cancelling = $state(false);
  let errorsOpen = $state(false);
  let unsub = null;
  let hideTimer = null;

  let view = $derived(dockView(dock, { collapsedRun, closedRun, hidden }, cancelling));
  let current = $derived(currentIndex(dock));

  $effect(() => {
    visible = view.form !== 'hidden';
  });

  onMount(async () => {
    await syncStatus();
    unsub = await subscribe('vndb-backfill', onEvent);
    window.addEventListener(BACKFILL_REFRESH, onRefreshEvent);
  });

  onDestroy(() => {
    unsub?.();
    window.removeEventListener(BACKFILL_REFRESH, onRefreshEvent);
    clearHideTimer();
  });

  function onRefreshEvent() {
    refresh();
  }

  function onEvent(ev) {
    const freshRun = Boolean(ev?.run_id) && ev.run_id !== dock.runId;
    if (freshRun) {
      collapsedRun = 0;
      closedRun = 0;
      errorsOpen = false;
    }
    const prevDone = freshRun ? 0 : dock.done;
    dock = reduce(dock, ev);
    if (ev?.phase === 'backfill' && dock.done > prevDone) notifyLibraryItemUpdated(ev.game_id);
    if (ev?.phase === 'finished') onFinished();
  }

  async function onFinished() {
    cancelling = false;
    try {
      const s = await api.backfillStatus();
      if (s?.finished) {
        dock = markFinished(dock, s.finished);
        notifyLibraryChanged();
        notifyLibraryTagsChanged();
        if (hidden) toast(backfillSummaryText(s.finished), 'success');
        else if (closedRun !== dock.runId) armHideTimer();
      }
    } catch {}
  }

  async function syncStatus() {
    try {
      const s = await api.backfillStatus();
      if (s?.running || s?.finished) {
        dock = seedFromStatus(dock, s);
        if (s.finished) armHideTimer();
        else clearHideTimer();
      } else {
        dock = createDockState();
      }
    } catch {}
  }

  async function refresh() {
    collapsedRun = 0;
    closedRun = 0;
    errorsOpen = false;
    await syncStatus();
  }

  async function togglePause() {
    try {
      const s = dock.paused ? await api.resumeBackfill() : await api.pauseBackfill();
      if (s) dock = { ...dock, paused: s.paused ?? false, pausedIdle: s.paused_idle ?? false };
    } catch {
      const s = await api.backfillStatus().catch(() => null);
      if (s) dock = { ...dock, paused: s.paused ?? false, pausedIdle: s.paused_idle ?? false };
    }
  }

  async function cancelTask() {
    cancelling = true;
    try {
      const stopped = await api.cancelTask('vndb-backfill');
      if (!stopped) cancelling = false;
    } catch {
      cancelling = false;
    }
  }

  function dismiss() {
    if (dock.running) {
      collapsedRun = dock.runId;
    } else if (dock.finished) {
      closedRun = dock.runId;
    }
    errorsOpen = false;
    clearHideTimer();
  }

  function expand() {
    if (dock.running) collapsedRun = 0;
    else if (dock.finished) closedRun = 0;
  }

  function armHideTimer() {
    clearHideTimer();
    hideTimer = setTimeout(autoHide, 6000);
  }

  function clearHideTimer() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }

  function autoHide() {
    hideTimer = null;
    closedRun = dock.runId;
    collapsedRun = 0;
  }

  function percent() {
    const total = dock.total ?? 0;
    if (total <= 0) return 0;
    return Math.round(((dock.done ?? 0) / total) * 100);
  }

  function phaseLabel() {
    return t(PHASE_I18N[dock.phase] ?? 'mylist.backfillPhaseStart');
  }

  function handleText() {
    if (dock.paused) {
      return dock.pausedIdle
        ? `${t('mylist.backfillPaused')} · ${dock.done ?? 0}/${dock.total ?? 0}`
        : t('mylist.backfillPausing');
    }
    if (dock.rateLimited) return t('mylist.backfillRateLimitedShort');
    return t('mylist.backfillRunning', { done: dock.done ?? 0, total: dock.total ?? 0 });
  }
</script>

{#snippet errorList()}
  <ul class="task-dock__errors scroll-thin">
    {#each dock.errors as e, i (i)}
      <li>{e.message}{e.times > 1 ? ` ×${e.times}` : ''}</li>
    {/each}
    {#if dock.overflow > 0}
      <li>{t('mylist.backfillErrorsMore', { count: dock.overflow })}</li>
    {/if}
  </ul>
{/snippet}

{#if view.form !== 'hidden'}
  <div
    class="task-dock"
    class:task-dock--handle={view.form === 'handle'}
    use:portal
    transition:fly={{ y: 8, duration: 200, easing: cubicOut }}
  >
    {#if view.form === 'handle'}
      <button
        type="button"
        class="task-dock__handle"
        aria-label={t('mylist.backfillShow')}
        onclick={expand}
      >
        <span class="task-dock__dot" class:is-paused={dock.pausedIdle}></span>
        <span class="task-dock__handle-text">{handleText()}</span>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="m18 15-6-6-6 6"></path>
        </svg>
      </button>
    {:else}
      <button
        type="button"
        class="task-dock__close"
        aria-label={t('mylist.backfillClose')}
        onclick={dismiss}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 6 6 18M6 6l12 12"></path>
        </svg>
      </button>

      {#if view.variant === 'summary'}
        <div class="task-dock__body">
          <span class="task-dock__text" role="status" aria-live="polite">
            {backfillSummaryText(dock.finished)}
          </span>
          {#if dock.errors.length}
            <button
              type="button"
              class="task-dock__err-toggle"
              aria-expanded={errorsOpen}
              onclick={() => (errorsOpen = !errorsOpen)}
            >
              {t('mylist.backfillErrorsN', { count: dock.errorCount })}
            </button>
          {/if}
          {#if errorsOpen && dock.errors.length}
            {@render errorList()}
          {/if}
        </div>
      {:else}
        <div class="task-dock__track" class:is-paused={dock.pausedIdle}>
          <div class="task-dock__fill" style="width: {percent()}%"></div>
        </div>
        <div class="task-dock__body">
          <div class="task-dock__status">
            <span class="task-dock__status-text" role="status" aria-live="polite">
              {#if view.variant === 'cancelling'}
                {t('mylist.backfillCancelling')}
              {:else if view.variant === 'pausing'}
                {t('mylist.backfillPausing')}
              {:else if view.variant === 'paused'}
                {t('mylist.backfillPaused')}
              {:else}
                {phaseLabel()} · {t('mylist.backfillProcessing', { current, total: dock.total ?? 0 })}
              {/if}
              {#if dock.rateLimited}
                <span class="task-dock__warn">{t('mylist.backfillRateLimited')}</span>
              {/if}
            </span>
            {#if dock.errors.length}
              <button
                type="button"
                class="task-dock__err-toggle"
                aria-expanded={errorsOpen}
                onclick={() => (errorsOpen = !errorsOpen)}
              >
                {t('mylist.backfillErrorsN', { count: dock.errorCount })}
              </button>
            {/if}
          </div>
          {#if errorsOpen && dock.errors.length}
            {@render errorList()}
          {/if}
          <div class="task-dock__actions">
            <button
              type="button"
              class="task-dock__btn"
              disabled={view.variant === 'cancelling'}
              onclick={togglePause}
            >
              {dock.paused ? t('mylist.backfillResume') : t('mylist.backfillPause')}
            </button>
            <button
              type="button"
              class="task-dock__btn task-dock__btn--danger"
              disabled={view.variant === 'cancelling'}
              onclick={cancelTask}
            >
              {t('mylist.backfillCancel')}
            </button>
          </div>
        </div>
      {/if}
    {/if}
  </div>
{/if}
