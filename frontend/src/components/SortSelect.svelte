<script>
  import Icon from './Icon.svelte';
  import SelectDd from './SelectDd.svelte';
  import { t } from '../lib/i18n.js';

  let { value = $bindable(), options = null, ariaLabel = null } = $props();

  const label = $derived(ariaLabel ?? t('query.sort'));

  const items = $derived(
    options ?? [
      { value: 'name', label: t('query.sortName') },
      { value: 'vndb', label: t('query.sortVndb') },
      { value: 'user', label: t('query.sortUser') },
      { value: 'added', label: t('query.sortAdded') },
    ]
  );

  const currentLabel = $derived(
    (items.find((opt) => opt.value === value) ?? items[0])?.label ?? ''
  );
</script>

{#snippet sortIcon()}
  <Icon name="sort" />
{/snippet}

<span class="sort-select">
  <span class="sort-select__value">{currentLabel}</span>
  <SelectDd
    options={items}
    bind:value
    iconOnly
    triggerIcon={sortIcon}
    ariaLabel={label}
    tooltip={label}
  />
</span>

<style>
  .sort-select {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .sort-select__value {
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }

  .sort-select :global(.select-dd-trigger) {
    height: 28px;
  }
</style>
