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
  let pushedOn = $state({});
  let step = $state('apply');
  let applying = $state(false);
  let pushDone = $state(0);
  let pushTotal = $state(0);
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
      const side = c.prefer === 'remote' ? 'remote' : 'local';
      if (c.status) next[`${c.game_id}:status`] = side;
      if (c.rating) next[`${c.game_id}:rating`] = side;
    }
    choices = next;
    const pickedNext = {};
    for (const c of plan.creates) pickedNext[c.vndb_id] = true;
    picked = pickedNext;
    const pushNext = {};
    for (const p of plan.pushes) pushNext[p.game_id] = true;
    pushedOn = pushNext;
    step = plan.conflicts.length > 0 ? 'conflicts' : 'apply';
    applying = false;
    pushDone = 0;
    pushTotal = 0;
  });

  $effect(() => {
    if (!open || !applying) return;
    let alive = true;
    let unsub = null;
    subscribe('vndb-sync-progress', (e) => {
      if (!alive || e.phase !== 'push') return;
      pushDone = e.done ?? 0;
      pushTotal = e.total ?? 0;
    }).then((u) => {
      if (alive) unsub = u;
      else u();
    });
    return () => {
      alive = false;
      unsub?.();
    };
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

  function pushedPushes() {
    return (plan?.pushes ?? []).filter((p) => pushedOn[p.game_id] !== false);
  }

  function togglePush(gameId) {
    pushedOn = { ...pushedOn, [gameId]: pushedOn[gameId] === false };
  }

  function uploads() {
    return plan?.mode === 'upload' || plan?.mode === 'both';
  }

  function conflictPushes() {
    if (!plan || !uploads()) return [];
    return plan.conflicts.filter((c) => {
      const status = c.status && choices[`${c.game_id}:status`] !== 'remote';
      const rating = c.rating && choices[`${c.game_id}:rating`] !== 'remote';
      return status || rating;
    });
  }

  function uploadCount() {
    return pushedPushes().length + conflictPushes().length;
  }

  function willUpload() {
    return uploadCount() > 0;
  }

  function hasApplicable() {
    if (!plan) return false;
    return (
      pickedCreates().length > 0 ||
      plan.updates.length > 0 ||
      plan.aligns.length > 0 ||
      plan.conflicts.length > 0 ||
      pushedPushes().length > 0
    );
  }

  function statusText(s) {
    return s ? t(`status.${s}`) : '—';
  }

  function ratingText(v) {
    return v == null ? t('settings.vndb.unrated') : String(Number(v.toFixed(2)));
  }

  function ratingNote(p) {
    if (p.remote_status == null) {
      return `${t('settings.vndb.ratingField')} ${ratingText(p.rating)}`;
    }
    return `${ratingText(p.remote_rating)} → ${ratingText(p.rating)}`;
  }

  function pushSections() {
    const fresh = new Map();
    const changed = new Map();
    const rated = [];
    const bucket = (map, key, item) => {
      if (!map.has(key)) map.set(key, []);
      map.get(key).push(item);
    };
    for (const p of plan.pushes) {
      if (p.remote_status == null) bucket(fresh, p.status, p);
      else if (p.status) bucket(changed, p.remote_status, p);
      else if (p.rating !== undefined) rated.push(p);
    }
    const ordered = (buckets) =>
      GROUP_ORDER.filter((key) => buckets.get(key)?.length).map((key) => ({
        status: key,
        items: buckets.get(key),
      }));
    return { fresh: ordered(fresh), changed: ordered(changed), rated };
  }

  function when(ts) {
    if (!ts) return '—';
    const d = new Date(ts * 1000);
    const pad = (n) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  async function apply() {
    if (applying) return;
    const statuses = [];
    const ratings = [];
    const pushes = [...pushedPushes()];
    for (const c of plan.conflicts) {
      const statusSide = choices[`${c.game_id}:status`];
      const ratingSide = choices[`${c.game_id}:rating`];
      if (c.status) {
        if (statusSide === 'remote') {
          statuses.push({ game_id: c.game_id, status: c.status.remote, synced: c.status.remote });
        } else if (!uploads()) {
          statuses.push({ game_id: c.game_id, status: c.status.local, synced: c.status.remote });
        }
      }
      if (c.rating) {
        if (ratingSide === 'remote') {
          ratings.push({ game_id: c.game_id, rating: c.rating.remote, synced: c.rating.remote });
        } else if (!uploads()) {
          ratings.push({ game_id: c.game_id, rating: c.rating.local, synced: c.rating.remote });
        }
      }
      if (!uploads()) continue;
      const push = { game_id: c.game_id, vndb_id: c.vndb_id, name: c.name };
      let any = false;
      if (c.status && statusSide !== 'remote') {
        push.status = c.status.local;
        any = true;
      }
      if (c.rating && ratingSide !== 'remote') {
        push.rating = c.rating.local;
        any = true;
      }
      if (any) pushes.push(push);
    }
    applying = true;
    pushDone = 0;
    pushTotal = pushes.length;
    try {
      await onapply?.({
        creates: pickedCreates(),
        updates: plan.updates,
        aligns: plan.aligns,
        statuses,
        ratings,
        pushes,
      });
    } finally {
      applying = false;
    }
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
    if (willUpload()) {
      parts.push(t('settings.vndb.resultPushed', { count: uploadCount() }));
    }
    if (plan.skipped > 0) {
      parts.push(t('settings.vndb.resultSkipped', { count: plan.skipped }));
    }
    return parts;
  }
</script>

<Modal {open} {title} variant="modal--result" onclose={applying ? () => false : oncancel}>
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
      <div class="sync-result__head">
        {#if plan.conflicts.length > 0}
          <p class="sync-result__step">
            {t('settings.vndb.stepOf', { current: step === 'conflicts' ? 1 : 2, total: 2 })}
          </p>
        {/if}
        <p class="sync-result__summary">{parts.join(' | ')}</p>
        <p class="form-hint">
          {t(step === 'conflicts' ? 'settings.vndb.stepConflicts' : 'settings.vndb.stepApply')}
        </p>
      </div>

      {#if step === 'conflicts'}
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
                <p class="sync-conflict__when">
                  {t('settings.vndb.conflictWhen', {
                    local: when(c.local_at),
                    remote: when(c.remote_at),
                  })}
                </p>
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
        </section>
      {/if}

      {#if step === 'apply' && plan.conflicts.length > 0}
        <section class="sync-result__section">
          <p class="form-hint">
            {t('settings.vndb.conflictsSettled', { count: plan.conflicts.length })}
          </p>
        </section>
      {/if}

      {#if step === 'apply' && plan.pushes.length > 0}
        {@const sections = pushSections()}
        <section class="sync-result__section">
          <div class="form-label">
            {t('settings.vndb.resultPushed', { count: plan.pushes.length })}
          </div>
          <p class="form-hint">{t('settings.vndb.pushHint')}</p>

          {#if sections.fresh.length > 0}
            <div class="sync-result__sub">{t('settings.vndb.pushFresh')}</div>
            {#each sections.fresh as group (group.status)}
              <div class="sync-result__group">
                <div class="sync-result__group-head">
                  <span class="sync-result__group-name">{statusText(group.status)}</span>
                  <span class="sync-result__group-num">{group.items.length}</span>
                </div>
                <ul class="sync-result__chips">
                  {#each group.items as p (p.game_id)}
                    <li>{@render pushChip(p)}</li>
                  {/each}
                </ul>
              </div>
            {/each}
          {/if}

          {#if sections.changed.length > 0}
            <div class="sync-result__sub">{t('settings.vndb.pushChanged')}</div>
            {#each sections.changed as group (group.status)}
              <div class="sync-result__group">
                <div class="sync-result__group-head">
                  <span class="sync-result__group-name">{statusText(group.status)}</span>
                  <span class="sync-result__group-num">{group.items.length}</span>
                </div>
                <ul class="sync-result__chips">
                  {#each group.items as p (p.game_id)}
                    <li>{@render pushChip(p)}</li>
                  {/each}
                </ul>
              </div>
            {/each}
          {/if}

          {#if sections.rated.length > 0}
            <div class="sync-result__sub">{t('settings.vndb.pushRating')}</div>
            <ul class="sync-result__chips">
              {#each sections.rated as p (p.game_id)}
                <li>{@render pushChip(p)}</li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}

      {#snippet pushChip(p)}
        <button
          type="button"
          class="tag-pill tag-pill--pick sync-chip"
          class:sync-chip--ghost={pushedOn[p.game_id] === false}
          aria-pressed={pushedOn[p.game_id] !== false}
          onclick={() => togglePush(p.game_id)}
        >
          {p.name}
          {#if p.status && p.remote_status != null}
            <span class="sync-chip__diff">→ {statusText(p.status)}</span>
          {/if}
          {#if p.rating !== undefined}
            <span class="sync-chip__diff">{ratingNote(p)}</span>
          {/if}
        </button>
      {/snippet}

      {#if step === 'apply' && plan.creates.length > 0}
        {@const groups = groupedCreates()}
        <section class="sync-result__section">
          <div class="form-label">
            {t('settings.vndb.resultCreatedPicked', {
              picked: pickedCreates().length,
              total: plan.creates.length,
            })}
          </div>
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
        </section>
      {/if}

      {#if step === 'apply' && plan.warnings.length > 0}
        <section class="sync-result__section">
          {#each plan.warnings as w, i (i)}
            <p class="form-hint">{w}</p>
          {/each}
        </section>
      {/if}
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
    {#if applying}
      <button type="button" class="btn btn-primary" disabled>
        {#if pushTotal > 0}
          {t('settings.vndb.applyingPush', { done: pushDone, total: pushTotal })}
        {:else}
          {t('settings.vndb.applying')}
        {/if}
      </button>
    {:else if loading && !plan && !error && !empty}
      <button type="button" class="btn" onclick={oncancel}>
        {t('common.cancel')}
      </button>
    {:else if plan && hasApplicable()}
      {#if step === 'conflicts'}
        <button type="button" class="btn" onclick={oncancel}>
          {t('common.cancel')}
        </button>
        <button type="button" class="btn btn-primary" onclick={() => (step = 'apply')}>
          {t('settings.vndb.next')}
        </button>
      {:else}
        {#if plan.conflicts.length > 0}
          <button type="button" class="btn" onclick={() => (step = 'conflicts')}>
            {t('settings.vndb.back')}
          </button>
        {:else}
          <button type="button" class="btn" onclick={oncancel}>
            {t('common.cancel')}
          </button>
        {/if}
        <button type="button" class="btn btn-primary" onclick={apply}>
          {willUpload() ? t('settings.vndb.applyPlanUpload') : t('settings.vndb.applyPlan')}
        </button>
      {/if}
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
