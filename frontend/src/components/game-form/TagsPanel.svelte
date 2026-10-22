<script>
  import { t } from '../../lib/i18n.js';
  import { getLibraryTags } from '../../lib/library-tags.svelte.js';
  import { tagKey, tagLabel } from '../../lib/tag-label.svelte.js';
  import { tagCategory } from '../../lib/tag-order.svelte.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import { pickImage } from '../../lib/pick-path.js';
  import PathField from '../PathField.svelte';

  let {
    tags = $bindable([]),
    tagDraft = $bindable(''),
    coverPath = $bindable(''),
    saving = false,
    onbrowse,
  } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);

  const GROUP_ORDER = ['custom', 'theme', 'character', 'setting', 'plot', 'style', 'other', 'tech'];

  const groups = $derived.by(() => {
    const buckets = new Map(GROUP_ORDER.map((key) => [key, []]));
    for (const tag of tags) buckets.get(tagCategory(tag)).push(tag);
    return GROUP_ORDER.filter((key) => buckets.get(key).length).map((key) => ({
      key,
      items: buckets.get(key),
    }));
  });

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

  function findLibraryTag(text) {
    const q = text.toLowerCase();
    return getLibraryTags().find(
      (item) =>
        tagLabel(item).toLowerCase() === q ||
        String(item.tag ?? '').toLowerCase() === q ||
        String(item.name ?? '').toLowerCase() === q
    );
  }

  function addTag() {
    const value = tagDraft.trim();
    if (!value) return;
    const hit = findLibraryTag(value);
    const next = hit?.id ? { id: hit.id, name: hit.tag ?? hit.name ?? value } : { name: value };
    const key = tagKey(next);
    if (!tags.some((tag) => tagKey(tag) === key)) tags = [...tags, next];
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
      e.currentTarget.blur();
      return;
    }
    if (e.key === 'Backspace' && !tagDraft && tags.length) {
      e.preventDefault();
      removeTag(tags[tags.length - 1]);
    }
  }

  function removeTag(tag) {
    tags = tags.filter((item) => item !== tag);
  }
</script>

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

<div class="form-group">
  <div class="form-label-row">
    <div class="form-label">{t('gameForm.field.tags')}</div>
    <button type="button" class="btn tag-field__browse" onclick={onbrowse} disabled={saving}>
      {t('gameForm.action.browseTags')}
    </button>
  </div>
  <div class="tag-input tag-input--grouped scroll-thin">
    {#each groups as group (group.key)}
      <section class="tag-field__group">
        <h3 class="tag-field__group-title">
          {t(`tagPicker.group.${group.key}`)}
          <span class="tag-field__group-num">{group.items.length}</span>
        </h3>
        <div class="tag-field__group-list">
          {#each group.items as tag (tagKey(tag))}
            <span class="tag-pill">
              {tagLabel(tag)}
              <button
                type="button"
                class="tag-pill__remove"
                aria-label={t('gameForm.action.removeTag')}
                onmousedown={(e) => e.preventDefault()}
                onclick={() => removeTag(tag)}
              >
                <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </span>
          {/each}
        </div>
      </section>
    {:else}
      <p class="tag-field__empty">{t('gameForm.placeholder.noTags')}</p>
    {/each}
  </div>
  <div class="tag-field__foot">
    <input
      id="game-form-tag-new"
      class="tag-input__new"
      type="text"
      bind:value={tagDraft}
      onkeydown={onTagNewKeydown}
      placeholder={t('gameForm.placeholder.tags')}
      aria-label={t('gameForm.field.tags')}
      disabled={saving}
    />
    <button
      type="button"
      class="btn btn-ghost"
      onclick={addTag}
      disabled={saving || !tagDraft.trim()}
    >
      {t('gameForm.action.addTag')}
    </button>
  </div>
</div>
