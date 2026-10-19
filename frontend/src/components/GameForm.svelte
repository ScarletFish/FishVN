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
    lengthMinutesRejected,
    ratingRejected,
    toNumber,
    votecountRejected,
  } from '../lib/form-metadata.js';
  import { applyIncoming, conflictRows } from '../lib/vn-merge.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { pickImage } from '../lib/pick-path.js';
  import VnMergeDialog from './VnMergeDialog.svelte';
  import CharacterEditor from './CharacterEditor.svelte';
  import InstallationList from './InstallationList.svelte';
  import PlaytimeField from './PlaytimeField.svelte';
  import PathField from './PathField.svelte';
  import SelectDd from './SelectDd.svelte';
  import { namedOnly, toDraft, toPayload } from '../lib/characters.js';
  import {
    defaultKeyOf,
    installPathInvalid,
    resolveDefaultId,
    toInstallationDraft,
    toInstallationPayload,
  } from '../lib/installations.js';
  import { applyCharacters, planCharacters } from '../lib/char-merge.js';
  import { STAFF_ROLES, staffRoleLabel } from '../lib/staff.js';
  import { attachScrollbars, destroyScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';

  let {
    open = $bindable(false),
    gameId = $bindable(null),
    onsaved,
    initialTab = 'basic',
    prefill = null,
  } = $props();

  const TABS = ['basic', 'profile', 'tags', 'staff', 'installations', 'characters'];

  const STATUS_KEYS = ['playing', 'wishlist', 'finished', 'dropped'];
  const STATUS_OPTIONS = STATUS_KEYS.map((key) => ({ value: key, label: statusLabel(key) }));
  const STAFF_ROLE_OPTIONS = STAFF_ROLES.map((role) => ({ value: role, label: staffRoleLabel(role) }));

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
  let lengthMinutes = $state('');
  let ratingError = $state('');
  let votecountError = $state('');
  let lengthMinutesError = $state('');
  let description = $state('');
  let developers = $state('');
  let tags = $state([]);
  let staff = $state([]);
  let coverPath = $state('');

  let tagDraft = $state('');
  let tagInputOpen = $state(false);
  let characters = $state([]);
  let installations = $state([]);
  let defaultInstallKey = $state(null);
  
  let mergeOpen = $state(false);
  let mergeRows = $state([]);
  let charMergeRows = $state([]);
  let incomingCharacters = $state([]);
  let pulling = $state(false);
  let syncProgress = $state(null);
  let syncCancelled = $state(false);
  let syncNotice = $state('');
  let incoming = $state(null);
  let coverBroken = $state(false);
  let coverUrl = $state(null);

  let tab = $state('basic');
  let statusOpen = $state(false);
  let staffRoleOpen = $state(null);

  let searchOpen = $state(false);
  const search = createVndbSearch();

  let panelsEl = $state(null);
  let panelsSb = null;

  $effect(() => {
    if (!open || saving) return;
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
    lengthMinutes = '';
    ratingError = '';
    votecountError = '';
    lengthMinutesError = '';
    description = '';
    developers = '';
    tags = [];
    staff = [];
    coverPath = '';
    coverUrl = null;
    tagDraft = '';
    tagInputOpen = false;
    characters = [];
    installations = [];
    defaultInstallKey = null;
    
    coverBroken = false;
    tab = initialTab;
    statusOpen = false;
    staffRoleOpen = null;
    searchOpen = false;
    mergeOpen = false;
    mergeRows = [];
    charMergeRows = [];
    incomingCharacters = [];
    incoming = null;
    pulling = false;
    syncProgress = null;
    syncCancelled = false;
    syncNotice = '';
    search.reset();
    loading = Boolean(gameId);
    if (gameId) {
      loadExisting(gameId);
    } else if (prefill) {
      name = prefill.name;
      const drafts = prefill.installations.map(toInstallationDraft);
      installations = drafts;
      defaultInstallKey = drafts[0]?.key ?? null;
    }
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
      const [game, state, meta, roles, dirs] = await Promise.all([
        api.getGame(id),
        api.getGameState(id),
        api.getGameMetadata(id),
        api.listGameCharacters(id).catch(() => []),
        api.listInstallations(id),
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
      lengthMinutes = meta?.length_minutes != null ? String(meta.length_minutes) : '';
      description = meta?.description ?? '';
      developers = (meta?.developers ?? []).join(', ');
      tags = (meta?.tags ?? []).map((tag) => ({ ...tag }));
      staff = (meta?.staff ?? []).map((entry) => ({ ...entry }));
      coverPath = meta?.image_path ?? '';
      characters = roles.map(toDraft);
      installations = dirs.map(toInstallationDraft);
      defaultInstallKey = defaultKeyOf(installations, state.default_installation_id);
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

  function addStaffEntry() {
    staff = [...staff, { role: 'scenario', name: '', original: '', note: null }];
  }

  function removeStaffEntry(index) {
    staff = staff.filter((_, i) => i !== index);
  }


  function formFields() {
    return {
      name,
      alias,
      released,
      rating,
      votecount,
      lengthMinutes,
      description,
      coverPath,
      tags,
      developers,
      staff,
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
    if (pulling) return;
    pulling = true;
    syncProgress = null;
    syncCancelled = false;
    syncNotice = '';
    error = '';
    let off = null;
    try {
      const games = await api.listGames();
      const owner = games.find((g) => g.vndb_id === id && g.id !== gameId);
      if (owner) {
        toast(t('gameForm.error.vndbTaken', { name: owner.name }), 'error');
        return;
      }
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
      vndbId = id;
      incomingCharacters = incoming?.characters ?? [];
      mergeRows = conflictRows(formFields(), incoming);
      charMergeRows = planCharacters(characters.map(toPayload), incomingCharacters).conflicts;
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

  function acceptMerge(picked, charPicked) {
    const next = applyIncoming(formFields(), incoming, picked);
    name = next.name;
    alias = next.alias;
    released = next.released;
    rating = next.rating;
    votecount = next.votecount;
    lengthMinutes = next.lengthMinutes;
    description = next.description;
    tags = next.tags;
    developers = next.developers;
    staff = next.staff;
    coverPath = next.coverPath;
    characters = applyCharacters(characters.map(toPayload), incomingCharacters, charPicked).map(
      toDraft
    );
    mergeOpen = false;
    incoming = null;
    incomingCharacters = [];
  }

  function close() {
    if (saving) return false;
    open = false;
  }

  function validateRanges() {
    ratingError = ratingRejected(rating) ? t('gameForm.error.ratingRange') : '';
    votecountError = votecountRejected(votecount) ? t('gameForm.error.votecountRange') : '';
    lengthMinutesError = lengthMinutesRejected(lengthMinutes)
      ? t('gameForm.error.lengthMinutesRange')
      : '';
    return ratingError || votecountError || lengthMinutesError;
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

    if (installations.some(installPathInvalid)) {
      tab = 'installations';
      return;
    }

    saving = true;
    error = '';
    const patch = buildMetadataPatch({
      alias,
      released,
      rating,
      votecount,
      lengthMinutes,
      description,
      coverPath,
      tags,
      developers,
      staff,
    });
    try {
      const base = {
        status,
        user_rating: userRating,
        notes: notes.trim() || null,
      };
      const entries = installations.map(toInstallationPayload);
      if (gameId) {
        await api.updateGame(gameId, { name: value });
        const dirs = await api.saveInstallations(gameId, entries);
        await api.updateGameState(gameId, {
          ...base,
          default_installation_id: resolveDefaultId(dirs, installations, defaultInstallKey),
        });
        await api.updateGameMetadata(gameId, patch);
        await api.saveGameCharacters(gameId, namedOnly(characters).map(toPayload));
        if (vndbId) await api.updateGame(gameId, { vndb_id: vndbId });
      } else {
        const created = await api.createGame({
          name: value,
          vndb_id: null,
          metadata: patch,
        });
        gameId = created.id;
        const dirs = await api.saveInstallations(created.id, entries);
        await api.updateGameState(created.id, {
          ...base,
          default_installation_id: resolveDefaultId(dirs, installations, defaultInstallKey),
        });
        await api.saveGameCharacters(created.id, namedOnly(characters).map(toPayload));
        if (vndbId) await api.updateGame(created.id, { vndb_id: vndbId });
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
            <SelectDd
              id="game-form-status"
              options={STATUS_OPTIONS}
              value={status}
              open={statusOpen}
              onopenchange={(v) => (statusOpen = v)}
              onchange={(v) => (status = v)}
              disabled={saving}
            />
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

          {#if gameId}
            <div class="panel-grid__full">
              <PlaytimeField {gameId} disabled={saving} />
            </div>
          {/if}
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
            <label for="game-form-meta-length">{t('gameForm.field.length_minutes')}</label>
            <input
              id="game-form-meta-length"
              type="text"
              inputmode="numeric"
              bind:value={lengthMinutes}
              placeholder={t('gameForm.placeholder.lengthMinutes')}
              disabled={saving}
            />
            {#if lengthMinutesError}
              <p class="form-hint form-hint--error">{lengthMinutesError}</p>
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
            <PathField
              id="game-form-cover"
              bind:value={coverPath}
              picker={pickImage}
              pickerTitle={t('gameForm.field.cover')}
              placeholder={t('gameForm.placeholder.cover')}
              ariaLabel={t('gameForm.field.cover')}
              disabled={saving}
            />
          </div>
        </div>
      </div>

      <div class="modal-panel" class:active={tab === 'staff'}>
        <div class="form-group">
          <label for="game-form-staff">{t('gameForm.field.staff')}</label>
          <div class="staff-rows">
            {#each staff as person, i (i)}
              <div class="staff-row">
                <SelectDd
                  options={STAFF_ROLE_OPTIONS}
                  value={person.role}
                  open={staffRoleOpen === i}
                  onopenchange={(v) => (staffRoleOpen = v ? i : null)}
                  onchange={(v) => (person.role = v)}
                  ariaLabel={t('gameForm.field.staffRole')}
                  disabled={saving}
                />
                <input
                  type="text"
                  bind:value={person.original}
                  placeholder={t('gameForm.placeholder.staffOriginal')}
                  disabled={saving}
                  aria-label={t('gameForm.field.staffOriginal')}
                />
                <input
                  type="text"
                  bind:value={person.name}
                  placeholder={t('gameForm.placeholder.staffName')}
                  disabled={saving}
                  aria-label={t('gameForm.field.staffAlias')}
                />
                <button
                  type="button"
                  class="staff-row__remove"
                  aria-label={t('gameForm.action.removeStaff')}
                  onmousedown={(e) => e.preventDefault()}
                  onclick={() => removeStaffEntry(i)}
                  disabled={saving}
                >
                  <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </div>
            {/each}
            <button
              type="button"
              class="staff-rows__add"
              onclick={addStaffEntry}
              disabled={saving}
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
              </svg>
              {t('gameForm.action.addStaff')}
            </button>
          </div>
          <p class="form-hint">{t('gameForm.hint.staff')}</p>
        </div>
      </div>

      <div class="modal-panel" class:active={tab === 'installations'}>
        <InstallationList
          bind:items={installations}
          bind:defaultKey={defaultInstallKey}
          disabled={saving}
        />
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
  charRows={charMergeRows}
  onaccept={acceptMerge}
  oncancel={() => (mergeOpen = false)}
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
