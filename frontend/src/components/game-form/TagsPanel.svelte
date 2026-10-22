<script>
  import { t } from '../../lib/i18n.js';
  import { tagLabel } from '../../lib/tag-label.svelte.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import { pickImage } from '../../lib/pick-path.js';
  import PathField from '../PathField.svelte';

  let {
    tags = $bindable([]),
    tagDraft = $bindable(''),
    tagInputOpen = $bindable(false),
    coverPath = $bindable(''),
    saving = false,
  } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);

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

  function addTag() {
    const value = tagDraft.trim();
    if (!value) return;
    if (!tags.some((tag) => tag.name === value)) tags = [...tags, { name: value }];
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
</script>

<div class="form-group">
  <label for="game-form-tags">{t('gameForm.field.tags')}</label>
  <div class="tag-input scroll-thin">
    {#each tags as tag, i (tag.id ?? tag.name)}
      <span class="tag-pill">
        {tagLabel(tag)}
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
