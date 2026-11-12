<script>
  import Icon from '../Icon.svelte';
  import { t } from '../../lib/i18n.js';
  import { api } from '../../lib/api.js';
  import { toast } from '../../lib/toast.svelte.js';
  import { createVndbSearch } from '../../lib/vndb-search.svelte.js';
  import { createVnMergeFlow } from '../../lib/vn-merge-flow.svelte.js';
  import { buildMetadataPatch } from '../../lib/form-metadata.js';
  import { BANNER_FOCAL } from '../../lib/cover-focal.js';
  import { openExternal } from '../../lib/open-external.js';
  import VnMergeDialog from '../VnMergeDialog.svelte';
  import { STATE_BADGE, STATE_LABEL } from '../../lib/backfill-labels.js';

  let { item, running = false, onchanged, onopenform, onclose } = $props();

  const PHASES = {
    meta: 'gameForm.search.phaseMeta',
    characters: 'gameForm.search.phaseCharacters',
    images: 'gameForm.search.phaseImages',
    cover: 'gameForm.search.phaseCover',
  };

  let keyword = $state('');
  const search = createVndbSearch();
  const flow = createVnMergeFlow();
  let searched = $state(false);
  let busy = $state(false);
  let mergeOpen = $state(false);
  let done = $state(null);

  let lastId = null;
  $effect(() => {
    if (item.game_id === lastId) return;
    lastId = item.game_id;
    keyword = item.name ?? '';
    search.reset();
    flow.reset();
    searched = false;
    done = null;
  });

  const guessed = $derived(item.bind_source === 'search');
  const hint = $derived(
    item.state === null
      ? 'backfillPage.panelLibrary'
      : guessed
        ? 'backfillPage.panelGuessed'
        : 'backfillPage.panelDirect',
  );
  const bound = $derived(item.hit_title ?? '');
  const year = $derived(item.hit_released ? item.hit_released.slice(0, 4) : '');

  const candidates = $derived(
    search.hits.length
      ? search.hits.map((hit) => ({
          id: hit.id,
          title: hit.alttitle || hit.title,
          alt: hit.alttitle && hit.alttitle !== hit.title ? hit.title : '',
          released: hit.released,
          cover: hit.image?.thumbnail ?? null,
        }))
      : (item.candidates ?? []).map((cand) => ({
          id: cand.id,
          title: cand.title,
          alt: '',
          released: cand.released,
          cover: null,
        })),
  );

  const pullText = $derived.by(() => {
    const progress = flow.progress;
    if (!progress) return t('backfillPage.pulling');
    const label = t(PHASES[progress.phase] ?? 'backfillPage.pulling');
    if (progress.total == null) return label;
    return `${label} ${progress.done ?? 0}/${progress.total}`;
  });

  const pullRatio = $derived(
    flow.progress?.total
      ? Math.min(1, (flow.progress.done ?? 0) / flow.progress.total)
      : null,
  );

  const doneText = $derived.by(() => {
    if (!done) return '';
    const parts = [];
    if (done.fields) parts.push(t('backfillPage.fixFields', { count: done.fields }));
    if (done.characters) parts.push(t('backfillPage.fixChars', { count: done.characters }));
    return parts.join(' · ') || t('backfillPage.fixNone');
  });

  function runSearch() {
    const term = (keyword ?? '').trim();
    if (!term || flow.pulling) return;
    searched = true;
    search.search(term);
  }

  function toFields(game, meta) {
    return {
      name: game.name,
      alias: meta?.alias ?? '',
      released: meta?.released ?? '',
      rating: meta?.rating != null ? String(meta.rating) : '',
      votecount: meta?.votecount != null ? String(meta.votecount) : '',
      lengthMinutes: meta?.length_minutes != null ? String(meta.length_minutes) : '',
      description: meta?.description ?? '',
      coverPath: meta?.image_path ?? '',
      focusX: meta?.cover_focal_x ?? 50,
      focusY: meta?.cover_focal_y ?? 50,
      tags: meta?.tags ?? [],
      developers: (meta?.developers ?? []).join(', '),
      staff: meta?.staff ?? [],
      ambiencePath: meta?.ambience_path ?? '',
      ambienceMode: meta?.ambience_mode ?? 'soft',
      ambiencePlate: meta?.ambience_plate ?? false,
      ambienceFocalX: meta?.ambience_focal_x ?? BANNER_FOCAL.x,
      ambienceFocalY: meta?.ambience_focal_y ?? BANNER_FOCAL.y,
    };
  }

  async function pick(id) {
    if (flow.pulling || busy || running) return;
    mergeOpen = false;
    done = null;
    const ok = await flow.pick(id, async () => {
      const [game, meta, chars] = await Promise.all([
        api.getGame(item.game_id),
        api.getGameMetadata(item.game_id),
        api.listGameCharacters(item.game_id).catch(() => []),
      ]);
      return {
        ownId: item.game_id,
        current: toFields(game, meta),
        characters: chars,
      };
    });
    if (ok) mergeOpen = true;
  }

  async function accept(picked, charPicked) {
    mergeOpen = false;
    const result = flow.accept(picked, charPicked);
    if (!result) return;
    busy = true;
    try {
      const next = result.fields;
      const kept = result.characters.filter(
        (c) => (c.name ?? '').trim() || (c.original ?? '').trim(),
      );
      const coverChanged = next.coverPath !== result.previous.coverPath;
      await api.updateGame(item.game_id, { name: next.name, vndb_id: result.id });
      await api.updateGameMetadata(
        item.game_id,
        buildMetadataPatch({
          ...next,
          focusX: coverChanged ? 50 : result.previous.focusX,
          focusY: coverChanged ? 50 : result.previous.focusY,
        }),
      );
      await api.saveGameCharacters(item.game_id, kept);
      done = { fields: picked.size, characters: kept.length };
      const patch = {
        game_id: item.game_id,
        vndb_id: result.id,
        state: 'ok',
        bind_source: 'existing',
        characters: kept.length,
        error: null,
      };
      if (result.incoming.title) patch.hit_title = result.incoming.title;
      if (result.incoming.released) patch.hit_released = result.incoming.released;
      onchanged?.(patch);
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      busy = false;
    }
  }

  async function retry() {
    if (busy || flow.pulling || running || !item.vndb_id) return;
    done = null;
    const state = await flow.run(
      'vndb-backfill',
      () => api.backfillSingle(item.game_id),
      (ev) => !ev.game_id || ev.game_id === item.game_id,
    );
    if (!state) return;
    onchanged?.({ ...state, bind_source: item.bind_source ?? state.bind_source });
    if (state.state === 'failed') {
      toast(state.error ?? t('backfillPage.retryFailedShort'), 'error');
      return;
    }
    done = { fields: state.filled?.length ?? 0, characters: state.characters ?? 0 };
  }

  function openVndb() {
    if (!item.vndb_id) return;
    openExternal(`https://vndb.org/${item.vndb_id}`);
  }
