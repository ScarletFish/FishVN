<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { subscribe } from '../lib/events.js';

  let {
    open = $bindable(false),
    plan = null,
    loading = false,
    error = null,
    empty = false,
    onapply,
    oncancel,
    onretry,
  } = $props();

  const GROUP_ORDER = ['playing', 'finished', 'stalled', 'dropped', 'wishlist'];

  let choices = $state({});
  let picked = $state({});
  let backfill = $state(true);
  let phase = $state('fetch');
  let fetched = $state(null);
  let total = $state(null);
  let elapsed = $state(0);

  const title = $derived(
    plan
      ? t('settings.vndb.resultTitle')
      : error
        ? t('settings.vndb.syncFailedTitle')
        : empty
          ? t('settings.vndb.syncUpToDateTitle')
          : loading
            ? t('settings.vndb.syncProgressTitle')
            : t('settings.vndb.resultTitle'),
  );

  const phaseText = $derived(
    phase === 'plan' ? t('settings.vndb.syncPhasePlan') : t('settings.vndb.syncPhaseFetch'),
  );

  const percent = $derived(
    phase === 'fetch' && fetched != null && total
      ? Math.min(100, Math.round((fetched / total) * 100))
      : -1,
  );

  $effect(() => {
    if (!open || !plan) return;
    const next = {};
    for (const c of plan.conflicts) {
      if (c.status) next[`${c.game_id}:status`] = 'local';
      if (c.rating) next[`${c.game_id}:rating`] = 'local';
    }
    choices = next;
    const pickedNext = {};
    for (const c of plan.creates) pickedNext[c.vndb_id] = true;
    picked = pickedNext;
    backfill = true;
  });

  $effect(() => {
    if (!open || !loading) return;
    phase = 'fetch';
    fetched = null;
    total = null;
    elapsed = 0;
    let alive = true;
    let unsub = null;
    subscribe('vndb-sync-progress', (e) => {
      if (!alive) return;
      if (e.phase === 'plan') {
        phase = 'plan';
      } else {
        phase = 'fetch';
        fetched = e.done ?? null;
        total = e.total ?? null;
      }
    }).then((u) => {
      if (alive) unsub = u;
      else u();
    });
    const timer = setInterval(() => {
      elapsed += 1;
    }, 1000);
    return () => {
      alive = false;
      unsub?.();
      clearInterval(timer);
    };
  });

  function setAll(side) {
    const next = {};
    for (const c of plan.conflicts) {
      if (c.status) next[`${c.game_id}:status`] = side;
      if (c.rating) next[`${c.game_id}:rating`] = side;
    }
    choices = next;
  }

  function pick(gameId, field, side) {
    choices = { ...choices, [`${gameId}:${field}`]: side };
  }

  function toggleCreate(vndbId) {
    picked = { ...picked, [vndbId]: !picked[vndbId] };
  }

  function groupedCreates() {
    if (!plan) return [];
    const buckets = new Map(GROUP_ORDER.map((key) => [key, []]));
    for (const c of plan.creates) {
      if (!buckets.has(c.status)) buckets.set(c.status, []);
      buckets.get(c.status).push(c);
    }
    return GROUP_ORDER.filter((key) => buckets.get(key).length).map((key) => ({
      status: key,
      items: buckets.get(key),
    }));
  }

  function groupAllOn(items) {
    return items.every((g) => picked[g.vndb_id]);
  }

  function setGroup(items, on) {
    const next = { ...picked };
    for (const g of items) next[g.vndb_id] = on;
    picked = next;
  }

  function pickedCreates() {
    return plan.creates.filter((c) => picked[c.vndb_id]);
  }

  function hasApplicable() {
    if (!plan) return false;
    return (
      pickedCreates().length > 0 ||
      plan.updates.length > 0 ||
      plan.aligns.length > 0 ||
      plan.conflicts.length > 0
    );
  }

  function statusText(s) {
    return s ? t(`status.${s}`) : '—';
  }

  function ratingText(v) {
    return v == null ? t('settings.vndb.unrated') : String(Number(v.toFixed(2)));
  }

  function apply() {
    const statuses = [];
    const ratings = [];
    for (const c of plan.conflicts) {
      if (c.status) {
        const pickedSide =
          choices[`${c.game_id}:status`] === 'remote' ? c.status.remote : c.status.local;
        statuses.push({ game_id: c.game_id, status: pickedSide, synced: c.status.remote });
      }
      if (c.rating) {
        const pickedSide =
          choices[`${c.game_id}:rating`] === 'remote' ? c.rating.remote : c.rating.local;
        ratings.push({ game_id: c.game_id, rating: pickedSide, synced: c.rating.remote });
      }
    }
    onapply?.({
      creates: pickedCreates(),
      updates: plan.updates,
      aligns: plan.aligns,
      statuses,
      ratings,
      backfill,
    });
  }

  function summaryParts() {
    if (!plan) return [];
    const parts = [t('settings.vndb.resultTotal', { count: plan.total })];
    if (plan.conflicts.length > 0) {
      parts.push(t('settings.vndb.resultConflicts', { count: plan.conflicts.length }));
    }
    if (plan.creates.length > 0) {
      parts.push(
        t('settings.vndb.resultCreatedPicked', {
          picked: pickedCreates().length,
          total: plan.creates.length,
        }),
      );
    }
    if (plan.updates.length > 0) {
      parts.push(t('settings.vndb.resultUpdated', { count: plan.updates.length }));
    }
    if (plan.skipped > 0) {
      parts.push(t('settings.vndb.resultSkipped', { count: plan.skipped }));
    }
    return parts;
  }
