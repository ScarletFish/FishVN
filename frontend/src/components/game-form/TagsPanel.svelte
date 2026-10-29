<script>
  import { t } from '../../lib/i18n.js';
  import { getLibraryTags } from '../../lib/library-tags.svelte.js';
  import { tagKey, tagLabel } from '../../lib/tag-label.svelte.js';
  import { tagCategory } from '../../lib/tag-order.svelte.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import { pickImage } from '../../lib/pick-path.js';
  import PathField from '../PathField.svelte';
  import CoverFocalDialog from './CoverFocalDialog.svelte';
  import TagField from '../TagField.svelte';

  let {
    tags = $bindable([]),
    tagDraft = $bindable(''),
    coverPath = $bindable(''),
    focusX = 50,
    focusY = 50,
    onfocus,
    saving = false,
    onbrowse,
  } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);
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

<div class="form-group">
  <div class="form-label-row">
    <label for="game-form-cover">{t('gameForm.field.cover')}</label>
  </div>
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
      <span class="cover-field__badge">{t('gameForm.action.adjustCover')}</span>
    </button>
    <PathField
      id="game-form-cover"
      bind:value={coverPath}
      picker={pickImage}
      pickerTitle={t('gameForm.field.cover')}
      placeholder={t('gameForm.placeholder.cover')}
      ariaLabel={t('gameForm.field.cover')}
      disabled={saving}
      onPick={() => {
        onfocus?.(50, 50);
        return true;
      }}
    />
  </div>
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
  keyOf={tagKey}
  labelOf={tagLabel}
  groupOf={tagCategory}
  groupOrder={GROUP_ORDER}
  groupLabel={(key) => t(`tagPicker.group.${key}`)}
  hiddenGroups={['tech']}
  {onbrowse}
  disabled={saving}
/>
