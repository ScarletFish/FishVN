<script>
  import { t } from '../../lib/i18n.js';
  import { getLibraryTags, loadLibraryTags } from '../../lib/library-tags.svelte.js';
  import { tagKey, tagLabel } from '../../lib/tag-label.svelte.js';
  import { tagCategory } from '../../lib/tag-order.svelte.js';
  import { panelScrollbars } from '../../lib/scrollbars.js';

  let {
    selected = [],
    extraNames = [],
    saving = false,
    allowCustom = false,
    includeHidden = true,
    emptyText = '',
    ontoggle,
    onadd,
    onclose,
  } = $props();

  const GROUP_ORDER = ['custom', 'theme', 'character', 'setting', 'plot', 'style', 'other', 'tech'];

  let query = $state('');
  let searchEl = $state(null);

  loadLibraryTags();

  const selectedKeys = $derived(new Set(selected));

  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const hit = (item) =>
      !q ||
      tagLabel(item).toLowerCase().includes(q) ||
      String(item.tag ?? '').toLowerCase().includes(q);
    const library = getLibraryTags().filter(
      (item) => includeHidden || tagCategory(item) !== 'tech'
    );
    const live = library.filter(hit).sort((a, b) => (b.count ?? 0) - (a.count ?? 0));
    const owned = new Set(library.flatMap((item) => [tagKey(item), item.tag]).filter(Boolean));
    const mine = extraNames
      .filter((name) => !owned.has(name))
      .map((name) => ({ tag: name, count: null }))
      .filter(hit);
    return [...live, ...mine];
  });

  const groups = $derived.by(() => {
    const buckets = new Map(GROUP_ORDER.map((key) => [key, []]));
    for (const item of matches) buckets.get(tagCategory(item)).push(item);
    return GROUP_ORDER.filter((key) => buckets.get(key).length).map((key) => ({
      key,
      items: buckets.get(key),
    }));
  });

  const freeName = $derived(query.trim());
  const canAddFree = $derived(
    allowCustom &&
      Boolean(freeName) &&
      !selectedKeys.has(freeName) &&
      !matches.some((item) => String(item.tag ?? '').toLowerCase() === freeName.toLowerCase())
  );

  $effect(() => {
    searchEl?.focus();
  });

  function toggle(item) {
    if (saving) return;
    ontoggle?.(tagKey(item), item);
  }

  function addFree() {
    if (saving || !canAddFree) return;
    onadd?.(freeName);
    query = '';
  }
</script>

<div class="modal-panels modal-panels--head-fixed">
  <div class="modal-panel active">
    <div class="tag-picker">
      <div class="tag-picker__head">
        <input
          bind:this={searchEl}
          class="tag-picker__search"
          type="text"
          bind:value={query}
          placeholder={t('tagPicker.search')}
          aria-label={t('tagPicker.search')}
          autocomplete="off"
          disabled={saving}
        />
        {#if !includeHidden}
          <p class="tag-picker__note">{t('tagPicker.techNote')}</p>
        {/if}
      </div>

      <div class="tag-picker__body" use:panelScrollbars>
        <div class="tag-picker__stack">
          {#if canAddFree}
            <button type="button" class="tag-picker__free" onclick={addFree} disabled={saving}>
              {t('tagPicker.addCustom', { name: freeName })}
            </button>
          {/if}

          {#if groups.length === 0}
            <p class="tag-picker__note">
              {freeName ? t('tagPicker.noMatch') : emptyText || t('tagPicker.empty')}
            </p>
          {:else}
            {#each groups as group (group.key)}
              <section class="tag-picker__group">
                <h3 class="tag-picker__title">
                  {t(`tagPicker.group.${group.key}`)}
                  <span class="tag-picker__num">{group.items.length}</span>
                </h3>
                <ul class="tag-picker__list">
                  {#each group.items as item (tagKey(item))}
                    <li>
                      <button
                        type="button"
                        class="tag-pill tag-pill--pick"
                        class:tag-pill--on={selectedKeys.has(tagKey(item))}
                        aria-pressed={selectedKeys.has(tagKey(item))}
                        onclick={() => toggle(item)}
                        disabled={saving}
                      >
                        {tagLabel(item)}
                        {#if item.count != null}
                          <span class="tag-pill__count">×{item.count}</span>
                        {/if}
                      </button>
                    </li>
                  {/each}
                </ul>
              </section>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<div class="modal-actions">
  <button type="button" class="btn" onclick={onclose}>{t('tagPicker.done')}</button>
</div>