</script>

<Modal {open} {title} variant="modal--result" onclose={oncancel}>
  {#if loading && !plan && !error && !empty}
    <div class="sync-progress">
      <div class="sync-progress__spinner" aria-hidden="true"></div>
      {#key phase}
        <p class="sync-progress__phase">{phaseText}</p>
      {/key}
      <div class="sync-progress__track" class:sync-progress__track--indet={percent < 0}>
        <div class="sync-progress__fill" style="width: {percent < 0 ? 40 : percent}%"></div>
      </div>
      <p class="sync-progress__meta">
        {#if fetched != null && total}
          <span>{t('settings.vndb.syncFetched', { fetched, total })}</span>
          <span class="sync-progress__sep">·</span>
        {/if}
        <span>{t('settings.vndb.syncElapsed', { secs: elapsed })}</span>
      </p>
      <p class="form-hint">{t('settings.vndb.syncProgressHint')}</p>
    </div>
  {:else if plan}
    {@const parts = summaryParts()}
    <div class="sync-result sync-result--enter" use:panelScrollbars>
      <p class="sync-result__summary">{parts.join(' | ')}</p>
      <p class="form-hint">{t('settings.vndb.resultPlanHint')}</p>

      {#if plan.conflicts.length > 0}
        <section class="sync-result__section">
          <div class="form-label">
            {t('settings.vndb.resultConflicts', { count: plan.conflicts.length })}
          </div>
          <p class="form-hint">{t('settings.vndb.conflictHint')}</p>
          <div class="sync-conflict__bulk">
            <button type="button" class="btn" onclick={() => setAll('local')}>
              {t('settings.vndb.keepAll')}
            </button>
            <button type="button" class="btn" onclick={() => setAll('remote')}>
              {t('settings.vndb.adoptAll')}
            </button>
          </div>
          <div class="sync-conflict__scroll">
            {#each plan.conflicts as c (c.game_id)}
              <div class="sync-conflict__game">
                <div class="sync-conflict__name">{c.name}</div>
                {#if c.status}
                  <div class="sync-conflict__field">
                    <span class="sync-conflict__label">{t('settings.vndb.statusField')}</span>
                    <button
                      type="button"
                      class="sync-conflict__side"
                      class:is-on={choices[`${c.game_id}:status`] === 'local'}
                      onclick={() => pick(c.game_id, 'status', 'local')}
                    >
                      <i>{t('settings.vndb.localSide')}</i>
                      <b>{statusText(c.status.local)}</b>
                    </button>
                    <button
                      type="button"
                      class="sync-conflict__side"
                      class:is-on={choices[`${c.game_id}:status`] === 'remote'}
                      onclick={() => pick(c.game_id, 'status', 'remote')}
                    >
                      <i>{t('settings.vndb.remoteSide')}</i>
                      <b>{statusText(c.status.remote)}</b>
                    </button>
                  </div>
                {/if}
                {#if c.rating}
                  <div class="sync-conflict__field">
                    <span class="sync-conflict__label">{t('settings.vndb.ratingField')}</span>
                    <button
                      type="button"
                      class="sync-conflict__side"
                      class:is-on={choices[`${c.game_id}:rating`] === 'local'}
                      onclick={() => pick(c.game_id, 'rating', 'local')}
                    >
                      <i>{t('settings.vndb.localSide')}</i>
                      <b>{ratingText(c.rating.local)}</b>
                    </button>
                    <button
                      type="button"
                      class="sync-conflict__side"
                      class:is-on={choices[`${c.game_id}:rating`] === 'remote'}
                      onclick={() => pick(c.game_id, 'rating', 'remote')}
                    >
                      <i>{t('settings.vndb.remoteSide')}</i>
                      <b>{ratingText(c.rating.remote)}</b>
                    </button>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
          <p class="form-hint">{t('settings.vndb.resultCancelHint')}</p>
        </section>
      {/if}

      {#if plan.creates.length > 0}
        {@const groups = groupedCreates()}
        <section class="sync-result__section">
          {#each groups as group (group.status)}
            <div class="sync-result__group">
              <div class="sync-result__group-head">
                <label class="sync-result__group-pick">
                  <input
                    type="checkbox"
                    class="checkbox"
                    checked={groupAllOn(group.items)}
                    onchange={() => setGroup(group.items, !groupAllOn(group.items))}
                  />
                  <span class="sync-result__group-name">{statusText(group.status)}</span>
                  <span class="sync-result__group-num">{group.items.length}</span>
                </label>
              </div>
              <ul class="sync-result__chips">
                {#each group.items as g (g.vndb_id)}
                  <li>
                    <button
                      type="button"
                      class="tag-pill tag-pill--pick sync-chip"
                      class:sync-chip--ghost={!picked[g.vndb_id]}
                      aria-pressed={!!picked[g.vndb_id]}
                      onclick={() => toggleCreate(g.vndb_id)}
                    >
                      {g.name}
                    </button>
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
          <label class="sync-result__backfill">
            <input type="checkbox" class="checkbox" bind:checked={backfill} />
            <span>{t('settings.vndb.fetchMeta')}</span>
          </label>
          <p class="form-hint">{t('settings.vndb.fetchMetaHint')}</p>
        </section>
      {/if}

      {#if plan.skipped > 0}
        <section class="sync-result__section">
          <p class="form-hint">{t('settings.vndb.resultSkipped', { count: plan.skipped })}</p>
          <p class="form-hint">{t('settings.vndb.resultSkippedHint')}</p>
        </section>
      {/if}

      {#each plan.warnings as w, i (i)}
        <p class="form-hint">{w}</p>
      {/each}
    </div>
  {:else if error}
    <div class="sync-error">
      <p class="sync-error__msg">{error}</p>
      <p class="form-hint">{t('settings.vndb.syncErrorHint')}</p>
    </div>
  {:else if empty}
    <div class="sync-empty">
      <div class="sync-empty__mark" aria-hidden="true">
        <svg viewBox="0 0 24 24"><path d="M4.5 12.5L10 18L19.5 6.5" /></svg>
      </div>
      <p class="sync-empty__text">{t('settings.vndb.syncUpToDate')}</p>
      <p class="form-hint">{t('settings.vndb.syncUpToDateHint')}</p>
    </div>
  {/if}

  <div class="modal-actions">
    {#if loading && !plan && !error && !empty}
      <button type="button" class="btn" onclick={oncancel}>
        {t('common.cancel')}
      </button>
    {:else if plan && hasApplicable()}
      <button type="button" class="btn" onclick={oncancel}>
        {t('common.cancel')}
      </button>
      <button type="button" class="btn btn-primary" onclick={apply}>
        {t('settings.vndb.applyPlan')}
      </button>
    {:else if error}
      <button type="button" class="btn" onclick={oncancel}>
        {t('common.close')}
      </button>
      <button type="button" class="btn btn-primary" onclick={onretry}>
        {t('settings.vndb.syncRetry')}
      </button>
    {:else}
      <button type="button" class="btn btn-primary" onclick={oncancel}>
        {t('settings.vndb.resultDone')}
      </button>
    {/if}
  </div>
</Modal>
