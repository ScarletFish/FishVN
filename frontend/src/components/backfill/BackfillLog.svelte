<script>
  import { t } from '../../lib/i18n.js';
  import { backfillSummaryParts } from '../../lib/backfill-summary.js';
  import { mergeProgress } from '../../lib/backfill-log.js';

  let { logs = [], status = null } = $props();

  const PHASE = {
    resolve: 'backfillPage.logResolve',
    backfill: 'backfillPage.logDone',
    unmatched: 'backfillPage.logUnmatched',
    failed: 'backfillPage.logFailed',
    finished: 'backfillPage.logSummary',
    'rate-limited': 'backfillPage.logRateLimited',
    paused: 'backfillPage.paused',
    meta: 'mylist.backfillPhaseMeta',
    cover: 'mylist.backfillPhaseCover',
    characters: 'mylist.backfillPhaseCharacters',
    images: 'mylist.backfillPhaseImages',
  };

  const names = $derived(
    new Map((status?.items ?? []).map((it) => [it.game_id, it.name])),
  );

  const rows = $derived(mergeProgress(logs));

  let listEl = $state(null);

  $effect(() => {
    const n = logs.length;
    if (!listEl || !n) return;
    if (listEl.scrollHeight - listEl.scrollTop - listEl.clientHeight < 60) {
      listEl.scrollTop = listEl.scrollHeight;
    }
  });

  function clock(ts) {
    if (!ts) return '';
    return new Date(ts).toLocaleTimeString('zh-CN', { hour12: false });
  }

  function textOf(ev) {
    if (ev.phase === 'finished') return '';
    if (ev.phase === 'backfill') return '';
    if (ev.detail) return ev.detail;
    if (ev.error) return ev.error;
    if (ev.total) return `${ev.done ?? 0}/${ev.total}`;
    return '';
  }

  const summaryParts = $derived(backfillSummaryParts(status?.finished));
</script>

<ol class="bf-loglist scroll-thin" bind:this={listEl}>
  {#each rows as row (row.seq)}
    {@const ev = row.event}
    {@const detail = textOf(ev)}
    <li class="bf-logrow" class:is-error={Boolean(ev.error)}>
      <span class="bf-logrow__time">{clock(ev.ts)}</span>
      <span class="bf-logrow__body">
        {#if names.get(ev.game_id)}
          <span class="bf-logrow__name" data-tooltip={names.get(ev.game_id)}>
            {names.get(ev.game_id)}
          </span>
        {/if}
        <span class="bf-logrow__text">
          <span class="bf-logrow__phase">{t(PHASE[ev.phase] ?? 'backfillPage.logOther')}</span>
          {#if ev.phase === 'finished'}
            {#if summaryParts.length}
              <span class="bf-logrow__sep">·</span>
              <span class="bf-logrow__parts">
                {#each summaryParts as part, i}
                  <span class="bf-logrow__part"
                    >{i < summaryParts.length - 1 ? `${part}，` : part}</span
                  >
                {/each}
              </span>
            {/if}
          {:else if detail}
            <span class="bf-logrow__sep">·</span>{detail}
          {/if}
        </span>
      </span>
    </li>
  {:else}
    <li class="bf-logrow__empty">{t('backfillPage.logEmpty')}</li>
  {/each}
</ol>
