<script>
  import { panelScrollbars } from '../lib/scrollbars.js';

  let {
    items = [],
    draft = $bindable(''),
    onItemsChange,
    label,
    browseText,
    placeholder,
    emptyText,
    removeText,
    resolve,
    keyOf,
    labelOf,
    groupOf,
    groupOrder = [],
    groupLabel,
    hiddenGroups = [],
    onbrowse,
    duplicateText = '',
    disabled = false,
  } = $props();

  let listEl = $state(null);

  const draftDuplicate = $derived.by(() => {
    if (!duplicateText || disabled || typeof resolve !== 'function') return false;
    const value = draft.trim();
    if (!value) return false;
    const key = keyOf(resolve(value));
    return items.some((item) => keyOf(item) === key);
  });

  function pulse(key) {
    const el = listEl?.querySelector(`[data-pill-key="${CSS.escape(key)}"]`);
    if (!el) return;
    el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    el.classList.remove('tag-pill--pulse');
    void el.offsetWidth;
    el.classList.add('tag-pill--pulse');
  }

  const groupRank = (key) => {
    const i = groupOrder.indexOf(key);
    return i < 0 ? groupOrder.length : i;
  };

  const groups = $derived.by(() => {
    if (!items?.length) return [];
    const buckets = new Map();
    for (const item of items) {
      const key = groupOf(item) || 'uncategorized';
      if (hiddenGroups.includes(key)) continue;
      if (!buckets.has(key)) buckets.set(key, []);
      buckets.get(key).push(item);
    }
    return [...buckets.entries()]
      .sort((a, b) => groupRank(a[0]) - groupRank(b[0]))
      .map(([key, list]) => ({ key, items: list }));
  });

  function addItem() {
    const value = draft.trim();
    if (!value) return;
    const next = resolve(value);
    const key = keyOf(next);
    const existing = items.find((item) => keyOf(item) === key);
    if (existing) {
      pulse(keyOf(existing));
      return;
    }
    onItemsChange([...items, next]);
    draft = '';
  }

  function onKeydown(e) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addItem();
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      draft = '';
      e.currentTarget.blur();
      return;
    }
  }

  function removeItem(item) {
    const key = keyOf(item);
    onItemsChange(items.filter((cur) => keyOf(cur) !== key));
  }
</script>

<div class="form-group">
  <div class="form-label-row">
    <div class="form-label">{label}</div>
    {#if onbrowse}
      <button type="button" class="btn tag-field__browse" onclick={onbrowse} {disabled}>
        {browseText}
      </button>
    {/if}
  </div>
  <div class="tag-input tag-input--grouped" bind:this={listEl} use:panelScrollbars>
    <div class="tag-input__flow tag-input__flow--stacked">
      {#each groups as group (group.key)}
        <section class="tag-field__group">
          <h3 class="tag-field__group-title">
            {groupLabel(group.key)}
            <span class="tag-field__group-num">{group.items.length}</span>
          </h3>
          <div class="tag-field__group-list">
            {#each group.items as item (keyOf(item))}
              <span
                class="tag-pill"
                data-pill-key={keyOf(item)}
                onanimationend={(e) => e.currentTarget.classList.remove('tag-pill--pulse')}
              >
                {labelOf(item)}
                <button
                  type="button"
                  class="tag-pill__remove"
                  aria-label={removeText}
                  onmousedown={(e) => e.preventDefault()}
                  onclick={() => removeItem(item)}
                  {disabled}
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
        <p class="tag-field__empty">{emptyText}</p>
      {/each}
    </div>
  </div>
  <input
    class="tag-input__new"
    type="text"
    bind:value={draft}
    onkeydown={onKeydown}
    {placeholder}
    aria-label={label}
    {disabled}
  />
  {#if draftDuplicate}
    <p class="form-hint">{duplicateText}</p>
  {/if}
</div>
