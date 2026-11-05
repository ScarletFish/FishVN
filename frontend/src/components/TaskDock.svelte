<script>
  import { onMount, onDestroy } from 'svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { t } from '../lib/i18n.js';
  import {
    notifyLibraryChanged,
    notifyLibraryItemUpdated,
    notifyLibraryTagsChanged,
    notifyLibraryTraitsChanged,
  } from '../lib/library-events.js';
  import { portal } from '../lib/portal.js';
  import { BACKFILL_REFRESH } from '../lib/task-events.js';
  import { toast } from '../lib/toast.svelte.js';
  import { backfillSummaryText } from '../lib/backfill-summary.js';
  import { createDockState, reduce, seedFromStatus, markFinished, dockView } from '../lib/dock-state.js';

  let { hidden = false, visible = $bindable(false), onopen } = $props();

  let collapsedRun = $state(0);
  let closedRun = $state(0);
  let dock = $state(createDockState());
  let unsub = null;
  let hideTimer = null;

  let view = $derived(dockView(dock, { collapsedRun, closedRun, hidden }, false));

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
    }
    const prevDone = freshRun ? 0 : dock.done;
    dock = reduce(dock, ev);
    if (ev?.phase === 'backfill' && dock.done > prevDone) notifyLibraryItemUpdated(ev.game_id);
    if (ev?.phase === 'finished') onFinished();
  }

  async function onFinished() {
    try {
      const s = await api.backfillStatus();
      if (s?.finished) {
        dock = markFinished(dock, s.finished);
        notifyLibraryChanged();
        notifyLibraryTagsChanged();
        notifyLibraryTraitsChanged();
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
    await syncStatus();
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

  function open() {
    clearHideTimer();
    collapsedRun = dock.runId;
    if (!dock.running) closedRun = dock.runId;
    onopen?.();
  }

  function handleText() {
    if (!dock.running) return t('backfillPage.doneHint');
    if (dock.pausedIdle) {
      return `${t('mylist.backfillPaused')} · ${dock.done ?? 0}/${dock.total ?? 0}`;
    }
    if (dock.rateLimited) return t('mylist.backfillRateLimitedShort');
    return t('mylist.backfillRunning', { done: dock.done ?? 0, total: dock.total ?? 0 });
  }
</script>

{#if view.form !== 'hidden'}
  <div
    class="task-dock task-dock--handle"
    use:portal
    transition:fly={{ y: 8, duration: 200, easing: cubicOut }}
  >
    <button
      type="button"
      class="task-dock__handle"
      aria-label={t('mylist.backfillShow')}
      onclick={open}
    >
      <span
        class="task-dock__dot"
        class:is-paused={dock.pausedIdle}
        class:is-done={!dock.running}
      ></span>
      <span class="task-dock__handle-text">{handleText()}</span>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="m9 18 6-6-6-6"></path>
      </svg>
    </button>
  </div>
{/if}
