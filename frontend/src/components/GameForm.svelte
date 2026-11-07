<script>
  import { tick, untrack } from 'svelte';
  import { Tabs } from 'bits-ui';
  import Modal from './Modal.svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { createVndbSearch } from '../lib/vndb-search.svelte.js';
  import { createVnMergeFlow } from '../lib/vn-merge-flow.svelte.js';
  import { buildMetadataPatch, buildStatePatch, clampRating } from '../lib/form-metadata.js';
  import { errorFieldId, errorTab, shownErrors, validateGameForm } from '../lib/form-validation.js';
  import { DEFAULT_STATUS } from '../lib/status.js';
  import VnMergeDialog from './VnMergeDialog.svelte';
  import CharacterEditor from './CharacterEditor.svelte';
  import InstallationList from './InstallationList.svelte';
  import { namedOnly, toDraft, toPayload } from '../lib/characters.js';
  import {
    defaultKeyOf,
    resolveDefaultId,
    toInstallationDraft,
    toInstallationPayload,
  } from '../lib/installations.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { quitGuard } from '../lib/quit-guard.svelte.js';
  import { notifyLibraryChanged, notifyLibraryTagsChanged, notifyLibraryTraitsChanged } from '../lib/library-events.js';
  import { tagLabel, allTagItems, loadTagDict, traitItems } from '../lib/tag-label.svelte.js';
  import { TAG_GROUP_ORDER } from '../lib/tag-order.svelte.js';

  const TRAIT_GROUP_ORDER = [
    'Hair',
    'Eyes',
    'Body',
    'Clothes',
    'Items',
    'Personality',
    'Role',
    'Engages in',
    'Subject of',
  ];
  import { toTagItems, loadLibraryTags } from '../lib/library-tags.svelte.js';
  import { toTraitItems, loadLibraryTraits } from '../lib/library-traits.svelte.js';
  import { BANNER_FOCAL } from '../lib/cover-focal.js';
  import BasicPanel from './game-form/BasicPanel.svelte';
  import ProfilePanel from './game-form/ProfilePanel.svelte';
  import TagsPanel from './game-form/TagsPanel.svelte';
  import TagPicker from './game-form/TagPicker.svelte';
  import VndbAside from './game-form/VndbAside.svelte';

  let {
    open = $bindable(false),
    gameId = $bindable(null),
    initialTab = 'basic',
    tab = $bindable('basic'),
    prefill = null,
  } = $props();

  const TABS = ['basic', 'profile', 'tags', 'installations', 'characters'];

  let name = $state('');
  let vndbId = $state(null);
  let status = $state(DEFAULT_STATUS);
  let userRating = $state(null);
  let userRatingDraft = $state('');
  let playSeconds = $state(0);
  let playSecondsLoaded = $state(0);
  let notes = $state('');
  let saving = $state(false);
  let loading = $state(false);
  let input = $state(null);
  let submitAttempted = $state(false);
  let touched = $state({});

  let alias = $state('');
  let released = $state('');
  let rating = $state('');
  let votecount = $state('');
  let lengthMinutes = $state('');
  let description = $state('');
  let developers = $state('');
  let tags = $state([]);
  let staff = $state([]);
  let coverPath = $state('');
  let coverFocalX = $state(50);
  let coverFocalY = $state(50);
  let ambiencePath = $state('');
  let ambienceMode = $state('soft');
  let ambiencePlate = $state(false);
  let ambienceFocalX = $state(BANNER_FOCAL.x);
  let ambienceFocalY = $state(BANNER_FOCAL.y);

  let tagDraft = $state('');
  let characters = $state([]);
  let installations = $state([]);
  let defaultInstallKey = $state(null);
  
  let mergeOpen = $state(false);

  let statusOpen = $state(false);
  let staffRoleOpen = $state(null);

  let asideOpen = $state(false);
  let asideMode = $state('search');
  let selectedCharId = $state(null);
  let baseline = $state('');
  const search = createVndbSearch();
  const flow = createVnMergeFlow();

  const formErrors = $derived(
    validateGameForm({ name, rating, votecount, lengthMinutes, installations })
  );
  const shown = $derived(shownErrors(formErrors, { submitAttempted, touched }));
  const visibleError = (field) => (shown[field] ? t(shown[field]) : '');
  function touch(field) {
    if (touched[field]) return;
    touched = { ...touched, [field]: true };
  }

  const customTagNames = $derived(tags.filter((tag) => !tag.id).map((tag) => tag.name));

  const customTagItems = $derived(
    customTagNames.map((name) => ({ key: name, label: name, group: null, count: null, raw: { name } }))
  );

  const tagPickerLibrary = $derived.by(() => {
    const seen = new Set(customTagNames);
    return [
      ...customTagItems,
      ...toTagItems()
        .map((it) => ({ ...it, key: it.label }))
        .filter((it) => !seen.has(it.key)),
    ];
  });

  const tagPickerAll = $derived.by(() => {
    const seen = new Set(customTagNames);
    return [
      ...customTagItems,
      ...allTagItems()
        .map((it) => ({ ...it, key: it.label }))
        .filter((it) => !seen.has(it.key)),
    ];
  });

  const selectedTraits = $derived(
    characters.find((c) => c.id === selectedCharId)?.traits ?? []
  );
  const customTraitItems = $derived(
    selectedTraits
      .filter((tr) => !tr.id)
      .map((tr) => ({ key: tr.name, label: tr.name, group: null, raw: { id: null, name: tr.name, group: null } }))
  );

  function selectTab(next) {
    tab = next;
    asideOpen = false;
  }

  function formSnapshot() {
    return JSON.stringify({
      name,
      vndbId,
      status,
      userRating,
      playSeconds,
      notes,
      alias,
      released,
      rating,
      votecount,
      lengthMinutes,
      description,
      developers,
      tags,
      staff,
      coverPath,
      coverFocalX,
      coverFocalY,
      ambiencePath,
      ambienceMode,
      ambiencePlate,
      ambienceFocalX,
      ambienceFocalY,
      characters,
      installations,
      defaultInstallKey,
    });
  }

  function toggleTag(key, item) {
    const label = tagLabel(item);
    if (tags.some((tag) => tagLabel(tag) === label)) {
      tags = tags.filter((tag) => tagLabel(tag) !== label);
      return;
    }
    tags = [...tags, item.id ? { id: item.id, name: item.tag } : { name: item.tag }];
  }

  function toggleTrait(key, raw) {
    const idx = characters.findIndex((c) => c.id === selectedCharId);
    if (idx < 0) return;
    const list = characters[idx].traits ?? [];
    const next = list.some((tr) => (tr.id ?? tr.name) === key)
      ? list.filter((tr) => (tr.id ?? tr.name) !== key)
      : [
          ...list,
          { id: raw?.id ?? null, name: raw?.name ?? key, group_name: raw?.group ?? null, spoiler: 0, lie: false },
        ];
    characters = characters.map((c, i) => (i === idx ? { ...c, traits: next } : c));
  }

  function addTraitName(name) {
    const idx = characters.findIndex((c) => c.id === selectedCharId);
    if (idx < 0) return;
    const list = characters[idx].traits ?? [];
    if (list.some((tr) => (tr.name ?? '').toLowerCase() === name.toLowerCase())) return;
    const next = [...list, { id: null, name, group_name: null, spoiler: 0, lie: false }];
    characters = characters.map((c, i) => (i === idx ? { ...c, traits: next } : c));
  }

  $effect(() => {
    if (!open || saving) return;
    baseline = '';
    name = '';
    vndbId = null;
    status = DEFAULT_STATUS;
    userRating = null;
    userRatingDraft = '';
    playSeconds = 0;
    playSecondsLoaded = 0;
    notes = '';
    submitAttempted = false;
    touched = {};
    released = '';
    alias = '';
    rating = '';
    votecount = '';
    lengthMinutes = '';
    description = '';
    developers = '';
    tags = [];
    staff = [];
    coverPath = '';
    coverFocalX = 50;
    coverFocalY = 50;
    ambiencePath = '';
    ambienceMode = 'soft';
    ambiencePlate = false;
    ambienceFocalX = BANNER_FOCAL.x;
    ambienceFocalY = BANNER_FOCAL.y;
    tagDraft = '';
    characters = [];
    installations = [];
    defaultInstallKey = null;
    tab = initialTab;
    statusOpen = false;
    staffRoleOpen = null;
    asideOpen = false;
    asideMode = 'search';
    selectedCharId = null;
    mergeOpen = false;
    flow.reset();
    search.reset();
    loading = Boolean(gameId);
    loadTagDict();
    loadLibraryTags();
    if (gameId) {
      loadExisting(gameId);
    } else if (prefill) {
      name = prefill.name;
      const drafts = prefill.installations.map(toInstallationDraft);
      installations = drafts;
      defaultInstallKey = drafts[0]?.key ?? null;
      baseline = untrack(formSnapshot);
    } else {
      baseline = untrack(formSnapshot);
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
      playSeconds = state.play_seconds ?? 0;
      playSecondsLoaded = playSeconds;
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
      coverFocalX = meta?.cover_focal_x ?? 50;
      coverFocalY = meta?.cover_focal_y ?? 50;
      ambiencePath = meta?.ambience_path ?? '';
      ambienceMode = meta?.ambience_mode ?? 'soft';
      ambiencePlate = meta?.ambience_plate ?? false;
      ambienceFocalX = meta?.ambience_focal_x ?? BANNER_FOCAL.x;
      ambienceFocalY = meta?.ambience_focal_y ?? BANNER_FOCAL.y;
      characters = roles.map(toDraft);
      installations = dirs.map(toInstallationDraft);
      defaultInstallKey = defaultKeyOf(installations, state.default_installation_id);
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      loading = false;
      baseline = formSnapshot();
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
    const ok = await flow.pick(id, () => ({
      ownId: gameId,
      current: formFields(),
      characters: characters.map(toPayload),
    }));
    if (!ok) return;
    vndbId = flow.selectedId;
    mergeOpen = true;
  }

  async function cancelSync() {
    await flow.cancel();
  }

  function acceptMerge(picked, charPicked) {
    const result = flow.accept(picked, charPicked);
    mergeOpen = false;
    if (!result) return;
    const next = result.fields;
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
    if (next.coverPath !== coverPath) {
      coverPath = next.coverPath;
      coverFocalX = 50;
      coverFocalY = 50;
    }
    characters = result.characters.map(toDraft);
  }

  const isDirty = $derived(baseline !== '' && formSnapshot() !== baseline);

  $effect(() => {
    quitGuard.dirty = isDirty;
  });

  async function close() {
    if (saving) return false;
    if (isDirty) {
      let ok = false;
      try {
        ok = await confirm('discardChanges');
      } catch {
        return false;
      }
      if (!ok) return false;
    }
    open = false;
    return true;
  }

  async function submit(e) {
    e.preventDefault();
    if (saving || loading) return;

    submitAttempted = true;
    const errors = validateGameForm({ name, rating, votecount, lengthMinutes, installations });
    if (Object.keys(errors).length) {
      const tabWithError = errorTab(errors);
      if (tabWithError) selectTab(tabWithError);
      await tick();
      const id = errorFieldId(errors);
      if (id) document.getElementById(id)?.focus();
      return;
    }

    saving = true;
    const value = name.trim();
    if (userRating !== null && clampRating(userRating) !== userRating) {
      userRating = clampRating(userRating);
      userRatingDraft = String(userRating);
    }
    const patch = buildMetadataPatch({
      alias,
      released,
      rating,
      votecount,
      lengthMinutes,
      description,
      coverPath,
      focusX: coverFocalX,
      focusY: coverFocalY,
      ambiencePath,
      ambienceMode,
      ambiencePlate,
      ambienceFocalX,
      ambienceFocalY,
      tags,
      developers,
      staff,
    });
    try {
      const base = buildStatePatch({
        status,
        userRating,
        notes,
        playSeconds: playSeconds === playSecondsLoaded ? undefined : playSeconds,
      });
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
      notifyLibraryChanged();
      notifyLibraryTagsChanged();
      notifyLibraryTraitsChanged();
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
  asideTitle={asideMode === 'tags' ? t('tagPicker.title') : asideMode === 'traits' ? t('tagPicker.titleTraits') : t('gameForm.search.title')}
  onclose={close}
>
  <form class="form-panel" onsubmit={submit}>
    <Tabs.Root bind:value={() => tab, selectTab}>
      <Tabs.List class="modal-tabs">
        {#each TABS as key (key)}
          <Tabs.Trigger
            value={key}
            class={tab === key ? 'modal-tab active' : 'modal-tab'}
          >
            {t(`gameForm.tab.${key}`)}
          </Tabs.Trigger>
        {/each}
      </Tabs.List>
    </Tabs.Root>

    <div class="modal-panels" use:panelScrollbars>
      <div class="modal-panel" class:active={tab === 'basic'}>
        <BasicPanel
          bind:name
          bind:status
          bind:statusOpen
          bind:userRating
          bind:userRatingDraft
          bind:playSeconds
          bind:notes
          bind:description
          bind:inputEl={input}
          nameError={visibleError('name')}
          ontouch={touch}
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
          bind:staff
          bind:staffRoleOpen
          ratingError={visibleError('rating')}
          votecountError={visibleError('votecount')}
          lengthMinutesError={visibleError('lengthMinutes')}
          ontouch={touch}
          {saving}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'tags'}>
        <TagsPanel
          bind:tags
          bind:tagDraft
          bind:coverPath
          bind:ambiencePath
          bind:ambienceMode
          bind:ambiencePlate
          bind:ambienceFocalX
          bind:ambienceFocalY
          title={name}
          {alias}
          focusX={coverFocalX}
          focusY={coverFocalY}
          onfocus={(x, y) => {
            coverFocalX = x;
            coverFocalY = y;
          }}
          {saving}
          onbrowse={() => {
            loadLibraryTags();
            loadTagDict();
            asideMode = 'tags';
            asideOpen = true;
          }}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'installations'}>
        <InstallationList
          bind:items={installations}
          bind:defaultKey={defaultInstallKey}
          revealAll={submitAttempted}
          disabled={saving}
        />
      </div>

      <div class="modal-panel" class:active={tab === 'characters'}>
        <CharacterEditor
          bind:characters
          bind:selectedId={selectedCharId}
          disabled={saving}
          onTraitBrowse={() => {
            loadTagDict();
            loadLibraryTraits();
            asideMode = 'traits';
            asideOpen = true;
          }}
        />
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
        libraryItems={tagPickerLibrary}
        allItems={tagPickerAll}
        selected={tags.map(tagLabel)}
        groupOrder={TAG_GROUP_ORDER}
        hiddenGroups={['tech']}
        allowCustom
        {saving}
        onToggle={toggleTag}
        onAdd={(name) => {
          if (!tags.some((tag) => tagLabel(tag) === name)) tags = [...tags, { name }];
        }}
        onClose={() => (asideOpen = false)}
      />
    {:else if asideMode === 'traits'}
      <TagPicker
        libraryItems={[...customTraitItems, ...toTraitItems()]}
        allItems={[...customTraitItems, ...traitItems()]}
        selected={selectedTraits.map((tr) => tr.id ?? tr.name)}
        groupOrder={TRAIT_GROUP_ORDER}
        groupLabel={(g) => t('detail.traitGroups.' + g)}
        allowCustom
        searchPlaceholder={t('tagPicker.searchTraits')}
        {saving}
        onToggle={(key, raw) => toggleTrait(key, raw)}
        onAdd={(name) => addTraitName(name)}
        onClose={() => (asideOpen = false)}
      />
    {:else}
      <VndbAside
        {search}
        pulling={flow.pulling}
        syncProgress={flow.progress}
        syncNotice={flow.notice}
        syncCancelled={flow.cancelled}
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
  rows={flow.rows}
  charRows={flow.charRows}
  coverFocalX={coverFocalX}
  coverFocalY={coverFocalY}
  onaccept={acceptMerge}
  oncancel={() => (mergeOpen = false)}
/>

<style>
  .game-form__sync {
    margin-right: auto;
  }
</style>