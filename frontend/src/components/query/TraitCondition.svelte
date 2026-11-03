<script>
  import { onMount } from 'svelte';
  import { t } from '../../lib/i18n.js';
  import { loadTagDict, traitItems, traitKeyLabel } from '../../lib/tag-label.svelte.js';
  import { loadLibraryTraits, toTraitItems } from '../../lib/library-traits.svelte.js';
  import { TRAIT_GROUP_ORDER } from '../../lib/tag-order.svelte.js';
  import Modal from '../Modal.svelte';
  import TagPicker from '../game-form/TagPicker.svelte';

  let { traits } = $props();

  let pickerOpen = $state(false);
  let pickerTarget = $state(null);

  onMount(() => {
    loadTagDict();
    loadLibraryTraits();
  });

  function targetList() {
    if (pickerTarget === 'exclude') return traits.exclude;
    return traits.groups[pickerTarget] ?? null;
  }

  function openPicker(target) {
    pickerTarget = target;
    pickerOpen = true;
  }

  function toggleItem(key) {
    if (typeof key !== 'string' || !key) return;
    const list = targetList();
    if (!list) return;
    const idx = list.indexOf(key);
    if (idx >= 0) list.splice(idx, 1);
    else list.push(key);
  }

  function removeFrom(list, i) {
    list.splice(i, 1);
  }

  function addGroup() {
    traits.groups.push([]);
  }

  function removeGroup(i) {
    if (traits.groups.length > 1) traits.groups.splice(i, 1);
  }

  const pickerItems = $derived(
    traitItems().map((item) => ({ ...item, group: item.group ?? 'uncategorized' })),
  );

  const libraryPickerItems = $derived(
    toTraitItems().map((item) => ({ ...item, group: item.group ?? 'uncategorized' })),
  );
</script>

<div class="tag-cond">
  <div class="tag-cond__modes" role="radiogroup" aria-label={t('query.mode')}>
    <button
      type="button"
      class="tag-cond__mode"
      class:is-active={traits.mode === 'any'}
      onclick={() => (traits.mode = 'any')}
    >{t('query.modeAny')}</button>
    <button
      type="button"
      class="tag-cond__mode"
      class:is-active={traits.mode === 'all'}
      onclick={() => (traits.mode = 'all')}
    >{t('query.modeAll')}</button>
  </div>

  {#each traits.groups as group, gi (gi)}
    <div class="tag-cond__group">
      <div class="tag-cond__group-head">
        <span class="tag-cond__group-title">{t('query.groupN', { n: gi + 1 })}</span>
        {#if traits.groups.length > 1}
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
        {#each group as key, i (key)}
          <button
            type="button"
            class="tag-cond__chip"
            data-tooltip={t('query.remove')}
            onclick={() => removeFrom(group, i)}
          >{traitKeyLabel(key)}<span class="tag-cond__chip-x">×</span></button>
        {/each}
        <button type="button" class="tag-cond__add" onclick={() => openPicker(gi)}>
          {t('query.pickTrait')}
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
      {#each traits.exclude as key, i (key)}
        <button
          type="button"
          class="tag-cond__chip is-danger"
          data-tooltip={t('query.remove')}
          onclick={() => removeFrom(traits.exclude, i)}
        >{traitKeyLabel(key)}<span class="tag-cond__chip-x">×</span></button>
      {/each}
      <button type="button" class="tag-cond__add" onclick={() => openPicker('exclude')}>
        {t('query.pickTrait')}
      </button>
    </div>
  </div>
</div>

<Modal bind:open={pickerOpen} title={t('query.pickTrait')} variant="modal--picker">
  <TagPicker
    items={pickerItems}
    libraryItems={libraryPickerItems}
    allItems={pickerItems}
    groupOrder={TRAIT_GROUP_ORDER}
    groupLabel={(g) => t('detail.traitGroups.' + g)}
    onToggle={toggleItem}
    onClose={() => (pickerOpen = false)}
  />
</Modal>
