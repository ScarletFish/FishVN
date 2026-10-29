<script>
  import { t } from '../../lib/i18n.js';
  import { panelScrollbars } from '../../lib/scrollbars.js';

  let {
    items = [],
    selected = [],
    groupOrder = [],
    allowCustom = false,
    hiddenGroups = [],
    emptyText = '',
    saving = false,
    searchPlaceholder = t('tagPicker.search'),
    groupLabel = (g) => t('tagPicker.group.' + g),
    onToggle,
    onAdd,
    onClose,
  } = $props();

  let query = $state('');
  let searchEl = $state(null);

  const selectedKeys = $derived(new Set(selected));

  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return items
      .filter((item) => !q || item.label.toLowerCase().includes(q))
      .sort((a, b) => (b.count ?? 0) - (a.count ?? 0) || a.label.localeCompare(b.label));
  });

  const groups = $derived.by(() => {
    const order = groupOrder.length ? groupOrder : [null];
    const buckets = new Map(order.map((key) => [key, []]));
    for (const item of matches) {
      if (hiddenGroups.includes(item.group)) continue;
      const g = item.group ?? null;
      if (!buckets.has(g)) buckets.set(g, []);
      buckets.get(g).push(item);
    }
    return [...buckets.entries()].filter(([, list]) => list.length);
  });

  const freeName = $derived(query.trim());
  const canAddFree = $derived(
    allowCustom &&
      Boolean(freeName) &&
      !selectedKeys.has(freeName) &&
      !items.some((item) => item.label.toLowerCase() === freeName.toLowerCase())
  );

  $effect(() => {
    searchEl?.focus();
  });

  function toggle(item) {
    onToggle?.(item.key, item.raw ?? item);
  }

  function addFree() {
    if (!canAddFree) return;
    onAdd?.(freeName);
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
          placeholder={searchPlaceholder}
          aria-label={searchPlaceholder}
          autocomplete="off"
          disabled={saving}
        />
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
            {#each groups as group (group[0] ?? '__all__')}
              <section class="tag-picker__group">
                {#if group[0] != null}
                  <h3 class="tag-picker__title">
                    {groupLabel(group[0])}
                    <span class="tag-picker__num">{group[1].length}</span>
                  </h3>
                {/if}
                <ul class="tag-picker__list">
                  {#each group[1] as item (item.key)}
                    <li>
                      <button
                        type="button"
                        class="tag-pill tag-pill--pick"
                        class:tag-pill--on={selectedKeys.has(item.key)}
                        aria-pressed={selectedKeys.has(item.key)}
                        onclick={() => toggle(item)}
                        disabled={saving}
                      >
                        {item.label}
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
  <button type="button" class="btn" onclick={onClose}>{t('tagPicker.done')}</button>
</div>
