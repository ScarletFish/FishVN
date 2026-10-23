<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { subscribe } from '../lib/events.js';
  import { createVndbSearch } from '../lib/vndb-search.svelte.js';
  import {
    buildMetadataPatch,
    lengthMinutesRejected,
    ratingRejected,
    votecountRejected,
  } from '../lib/form-metadata.js';
  import { applyIncoming, conflictRows } from '../lib/vn-merge.js';
  import VnMergeDialog from './VnMergeDialog.svelte';
  import CharacterEditor from './CharacterEditor.svelte';
  import InstallationList from './InstallationList.svelte';
  import { namedOnly, toDraft, toPayload } from '../lib/characters.js';
  import {
    defaultKeyOf,
    installPathInvalid,
    resolveDefaultId,
    toInstallationDraft,
    toInstallationPayload,
  } from '../lib/installations.js';
  import { applyCharacters, planCharacters } from '../lib/char-merge.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';
  import { notifyLibraryTagsChanged } from '../lib/library-events.js';
  import { tagKey } from '../lib/tag-label.svelte.js';
  import BasicPanel from './game-form/BasicPanel.svelte';
  import ProfilePanel from './game-form/ProfilePanel.svelte';
  import TagsPanel from './game-form/TagsPanel.svelte';
  import TagPicker from './game-form/TagPicker.svelte';
  import StaffPanel from './game-form/StaffPanel.svelte';
  import VndbAside from './game-form/VndbAside.svelte';

  let {
    open = $bindable(false),
    gameId = $bindable(null),
    onsaved,
    initialTab = 'basic',
    prefill = null,
  } = $props();

  const TABS = ['basic', 'profile', 'tags', 'staff', 'installations', 'characters'];

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

  let tab = $state('basic');
  let statusOpen = $state(false);
  let staffRoleOpen = $state(null);

  let asideOpen = $state(false);
  let asideMode = $state('search');
  const search = createVndbSearch();

  const customTagNames = $derived(tags.filter((tag) => !tag.id).map((tag) => tag.name));

  function toggleTag(key, item) {
    if (tags.some((tag) => tagKey(tag) === key)) {
      tags = tags.filter((tag) => tagKey(tag) !== key);
      return;
    }
    tags = [...tags, item.id ? { id: item.id, name: item.tag } : { name: item.tag }];
  }

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
    tagDraft = '';
    characters = [];
    installations = [];
    defaultInstallKey = null;
    tab = initialTab;
    statusOpen = false;
    staffRoleOpen = null;
    asideOpen = false;
    asideMode = 'search';
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
      notifyLibraryTagsChanged();
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
  bind:asideOpen
  title={gameId ? t('common.actionEdit') : t('gameForm.title')}
  variant="modal--form"
  asideTitle={asideMode === 'tags' ? t('tagPicker.title') : t('gameForm.search.title')}
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

    <div class="modal-panels" use:panelScrollbars>
      <div class="modal-panel" class:active={tab === 'basic'}>
        <BasicPanel
          bind:name
          bind:status
          bind:statusOpen
          bind:userRating
          bind:userRatingDraft
          bind:notes
          bind:description
          bind:inputEl={input}
          {error}
          {saving}
          {gameId}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'profile'}>
        <ProfilePanel
          bind:alias
          bind:released
          bind:rating
          bind:votecount
          bind:lengthMinutes
          bind:developers
          {ratingError}
          {votecountError}
          {lengthMinutesError}
          {saving}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'tags'}>
        <TagsPanel
          bind:tags
          bind:tagDraft
          bind:coverPath
          {saving}
          onbrowse={() => {
            asideMode = 'tags';
            asideOpen = true;
          }}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'staff'}>
        <StaffPanel bind:staff bind:staffRoleOpen {saving} />
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
          asideMode = 'search';
          asideOpen = true;
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
    {#if asideMode === 'tags'}
      <TagPicker
        selected={tags.map(tagKey)}
        extraNames={customTagNames}
        allowCustom
        {saving}
        ontoggle={toggleTag}
        onadd={(name) => (tags = [...tags, { name }])}
        onclose={() => (asideOpen = false)}
      />
    {:else}
      <VndbAside
        {search}
        {pulling}
        {syncProgress}
        {syncNotice}
        {syncCancelled}
        selectedId={vndbId}
        onpick={pickHit}
        oncancel={cancelSync}
        onclose={() => (asideOpen = false)}
      />
    {/if}
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
</style>