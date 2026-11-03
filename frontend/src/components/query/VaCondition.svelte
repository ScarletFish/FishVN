<script>
  import { onMount } from 'svelte';
  import { t } from '../../lib/i18n.js';
  import { api } from '../../lib/api.js';
  import Modal from '../Modal.svelte';
  import TagPicker from '../game-form/TagPicker.svelte';

  let { va } = $props();

  let pickerOpen = $state(false);
  let staff = $state([]);

  onMount(async () => {
    try {
      staff = await api.listStaff();
    } catch {
      staff = [];
    }
  });

  function staffLabel(s) {
    const base = s.displayName || s.name || s.original || s.id;
    const orig = s.displayOriginal || s.original;
    return orig && orig !== base ? `${base}（${orig}）` : base;
  }

  function labelOf(id) {
    const hit = staff.find((s) => s.id === id);
    return hit ? staffLabel(hit) : id;
  }

  function toggleItem(key) {
    if (typeof key !== 'string' || !key) return;
    const idx = va.values.indexOf(key);
    if (idx >= 0) va.values.splice(idx, 1);
    else va.values.push(key);
  }

  function removeFrom(i) {
    va.values.splice(i, 1);
  }

  const pickerItems = $derived(
    staff.map((s) => ({ key: s.id, label: staffLabel(s), group: 'cv', count: s.roles })),
  );
</script>

<div class="tag-cond">
  <div class="tag-cond__chips">
    {#each va.values as id, i (id)}
      <button
        type="button"
        class="tag-cond__chip"
        data-tooltip={t('query.remove')}
        onclick={() => removeFrom(i)}
      >{labelOf(id)}<span class="tag-cond__chip-x">×</span></button>
    {/each}
    <button type="button" class="tag-cond__add" onclick={() => (pickerOpen = true)}>
      {t('query.pickVa')}
    </button>
  </div>
</div>

<Modal bind:open={pickerOpen} title={t('query.pickVa')} variant="modal--picker">
  <TagPicker
    items={pickerItems}
    allItems={pickerItems}
    groupOrder={['cv']}
    groupLabel={() => t('query.vaGroup')}
    selected={va.values}
    onToggle={toggleItem}
    onClose={() => (pickerOpen = false)}
  />
</Modal>
