<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { subscribe } from '../lib/events.js';
  import { createVndbSearch } from '../lib/vndb-search.svelte.js';
  import {
    buildMetadataPatch,
    clampRating,
    cleanRating,
    ratingRejected,
    toNumber,
    votecountRejected,
  } from '../lib/form-metadata.js';
  import { applyIncoming, conflictRows } from '../lib/vn-merge.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { pickImage } from '../lib/pick-path.js';
  import VnMergeDialog from './VnMergeDialog.svelte';
  import CharacterMergeDialog from './CharacterMergeDialog.svelte';
  import CharacterEditor from './CharacterEditor.svelte';
  import { namedOnly, toDraft, toPayload } from '../lib/characters.js';
  import { applyCharacters, planCharacters } from '../lib/char-merge.js';
  import { attachScrollbars, destroyScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';

  let { open = $bindable(false), gameId = $bindable(null), onsaved } = $props();

  const TABS = ['basic', 'profile', 'tags', 'characters'];

  const STATUS_KEYS = ['playing', 'wishlist', 'finished', 'dropped'];

  let name = $state('');
  let vndbId = $state(null);
  let status = $state('wishlist');
  let userRating = $state(null);
  let userRatingDraft = $state('');
  let notes = $state('');
  let error = $state('');
  let saving = $state(false);
  let loading = $state(false);
  let input = $state(null);

  let alias = $state('');
  let released = $state('');
  let rating = $state('');
  let votecount = $state('');
  let ratingError = $state('');
  let votecountError = $state('');
  let description = $state('');
  let developers = $state('');
  let tags = $state([]);
  let coverPath = $state('');

  let tagDraft = $state('');
  let tagInputOpen = $state(false);
  let characters = $state([]);
  
  let mergeOpen = $state(false);
  let mergeRows = $state([]);
  let charMergeOpen = $state(false);
  let charMergeRows = $state([]);
  let incomingCharacters = $state([]);
  let pulling = $state(false);
  let syncProgress = $state(null);
  let syncCancelled = $state(false);
  let syncNotice = $state('');
  let incoming = $state(null);
  let coverBroken = $state(false);
  let coverUrl = $state(null);
  let coverError = $state('');

  let tab = $state('basic');
  let statusOpen = $state(false);

  let searchOpen = $state(false);
  const search = createVndbSearch();

  let panelsEl = $state(null);
  let panelsSb = null;

  $effect(() => {
    if (!open) return;
    name = '';
    vndbId = null;
    status = 'wishlist';
    userRating = null;
    userRatingDraft = '';
    notes = '';
    error = '';
    released = '';
    alias = '';
    rating = '';
    votecount = '';
    ratingError = '';
    votecountError = '';
    description = '';
    developers = '';
    tags = [];
    coverPath = '';
    coverUrl = null;
    coverError = '';
    tagDraft = '';
    tagInputOpen = false;
    characters = [];
    
    coverBroken = false;
    tab = 'basic';
    statusOpen = false;
    searchOpen = false;
    mergeOpen = false;
    mergeRows = [];
    charMergeOpen = false;
    charMergeRows = [];
    incomingCharacters = [];
    incoming = null;
    pulling = false;
    syncProgress = null;
    syncCancelled = false;
    syncNotice = '';
    search.reset();
    loading = Boolean(gameId);
    if (gameId) loadExisting(gameId);
    tick().then(() => input?.focus());
  });

  $effect(() => {
    if (open && panelsEl && !panelsSb) {
      panelsSb = attachScrollbars(panelsEl);
    } else if ((!open || !panelsEl) && panelsSb) {
      destroyScrollbars(panelsSb);
      panelsSb = null;
    }
  });

  function statusLabel(key) {
    return t(`status.${key}`);
  }

  async function loadExisting(id) {
    try {
      const [game, state, meta, roles] = await Promise.all([
        api.getGame(id),
        api.getGameState(id),
        api.getGameMetadata(id),
        api.listGameCharacters(id).catch(() => []),
      ]);
      if (gameId !== id) return;
      name = game.name;
      vndbId = game.vndb_id;
      status = state.status;
      userRating = state.user_rating;
      userRatingDraft = state.user_rating == null ? '' : String(state.user_rating);
      notes = state.notes ?? '';
      released = meta?.released ?? '';
      alias = meta?.alias ?? '';
      rating = meta?.rating != null ? String(meta.rating) : '';
      votecount = meta?.votecount != null ? String(meta.votecount) : '';
      description = meta?.description ?? '';
      developers = (meta?.developers ?? []).join(', ');
      tags = (meta?.tags ?? []).map((tag) => ({ ...tag }));
      coverPath = meta?.image_path ?? '';
      characters = roles.map(toDraft);
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    const path = coverPath.trim();
    coverBroken = false;
    coverUrl = null;
    if (!path) return;
    let live = true;
    coverSrc(path)
      .then((url) => {
        if (live) coverUrl = url;
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  });

  async function browseCover() {
    coverError = '';
    try {
      const picked = await pickImage(t('gameForm.field.cover'));
      if (picked) coverPath = picked;
    } catch (err) {
      coverError = err.message;
    }
  }

  function stepRating(delta) {
    const current = userRating ?? 0;
    const next = Math.round((current + delta) * 10) / 10;
    if (next <= 0) {
      userRating = delta < 0 ? null : 5;
    } else {
      userRating = Math.min(100, next);
    }
    userRatingDraft = userRating === null ? '' : String(userRating);
  }

  function onUserRatingInput(e) {
    const cleaned = cleanRating(e.target.value);
    e.target.value = cleaned;
    userRatingDraft = cleaned;
    userRating = cleaned === '' ? null : toNumber(cleaned);
  }

  function onUserRatingBlur(e) {
    if (userRating === null) {
      e.target.value = '';
      return;
    }
    userRating = clampRating(userRating);
    userRatingDraft = String(userRating);
    e.target.value = userRatingDraft;
  }

  function newTag(name) {
    return { name };
  }

  function addTag() {
    const value = tagDraft.trim();
    if (!value) return;
    if (!tags.some((tag) => tag.name === value)) tags = [...tags, newTag(value)];
    tagDraft = '';
  }

  function onTagNewKeydown(e) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag();
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      tagDraft = '';
      tagInputOpen = false;
      return;
    }
    if (e.key === 'Backspace' && !tagDraft && tags.length) {
      e.preventDefault();
      removeTag(tags.length - 1);
    }
  }

  function removeTag(index) {
    tags = tags.filter((_, i) => i !== index);
  }

  

  function formFields() {
    return {
      name,
      alias,
      released,
      rating,
      votecount,
      description,
      coverPath,
      tags,
      developers,
    };
  }

  const PHASE_KEYS = {
    meta: 'gameForm.search.phaseMeta',
    characters: 'gameForm.search.phaseCharacters',
    images: 'gameForm.search.phaseImages',
    cover: 'gameForm.search.phaseCover',
  };

  const pullText = $derived.by(() => {
    if (!syncProgress) return t('gameForm.search.pulling');
    const label = t(PHASE_KEYS[syncProgress.phase] ?? 'gameForm.search.pulling');
    if (syncProgress.total == null) return label;
    return `${label} ${syncProgress.done ?? 0}/${syncProgress.total}`;
  });

  const pullRatio = $derived(
    syncProgress?.total ? Math.min(1, (syncProgress.done ?? 0) / syncProgress.total) : null
  );

  async function pickHit(id) {
    vndbId = id;
    if (pulling) return;
    pulling = true;
    syncProgress = null;
    syncCancelled = false;
    syncNotice = '';
    error = '';
    let off = null;
    try {
      off = await subscribe('vndb-sync', (event) => {
        if (event.error) {
          syncNotice = event.error;
        } else {
          syncProgress = event;
        }
      });
      incoming = await api.fetchVnMetadata(id);
      if (syncCancelled) {
        incoming = null;
        incomingCharacters = [];
        return;
      }
      incomingCharacters = incoming?.characters ?? [];
      mergeRows = conflictRows(formFields(), incoming);
      mergeOpen = true;
    } catch (err) {
      incoming = null;
      incomingCharacters = [];
      if (!syncCancelled) toast(err.message, 'error');
    } finally {
      off?.();
      pulling = false;
      syncProgress = null;
    }
  }

  async function cancelSync() {
    syncCancelled = true;
    await api.cancelTask('vndb-sync');
  }

  function acceptMerge(picked) {
    const next = applyIncoming(formFields(), incoming, picked);
    name = next.name;
    alias = next.alias;
    released = next.released;
    rating = next.rating;
    votecount = next.votecount;
    description = next.description;
    tags = next.tags;
    developers = next.developers;
    coverPath = next.coverPath;
    mergeOpen = false;
    incoming = null;
    openCharacterMerge();
  }

  function openCharacterMerge() {
    const local = characters.map(toPayload);
    const plan = planCharacters(local, incomingCharacters);
    if (plan.conflicts.length === 0) {
      characters = applyCharacters(local, incomingCharacters, new Set()).map(toDraft);
      incomingCharacters = [];
      return;
    }
    charMergeRows = plan.conflicts;
    charMergeOpen = true;
  }

  function acceptCharacterMerge(adopted) {
    characters = applyCharacters(characters.map(toPayload), incomingCharacters, adopted).map(
      toDraft
    );
    charMergeOpen = false;
    incomingCharacters = [];
  }

  function close() {
    if (saving) return false;
    open = false;
  }

  function validateRanges() {
    ratingError = ratingRejected(rating) ? t('gameForm.error.ratingRange') : '';
    votecountError = votecountRejected(votecount) ? t('gameForm.error.votecountRange') : '';
    return ratingError || votecountError;
  }

  async function submit(e) {
    e.preventDefault();
    if (saving || loading) return;

    const value = name.trim();
    if (!value) {
      error = t('gameForm.error.nameRequired');
      return;
    }

    const rangeError = validateRanges();
    if (rangeError) {
      tab = 'profile';
      return;
    }

    saving = true;
    error = '';
    const patch = buildMetadataPatch({
      alias,
      released,
      rating,
      votecount,
      description,
      coverPath,
      tags,
      developers,
    });
    try {
      const state = {
        status,
        user_rating: userRating,
        notes: notes.trim() || null,
      };
      if (gameId) {
        await api.updateGame(gameId, { name: value, vndb_id: vndbId });
        await api.updateGameState(gameId, state);
        await api.updateGameMetadata(gameId, patch);
        await api.saveGameCharacters(gameId, namedOnly(characters).map(toPayload));
      } else {
        const created = await api.createGame({
          name: value,
          vndb_id: vndbId,
          metadata: patch,
        });
        await api.updateGameState(created.id, state);
          await api.saveGameCharacters(created.id, namedOnly(characters).map(toPayload));
      }
      open = false;
      onsaved?.();
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      saving = false;
    }
  }
</script>

<Modal
  bind:open
  bind:asideOpen={searchOpen}
  title={t('gameForm.title')}
  variant="modal--form"
  asideTitle={t('gameForm.search.title')}
  onclose={close}
>
  <form class="form-panel" onsubmit={submit}>
    <div class="modal-tabs" role="tablist">
      {#each TABS as key (key)}
        <button
          type="button"
          role="tab"
          class="modal-tab"
          class:active={tab === key}
          aria-selected={tab === key}
          onclick={() => (tab = key)}
        >
          {t(`gameForm.tab.${key}`)}
        </button>
      {/each}
    </div>

    <div class="modal-panels scroll-thin scroll-form" bind:this={panelsEl}>
      <div class="modal-panel" class:active={tab === 'basic'}>
        <div class="panel-grid">
          <div class="form-group panel-grid__full">
            <label for="game-form-name">{t('gameForm.field.name')}</label>
            <input
              id="game-form-name"
              type="text"
              bind:value={name}
              bind:this={input}
              placeholder={t('gameForm.namePlaceholder')}
              disabled={saving}
            />
            {#if error}
              <p class="form-hint form-hint--error">{error}</p>
            {/if}
          </div>

          <div class="form-group">
            <label for="game-form-status">{t('gameForm.field.status')}</label>
            <div class="select-dd" class:is-open={statusOpen}>
              <button
                id="game-form-status"
                type="button"
                class="select-dd-trigger"
                aria-haspopup="listbox"
                aria-expanded={statusOpen}
                disabled={saving}
                onclick={() => (statusOpen = !statusOpen)}
                onblur={() => (statusOpen = false)}
              >
                <span class="select-dd-text">{statusLabel(status)}</span>
                <svg
                  class="select-dd-chevron"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
              </button>
              {#if statusOpen}
                <div class="select-dd-menu" role="listbox">
                  {#each STATUS_KEYS as key (key)}
                    <button
                      type="button"
                      role="option"
                      aria-selected={status === key}
                      class="select-dd-opt"
                      class:is-selected={status === key}
                      onmousedown={(e) => e.preventDefault()}
                      onclick={() => {
                        status = key;
                        statusOpen = false;
                      }}
                    >
                      <span>{statusLabel(key)}</span>
                      <svg
                        class="select-dd-opt-check"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2.5"
                      >
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>

          <div class="form-group">
            <label for="game-form-rating">{t('gameForm.field.userRating')}</label>
            <div class="num-stepper">
              <button
                type="button"
                class="num-stepper-btn"
                onclick={() => stepRating(-5)}
                disabled={saving}
                aria-label={t('gameForm.action.ratingDown')}
              >−</button>
              <input
                id="game-form-rating"
                class="num-stepper-val"
                type="text"
                inputmode="decimal"
                value={userRatingDraft}
                placeholder={t('gameForm.placeholder.rating')}
                oninput={onUserRatingInput}
                onblur={onUserRatingBlur}
                disabled={saving}
                aria-label={t('gameForm.field.userRating')}
              />
              <button
                type="button"
                class="num-stepper-btn"
                onclick={() => stepRating(5)}
                disabled={saving}
                aria-label={t('gameForm.action.ratingUp')}
              >+</button>
            </div>
            <p class="form-hint">{t('gameForm.hint.userRating')}</p>
          </div>

          <div class="form-group panel-grid__full">
            <label for="game-form-notes">{t('gameForm.field.notes')}</label>
            <textarea
              id="game-form-notes"
              bind:value={notes}
              placeholder={t('gameForm.placeholder.notes')}
              disabled={saving}
            ></textarea>
          </div>

          <div class="form-group panel-grid__full form-group--tall">
            <label for="game-form-description">{t('gameForm.field.description')}</label>
            <textarea
              id="game-form-description"
              class="scroll-thin"
              bind:value={description}
              placeholder={t('gameForm.placeholder.description')}
              disabled={saving}
            ></textarea>
          </div>
        </div>
      </div>

      <div class="modal-panel" class:active={tab === 'profile'}>
        <div class="panel-grid">
          <div class="form-group panel-grid__full">
            <label for="game-form-meta-alias">{t('gameForm.field.alias')}</label>
            <input
              id="game-form-meta-alias"
              type="text"
              bind:value={alias}
              placeholder={t('gameForm.placeholder.alias')}
              disabled={saving}
            />
          </div>

          <div class="form-group">
            <label for="game-form-meta-released">{t('gameForm.field.released')}</label>
            <input
              id="game-form-meta-released"
              type="text"
              bind:value={released}
              placeholder={t('gameForm.placeholder.released')}
              disabled={saving}
            />
          </div>

          <div class="form-group">
            <label
              for="game-form-meta-rating"
              data-tooltip={t('gameForm.hint.ratingScale')}
            >
              {t('gameForm.field.rating')}
            </label>
            <input
              id="game-form-meta-rating"
              type="text"
              inputmode="decimal"
              bind:value={rating}
              placeholder={t('gameForm.placeholder.rating')}
              disabled={saving}
            />
            <p class="form-hint">{t('gameForm.hint.ratingScale')}</p>
            {#if ratingError}
              <p class="form-hint form-hint--error">{ratingError}</p>
            {/if}
          </div>

          <div class="form-group">
            <label for="game-form-meta-votecount">{t('gameForm.field.votecount')}</label>
            <input
              id="game-form-meta-votecount"
              type="text"
              inputmode="numeric"
              bind:value={votecount}
              placeholder={t('gameForm.placeholder.votecount')}
              disabled={saving}
            />
            {#if votecountError}
              <p class="form-hint form-hint--error">{votecountError}</p>
            {/if}
          </div>

          <div class="form-group">
            <label for="game-form-meta-developers">{t('gameForm.field.developers')}</label>
            <input
              id="game-form-meta-developers"
              type="text"
              bind:value={developers}
              placeholder={t('gameForm.placeholder.developers')}
              disabled={saving}
            />
          </div>
        </div>
      </div>

      <div class="modal-panel" class:active={tab === 'tags'}>
        <div class="form-group">
          <label for="game-form-tags">{t('gameForm.field.tags')}</label>
          <div class="tag-input scroll-thin">
            {#each tags as tag, i (tag.id ?? tag.name)}
              <span class="tag-pill">
                {tag.name}
                <button
                  type="button"
                  class="tag-pill__remove"
                  aria-label={t('gameForm.action.removeTag')}
                  onmousedown={(e) => e.preventDefault()}
                  onclick={() => removeTag(i)}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </span>
            {/each}
            <button
              type="button"
              class="tag-input__add"
              aria-expanded={tagInputOpen}
              aria-label={t('gameForm.action.addTag')}
              onclick={() => (tagInputOpen = !tagInputOpen)}
              disabled={saving}
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
              </svg>
            </button>
          </div>
          {#if tagInputOpen}
            <input
              id="game-form-tag-new"
              class="tag-input__new"
              type="text"
              bind:value={tagDraft}
              onkeydown={onTagNewKeydown}
              placeholder={t('gameForm.placeholder.tags')}
              disabled={saving}
            />
          {/if}
        </div>

        <div class="form-group">
          <label for="game-form-cover">{t('gameForm.field.cover')}</label>
          <div class="cover-field">
            <div class="cover-field__preview">
              {#if coverUrl && !coverBroken}
                <img
                  src={coverUrl}
                  alt=""
                  onerror={() => (coverBroken = true)}
                />
              {:else}
                <div class="cover-field__ph">{t('gameForm.placeholder.empty')}</div>
              {/if}
            </div>
            <div class="cover-field__main">
              <div class="cover-field__row">
                <input
                  id="game-form-cover"
                  type="text"
                  bind:value={coverPath}
                  placeholder={t('gameForm.placeholder.cover')}
                  disabled={saving}
                />
                <button
                  type="button"
                  class="btn cover-field__browse"
                  onclick={browseCover}
                  disabled={saving}
                >
                  {t('gameForm.action.browseCover')}
                </button>
              </div>
              {#if coverError}
                <p class="form-hint form-hint--error">{coverError}</p>
              {/if}
            </div>
          </div>
        </div>
      </div>

      <div class="modal-panel" class:active={tab === 'characters'}>
        <CharacterEditor bind:characters disabled={saving} />
      </div>
    </div>

    <div class="modal-actions">
      <button
        type="button"
        class="game-form__sync btn"
        onclick={() => {
          searchOpen = true;
          search.search(name);
        }}
        disabled={saving || search.searching || !name.trim()}
      >
        {t('gameForm.action.searchVndb')}
      </button>
      <button type="button" class="btn" onclick={close} disabled={saving}>
        {t('common.cancel')}
      </button>
      <button type="submit" class="btn btn-primary" disabled={saving}>
        {t('common.save')}
      </button>
    </div>
  </form>

  {#snippet aside()}
    <div class="modal-panels scroll-thin">
      <div class="modal-panel active">
        {#if search.searching}
          <div class="vndb-progress" role="progressbar" aria-label={t('gameForm.search.loading')}>
            <span class="vndb-progress__bar"></span>
          </div>
        {:else if search.searchError}
          <p class="vndb-note vndb-note--error">{search.searchError}</p>
        {:else if search.hits.length === 0}
          <p class="vndb-note">{t('gameForm.search.empty')}</p>
        {:else}
          {#if pulling}
            <p class="vndb-note">{pullText}</p>
            <div
              class="vndb-progress"
              class:vndb-progress--determinate={pullRatio != null}
              role="progressbar"
              aria-label={pullText}
              aria-valuenow={pullRatio == null ? undefined : Math.round(pullRatio * 100)}
            >
              <span
                class="vndb-progress__bar"
                style={pullRatio == null ? undefined : `width: ${pullRatio * 100}%`}
              ></span>
            </div>
            {#if syncNotice}
              <p class="vndb-note vndb-note--error">{syncNotice}</p>
            {/if}
            <button type="button" class="btn vndb-pull-cancel" onclick={cancelSync} disabled={syncCancelled}>
              {t('common.cancel')}
            </button>
          {/if}
          <ul class="vndb-hits" class:is-pulling={pulling}>
            {#each search.hits as hit (hit.id)}
              <li>
                <button
                  type="button"
                  class="vndb-hit"
                  class:active={vndbId === hit.id}
              disabled={pulling}
                  onclick={() => pickHit(hit.id)}
                >
                  <span class="vndb-hit__cover">
                    {#if hit.image?.thumbnail}
                      <img src={hit.image.thumbnail} alt="" loading="lazy" decoding="async" />
                    {:else}
                      <span class="vndb-hit__ph">{(hit.alttitle || hit.title || '?')[0]}</span>
                    {/if}
                  </span>
                  <span class="vndb-hit__text">
                    <span class="vndb-hit__title">{hit.alttitle || hit.title}</span>
                    {#if hit.alttitle && hit.alttitle !== hit.title}
                      <span class="vndb-hit__alt">{hit.title}</span>
                    {/if}
                    <span class="vndb-hit__meta">
                      {[hit.released?.slice(0, 4), hit.olang].filter(Boolean).join(' · ')}
                    </span>
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>

    <div class="modal-actions">
      <button type="button" class="btn" onclick={() => (searchOpen = false)}>
        {t('gameForm.search.close')}
      </button>
    </div>
  {/snippet}
</Modal>

<VnMergeDialog
  bind:open={mergeOpen}
  rows={mergeRows}
  onaccept={acceptMerge}
  oncancel={() => (mergeOpen = false)}
/>

<CharacterMergeDialog
  bind:open={charMergeOpen}
  rows={charMergeRows}
  onaccept={acceptCharacterMerge}
  oncancel={() => (charMergeOpen = false)}
/>

<style>
  .game-form__sync {
    margin-right: auto;
  }

  .vndb-note {
    padding: var(--space-4) 0;
    font-size: var(--text-base);
    color: var(--fg-muted);
  }

  .vndb-note--error {
    color: var(--error);
  }

  .vndb-progress {
    position: relative;
    overflow: hidden;
    height: var(--space-075);
    margin-top: var(--space-2);
    background: var(--bg-surface);
    border-radius: var(--radius-pill);
  }

  .vndb-progress__bar {
    position: absolute;
    inset-block: 0;
    width: 40%;
    background: var(--accent-gradient);
    border-radius: var(--radius-pill);
    animation: vndb-sweep 1.1s var(--ease-out) infinite;
  }

  .vndb-progress--determinate .vndb-progress__bar {
    animation: none;
    transition: width var(--duration-fast) var(--ease-out);
  }

  .vndb-pull-cancel {
    margin-top: var(--space-2);
  }

  @keyframes vndb-sweep {
    from {
      left: -40%;
    }
    to {
      left: 100%;
    }
  }

  .vndb-hits {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    list-style: none;
    transition: opacity var(--duration-fast) var(--ease-out);
  }

  .vndb-hits.is-pulling {
    opacity: 0.45;
  }

  .vndb-hit {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3);
    text-align: left;
    color: var(--fg-default);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  .vndb-hit:hover {
    background: var(--bg-raised);
    border-color: var(--border-strong);
  }

  .vndb-hit.active {
  background: var(--bg-raised);
  border-color: var(--border-strong);
}

  .vndb-hit__cover {
    flex-shrink: 0;
    width: var(--space-28);
    aspect-ratio: 3 / 4;
    overflow: hidden;
    background: var(--bg-raised);
    border-radius: var(--radius-sm);
  }

  .vndb-hit__cover img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .vndb-hit__ph {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    font-size: var(--text-xl);
    color: var(--fg-muted);
  }

  .vndb-hit__text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .vndb-hit__title {
    display: -webkit-box;
    overflow: hidden;
    font-size: var(--text-body);
    font-weight: var(--fw-medium);
    color: var(--fg-default);
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .vndb-hit__alt,
  .vndb-hit__meta {
    overflow: hidden;
    font-size: var(--text-base);
    color: var(--fg-muted);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
