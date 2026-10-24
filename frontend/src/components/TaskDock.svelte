<script>
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { t } from '../lib/i18n.js';
  import { notifyLibraryChanged } from '../lib/library-events.js';
  import { portal } from '../lib/portal.js';
  import { BACKFILL_REFRESH } from '../lib/task-events.js';
  import { backfillSummaryText } from '../lib/backfill-summary.js';
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
    try {
      const s = await api.backfillStatus();
      if (s?.running) {
        dock = seedFromStatus(dock, s);
      } else {
        dock = createDockState();
      }
    } catch {
      dock = createDockState();
    }
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
    if (ev?.run_id && ev.run_id !== dock.runId) {
      collapsedRun = 0;
      closedRun = 0;
      errorsOpen = false;
    }
    dock = reduce(dock, ev);
    if (ev?.phase === 'finished') onFinished();
  }

  async function onFinished() {
    cancelling = false;
    try {
      const s = await api.backfillStatus();
      if (s?.finished) {
        dock = markFinished(dock, s.finished);
        notifyLibraryChanged();
        if (closedRun !== dock.runId) armHideTimer();
      }
    } catch {
    }
  }

  async function refresh() {
    collapsedRun = 0;
    closedRun = 0;
    errorsOpen = false;
    try {
      const s = await api.backfillStatus();
      if (s?.running || s?.finished) {
        dock = seedFromStatus(dock, s);
        if (s.finished) armHideTimer();
        else clearHideTimer();
      } else {
        dock = createDockState();
      }
    } catch {
      dock = createDockState();
    }
  }

  async function togglePause() {
    try {
      const s = dock.paused ? await api.resumeBackfill() : await api.pauseBackfill();
      if (s) dock = { ...dock, paused: s.paused ?? false };
    } catch {
      const s = await api.backfillStatus().catch(() => null);
      if (s) dock = { ...dock, paused: s.paused ?? false };
    }
  }

  function cancelTask() {
    cancelling = true;
    api.cancelTask('vndb-backfill').catch(() => {
      cancelling = false;
    });
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
    return dock.phase ? t(PHASE_I18N[dock.phase]) : t('mylist.backfillPhaseStart');
  }
</script>

{#if view.form !== 'hidden'}
  <div
    class="task-dock"
    class:task-dock--handle={view.form === 'handle'}
    role={view.form === 'panel' ? 'status' : undefined}
    aria-live={view.form === 'panel' ? 'polite' : undefined}
    use:portal
    transition:fade={{ duration: 160 }}
  >
    {#if view.form === 'handle'}
      <button
        type="button"
        class="task-dock__handle"
        aria-label={t('mylist.backfillShow')}
        onclick={expand}
      >
        <span class="task-dock__dot" class:is-paused={dock.paused}></span>
        <span class="task-dock__handle-text">
          {t('mylist.backfillRunning', { done: dock.done ?? 0, total: dock.total ?? 0 })}
        </span>
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
          <span class="task-dock__text">{backfillSummaryText(dock.finished)}</span>
          {#if dock.errors.length}
            <button
              type="button"
              class="task-dock__err-toggle"
              aria-expanded={errorsOpen}
              onclick={() => (errorsOpen = !errorsOpen)}
            >
              {t('mylist.backfillErrorsN', { count: dock.errors.length })}
            </button>
          {/if}
          {#if errorsOpen && dock.errors.length}
            <ul class="task-dock__errors">
              {#each dock.errors as e (e.message)}
                <li>{e.message}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else}
        <div class="task-dock__track" class:is-paused={dock.paused}>
          <div class="task-dock__fill" style="width: {percent()}%"></div>
        </div>
        <div class="task-dock__body">
          <div class="task-dock__status">
            {#if view.variant === 'cancelling'}
              {t('mylist.backfillCancelling')}
            {:else if view.variant === 'paused'}
              {t('mylist.backfillPaused')}
            {:else}
              {phaseLabel()} · {t('mylist.backfillProcessing', { current, total: dock.total ?? 0 })}
            {/if}
            {#if dock.rateLimited}
              <span class="task-dock__warn">{t('mylist.backfillRateLimited')}</span>
            {/if}
            {#if dock.errors.length}
              <button
                type="button"
                class="task-dock__err-toggle"
                aria-expanded={errorsOpen}
                onclick={() => (errorsOpen = !errorsOpen)}
              >
                {t('mylist.backfillErrorsN', { count: dock.errors.length })}
              </button>
            {/if}
          </div>
          {#if errorsOpen && dock.errors.length}
            <ul class="task-dock__errors">
              {#each dock.errors as e (e.message)}
                <li>{e.message}</li>
              {/each}
            </ul>
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
