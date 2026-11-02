<script>
  import { t } from '../../lib/i18n.js';
  import { getLibraryTags } from '../../lib/library-tags.svelte.js';
  import { tagLabel } from '../../lib/tag-label.svelte.js';
  import { tagCategory } from '../../lib/tag-order.svelte.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import { pickImage } from '../../lib/pick-path.js';
  import CoverFocalDialog from './CoverFocalDialog.svelte';
  import AmbienceField from './AmbienceField.svelte';
  import TagField from '../TagField.svelte';

  let {
    tags = $bindable([]),
    tagDraft = $bindable(''),
    coverPath = $bindable(''),
    ambiencePath = $bindable(''),
    ambienceMode = $bindable('soft'),
    ambiencePlate = $bindable(false),
    ambienceFocalX = $bindable(50),
    ambienceFocalY = $bindable(22),
    title = '',
    alias = '',
    focusX = 50,
    focusY = 50,
    onfocus,
    saving = false,
    onbrowse,
  } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);
  let coverError = $state('');
  let focalOpen = $state(false);

  const GROUP_ORDER = ['custom', 'theme', 'character', 'setting', 'plot', 'style', 'other', 'tech'];

  function resolveTag(text) {
    const q = text.toLowerCase();
    const hit = getLibraryTags().find(
      (item) =>
        tagLabel(item).toLowerCase() === q ||
        String(item.tag ?? '').toLowerCase() === q ||
        String(item.name ?? '').toLowerCase() === q
    );
    return hit?.id ? { id: hit.id, name: hit.tag ?? hit.name ?? text } : { name: text };
  }

  async function browseCover() {
    if (saving) return;
    coverError = '';
    try {
      const picked = await pickImage(t('gameForm.field.cover'));
      if (!picked) return;
      coverPath = picked;
      onfocus?.(50, 50);
    } catch (err) {
      coverError = err.message;
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
</script>

<div class="media-row">
  <div class="form-group">
    <span class="form-label">{t('gameForm.field.cover')}</span>
    <div class="cover-field">
      <button
        type="button"
        class="cover-field__preview"
        disabled={saving || !coverUrl || coverBroken}
        onclick={() => (focalOpen = true)}
      >
        {#if coverUrl && !coverBroken}
          <img
            src={coverUrl}
            alt=""
            style:object-position="{focusX}% {focusY}%"
            onerror={() => (coverBroken = true)}
          />
        {:else}
          <div class="cover-field__ph">{t('gameForm.placeholder.empty')}</div>
        {/if}
        <span class="media-badge">{t('gameForm.action.adjustFocal')}</span>
      </button>
      <div class="cover-field__actions">
        <button type="button" class="btn" onclick={browseCover} disabled={saving}>
          {t('gameForm.action.replaceCover')}
        </button>
        {#if coverPath.trim()}
          <button
            type="button"
            class="btn btn-danger"
            onclick={() => (coverPath = '')}
            disabled={saving}
          >
            {t('gameForm.action.removeCover')}
          </button>
        {/if}
      </div>
      {#if coverError}
        <p class="form-hint form-hint--error">{coverError}</p>
      {/if}
    </div>
  </div>

  <AmbienceField
    bind:path={ambiencePath}
    bind:mode={ambienceMode}
    bind:plate={ambiencePlate}
    bind:focalX={ambienceFocalX}
    bind:focalY={ambienceFocalY}
    {coverPath}
    {title}
    {alias}
    {saving}
  />
</div>

<CoverFocalDialog
  bind:open={focalOpen}
  imagePath={coverPath}
  {focusX}
  {focusY}
  oncommit={(x, y) => onfocus?.(x, y)}
  onclose={() => (focalOpen = false)}
/>

<TagField
  items={tags}
  onItemsChange={(v) => (tags = v)}
  bind:draft={tagDraft}
  label={t('gameForm.field.tags')}
  browseText={t('gameForm.action.browseTags')}
  placeholder={t('gameForm.placeholder.tags')}
  emptyText={t('gameForm.placeholder.noTags')}
  removeText={t('gameForm.action.removeTag')}
  resolve={resolveTag}
  keyOf={(item) => item?.name ?? ''}
  duplicateText={t('gameForm.hint.tagExists')}
  labelOf={tagLabel}
  groupOf={tagCategory}
  groupOrder={GROUP_ORDER}
  groupLabel={(key) => t(`tagPicker.group.${key}`)}
  hiddenGroups={['tech']}
  {onbrowse}
  disabled={saving}
/>
