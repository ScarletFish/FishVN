<script>
  import { onMount, onDestroy } from 'svelte';
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { report } from '../lib/frontend-log.js';
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
  import { createDockState, reduce, seedFromStatus, markFinished, dockVisible } from '../lib/dock-state.js';

  let { hidden = false, visible = $bindable(false), onopen } = $props();

  let closedRun = $state(0);
  let dock = $state(createDockState());
  let unsub = null;
  let hideTimer = null;
  let seenItems = 0;
  let sentNotifies = 0;

  let shown = $derived(dockVisible(dock, { closedRun, hidden }));

  $effect(() => {
    visible = shown;
  });

  onMount(async () => {
    await syncStatus();
    try {
      unsub = await subscribe('vndb-backfill', onEvent);
    } catch (e) {
      report('error', `任务事件订阅失败：${e?.message ?? e}`);
    }
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
      closedRun = 0;
      seenItems = 0;
      sentNotifies = 0;
    }
    dock = reduce(dock, ev);
    if (ev?.phase === 'backfill') {
      seenItems += 1;
      sentNotifies += 1;
      report('debug', `任务事件：done=${ev.done} → 发出库变更通知 ${ev.game_id ?? ''}`);
      notifyLibraryItemUpdated(ev.game_id);
    }
    if (ev?.phase === 'finished') onFinished();
  }

  async function onFinished() {
    report('info', `补全收尾：收到 ${seenItems} 项事件，发出 ${sentNotifies} 次库变更通知`);
    try {
      const s = await api.backfillStatus();
      if (s?.finished) {
        dock = markFinished(dock, s.finished);
        notifyLibraryChanged();
        notifyLibraryTagsChanged();
        notifyLibraryTraitsChanged();
        if (hidden) {
          toast(backfillSummaryText(s.finished), s.finished.cancelled ? 'info' : 'success');
        } else if (s.finished.cancelled) {
          closedRun = dock.runId;
        } else if (closedRun !== dock.runId) {
          armHideTimer();
        }
      }
    } catch {}
  }

  async function syncStatus() {
    try {
      const s = await api.backfillStatus();
      if (s?.running || s?.finished) {
        dock = seedFromStatus(dock, s);
        if (s.finished && !s.finished.cancelled) armHideTimer();
        else clearHideTimer();
      } else {
        dock = createDockState();
      }
    } catch {}
  }

  async function refresh() {
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
  }

  function open() {
    clearHideTimer();
    if (!dock.running) closedRun = dock.runId;
    onopen?.();
  }

  function handleText() {
    if (!dock.running) return t('backfillPage.doneHint');
    if (dock.rateLimited) return t('mylist.backfillRateLimitedShort');
    return t('mylist.backfillRunning', { done: dock.done ?? 0, total: dock.total ?? 0 });
  }
</script>

{#if shown}
  <div
    class="task-dock"
    use:portal
    transition:fly={{ y: 8, duration: 200, easing: cubicOut }}
  >
    <button
      type="button"
      class="task-dock__handle"
      aria-label={t('mylist.backfillShow')}
      onclick={open}
    >
      <span class="task-dock__dot" class:is-done={!dock.running}></span>
      <span class="task-dock__handle-text">{handleText()}</span>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="m9 18 6-6-6-6"></path>
      </svg>
    </button>
  </div>
{/if}
