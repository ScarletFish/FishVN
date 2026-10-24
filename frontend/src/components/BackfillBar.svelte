<script>
  import { onMount, onDestroy } from 'svelte';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { t } from '../lib/i18n.js';

  let { ondone, running = $bindable(false) } = $props();

  let dismissed = false;
  let status = $state(null);
  let unsub = null;

  onMount(async () => {
    try {
      status = await api.backfillStatus();
      if (status?.finished && dismissed) status = null;
      running = status?.running ?? false;
    } catch {
      status = null;
    }
    unsub = await subscribe('vndb-backfill', onEvent);
  });

  onDestroy(() => unsub?.());

  function onEvent(ev) {
    if (!status) status = {};
    if (ev.phase === 'backfill') {
      status = {
        ...status,
        running: true,
        done: ev.done ?? status.done ?? 0,
        total: ev.total ?? status.total ?? 0,
        rate_limited: false,
      };
      running = true;
    } else if (ev.phase === 'rate-limited') {
      status = { ...status, running: true, rate_limited: true };
      running = true;
    } else if (ev.phase === 'unmatched') {
      status = { ...status, running: true, unmatched: (status.unmatched ?? 0) + 1 };
      running = true;
    } else if (ev.phase === 'failed') {
      status = { ...status, running: true, failed: (status.failed ?? 0) + 1 };
      running = true;
    } else if (ev.phase === 'finished') {
      refresh();
    }
  }

  async function refresh() {
    try {
      status = await api.backfillStatus();
    } catch {
      status = null;
    }
    dismissed = false;
    running = status?.running ?? false;
    ondone?.();
  }

  async function togglePause() {
    try {
      status = status?.paused ? await api.resumeBackfill() : await api.pauseBackfill();
    } catch {
      status = await api.backfillStatus().catch(() => status);
    }
  }

  function dismiss() {
    dismissed = true;
    status = null;
  }

  function percent() {
    const total = status?.total ?? 0;
    if (total <= 0) return 0;
    return Math.round(((status?.done ?? 0) / total) * 100);
  }

  function summaryParts() {
    const r = status?.finished;
    if (!r) return [];
    const parts = [t('mylist.backfillFetched', { count: r.fetched })];
    if (r.bound > 0) parts.push(t('mylist.backfillBound', { count: r.bound }));
    if (r.unmatched > 0) parts.push(t('mylist.backfillUnmatched', { count: r.unmatched }));
    if (r.failed > 0) parts.push(t('mylist.backfillFailed', { count: r.failed }));
    return parts;
  }
</script>

{#if status && (status.running || status.finished)}
  <div class="backfill-bar">
    {#if status.running}
      <div class="backfill-bar__track" class:is-paused={status.paused}>
        <div class="backfill-bar__fill" style="width: {percent()}%"></div>
      </div>
      <span class="backfill-bar__text">
        {#if status.paused}
          {t('mylist.backfillPaused')}
        {:else}
          {t('mylist.backfillRunning', { done: status.done ?? 0, total: status.total ?? 0 })}
        {/if}
        {#if (status.unmatched ?? 0) > 0}
          <span class="backfill-bar__sep"> | </span>{t('mylist.backfillUnmatched', { count: status.unmatched })}
        {/if}
      </span>
      {#if status.rate_limited}
        <span class="backfill-bar__warn">{t('mylist.backfillRateLimited')}</span>
      {/if}
      <button type="button" class="backfill-bar__btn" onclick={togglePause}>
        {status.paused ? t('mylist.backfillResume') : t('mylist.backfillPause')}
      </button>
      <button type="button" class="backfill-bar__btn" onclick={() => api.cancelTask('vndb-backfill')}>
        {t('mylist.backfillCancel')}
      </button>
    {:else}
      <span class="backfill-bar__text">{summaryParts().join(' | ')}</span>
      <button type="button" class="backfill-bar__btn" onclick={dismiss}>
        {t('mylist.backfillClose')}
      </button>
    {/if}
  </div>
{/if}
