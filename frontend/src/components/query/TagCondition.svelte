<script>
  import { onMount } from 'svelte';
  import { t } from '../../lib/i18n.js';
  import { toTagItems, loadLibraryTags } from '../../lib/library-tags.svelte.js';
  import { allTagItems, loadTagDict, tagKeyLabel } from '../../lib/tag-label.svelte.js';
  import { TAG_GROUP_ORDER } from '../../lib/tag-order.svelte.js';
  import Modal from '../Modal.svelte';
  import TagPicker from '../game-form/TagPicker.svelte';

  let { tags } = $props();

  let pickerOpen = $state(false);
  let pickerTarget = $state(null);

  onMount(() => {
    loadTagDict();
    loadLibraryTags();
  });

  function targetList() {
    if (pickerTarget === 'exclude') return tags.exclude;
    return tags.groups[pickerTarget] ?? null;
  }

  function openPicker(target) {
    pickerTarget = target;
    pickerOpen = true;
  }

  function toggleItem(item) {
    const raw = item.raw ?? item;
    const id = raw.id;
    const list = targetList();
    if (!id || !list) return;
    const idx = list.indexOf(id);
    if (idx >= 0) list.splice(idx, 1);
    else list.push(id);
  }

  function removeFrom(list, i) {
    list.splice(i, 1);
  }

  function addGroup() {
    tags.groups.push([]);
  }

  function removeGroup(i) {
    if (tags.groups.length > 1) tags.groups.splice(i, 1);
  }

  const selectedTags = $derived([...tags.groups.flat(), ...tags.exclude]);
</script>

<div class="tag-cond">
  <div class="tag-cond__modes" role="radiogroup" aria-label={t('query.mode')}>
    <button
      type="button"
      class="tag-cond__mode"
      class:is-active={tags.mode === 'any'}
      onclick={() => (tags.mode = 'any')}
    >{t('query.modeAny')}</button>
    <button
      type="button"
      class="tag-cond__mode"
      class:is-active={tags.mode === 'all'}
      onclick={() => (tags.mode = 'all')}
    >{t('query.modeAll')}</button>
  </div>

  {#each tags.groups as group, gi (gi)}
    <div class="tag-cond__group">
      <div class="tag-cond__group-head">
        <span class="tag-cond__group-title">{t('query.groupN', { n: gi + 1 })}</span>
        {#if tags.groups.length > 1}
          <button
            type="button"
            class="tag-cond__group-remove"
            aria-label={t('query.removeGroup')}
            data-tooltip={t('query.removeGroup')}
            onclick={() => removeGroup(gi)}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        {/if}
      </div>
      <div class="tag-cond__chips">
        {#each group as name, i (name)}
          <button
            type="button"
            class="tag-cond__chip"
            data-tooltip={t('query.remove')}
            onclick={() => removeFrom(group, i)}
          >{tagKeyLabel(name)}<span class="tag-cond__chip-x">×</span></button>
        {/each}
        <button type="button" class="tag-cond__add" onclick={() => openPicker(gi)}>
          {t('query.pickTag')}
        </button>
      </div>
    </div>
  {/each}

  <button type="button" class="tag-cond__add-group" onclick={addGroup}>
    {t('query.addGroup')}
  </button>

  <div class="tag-cond__group tag-cond__group--exclude">
    <div class="tag-cond__group-head">
      <span class="tag-cond__group-title">{t('query.exclude')}</span>
    </div>
    <div class="tag-cond__chips">
      {#each tags.exclude as name, i (name)}
        <button
          type="button"
          class="tag-cond__chip is-danger"
          data-tooltip={t('query.remove')}
          onclick={() => removeFrom(tags.exclude, i)}
        >{tagKeyLabel(name)}<span class="tag-cond__chip-x">×</span></button>
      {/each}
      <button type="button" class="tag-cond__add" onclick={() => openPicker('exclude')}>
        {t('query.pickTag')}
      </button>
    </div>
  </div>
</div>

<Modal bind:open={pickerOpen} title={t('query.pickTag')} variant="modal--picker">
  <TagPicker
    items={toTagItems()}
    libraryItems={toTagItems()}
    allItems={allTagItems()}
    groupOrder={TAG_GROUP_ORDER}
    hiddenGroups={['tech']}
    selected={selectedTags}
    onToggle={(_key, item) => toggleItem(item)}
    onClose={() => (pickerOpen = false)}
  />
</Modal>