</script>

<div class="bf-panel__head">
  {#if item.state}
    <span class="bf-badge {STATE_BADGE[item.state]}">
      {t(STATE_LABEL[item.state])}
    </span>
  {:else}
    <span class="bf-badge bf-badge--pending">{t('backfillPage.library')}</span>
  {/if}
  <button type="button" class="bf-panel__close" aria-label={t('common.close')} onclick={onclose}>
    ×
  </button>
</div>

<p class="bf-note">{t(hint)}</p>

{#if guessed}
  <div class="bf-search">
    <input
      class="bf-input"
      type="text"
      bind:value={keyword}
      placeholder={t('backfillPage.searchPlaceholder')}
      aria-label={t('backfillPage.searchPlaceholder')}
      onkeydown={(e) => {
        if (e.key === 'Enter' && !e.isComposing) runSearch();
      }}
    />
    <button
      type="button"
      class="btn btn-ghost btn-sm"
      disabled={search.searching || flow.pulling || !(keyword ?? '').trim()}
      onclick={runSearch}
    >
      {search.searching ? t('gameForm.search.loading') : t('backfillPage.searchAction')}
    </button>
  </div>
{/if}

{#if flow.pulling}
  <div class="bf-pull">
    <span class="bf-pull__text">{pullText}</span>
    <div
      class="bf-prog"
      class:bf-prog--known={pullRatio != null}
      role="progressbar"
      aria-label={pullText}
      aria-valuenow={pullRatio == null ? undefined : Math.round(pullRatio * 100)}
    >
      <span
        class="bf-prog__bar"
        style={pullRatio == null ? undefined : `width: ${pullRatio * 100}%`}
      ></span>
    </div>
    {#if flow.notice}
      <span class="bf-field__value bf-field__value--error">{flow.notice}</span>
    {/if}
  </div>
{/if}

{#if guessed && candidates.length}
  <ul class="bf-cands">
    {#each candidates as cand (cand.id)}
      <li>
        <button
          type="button"
          class="bf-cand"
          class:is-current={cand.id === item.vndb_id}
          disabled={flow.pulling || busy || running}
          onclick={() => pick(cand.id)}
        >
          <span class="bf-cand__cover">
            {#if cand.cover}
              <img src={cand.cover} alt="" loading="lazy" decoding="async" />
            {:else}
              <span class="bf-cand__ph">{(cand.title || '?')[0]}</span>
            {/if}
          </span>
          <span class="bf-cand__text">
            <span class="bf-cand__title">{cand.title}</span>
            {#if cand.alt}<span class="bf-cand__alt">{cand.alt}</span>{/if}
            <span class="bf-cand__sub">
              {[cand.released?.slice(0, 4), cand.id].filter(Boolean).join(' · ')}
            </span>
          </span>
        </button>
      </li>
    {/each}
  </ul>
{:else if searched && !search.searching && !search.searchError && !flow.pulling}
  <p class="bf-field__muted">{t('backfillPage.candidatesEmpty')}</p>
{/if}

{#if guessed && search.searchError}
  <p class="bf-field__value bf-field__value--error">{search.searchError}</p>
{/if}

<div class="bf-field">
  <span class="bf-field__label">{t('backfillPage.panelLocal')}</span>
  <span class="bf-field__value">{item.name}</span>
</div>

<div class="bf-field">
  <span class="bf-field__label">{t('backfillPage.panelBound')}</span>
  {#if bound || item.vndb_id}
    <span class="bf-field__value">
      {bound || item.vndb_id}
      {#if year}<span class="bf-field__muted">{year}</span>{/if}
    </span>
    {#if item.vndb_id}
      <a
        class="bf-vndb-link"
        href={`https://vndb.org/${item.vndb_id}`}
        onclick={(e) => {
          e.preventDefault();
          openVndb();
        }}
      >
        {item.vndb_id} · {t('backfillPage.openVndb')}
        <Icon name="arrow-up-right" size="sm" />
      </a>
    {/if}
  {:else}
    <span class="bf-field__muted">—</span>
  {/if}
</div>

{#if done}
  <div class="bf-field">
    <span class="bf-field__label">{t('backfillPage.panelResult')}</span>
    <span class="bf-field__value">{doneText}</span>
  </div>
{:else if item.state === 'ok'}
  <div class="bf-field">
    <span class="bf-field__label">{t('backfillPage.panelFields')}</span>
    {#if item.filled.length}
      <span class="bf-chips">
        {#each item.filled as f (f)}<span class="bf-chip">{f}</span>{/each}
      </span>
    {:else}
      <span class="bf-field__muted">{t('backfillPage.filledNone')}</span>
    {/if}
    {#if item.characters > 0}
      <span class="bf-field__muted">
        {t('backfillPage.characterCount', { count: item.characters })}
      </span>
    {/if}
  </div>
{:else if item.error}
  <div class="bf-field">
    <span class="bf-field__label">{t('backfillPage.panelError')}</span>
    <span class="bf-field__value bf-field__value--error">{item.error}</span>
  </div>
{/if}

<div class="bf-panel__actions">
  <button
    type="button"
    class="btn btn-ghost btn-sm"
    disabled={busy || flow.pulling || running || !item.vndb_id}
    onclick={retry}
  >
    {t('backfillPage.retryOne')}
  </button>
  <button
    type="button"
    class="btn btn-ghost btn-sm"
    disabled={busy || flow.pulling || running}
    onclick={() => onopenform?.(item.game_id)}
  >
    {t('backfillPage.openInForm')}
  </button>
</div>

<VnMergeDialog
  bind:open={mergeOpen}
  rows={flow.rows}
  charRows={flow.charRows}
  onaccept={accept}
  oncancel={() => (mergeOpen = false)}
/>
