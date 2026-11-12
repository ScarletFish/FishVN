<script>
  import Icon from './Icon.svelte';
  import { Select } from 'bits-ui';

  let {
    id = undefined,
    options,
    value = $bindable(undefined),
    open = undefined,
    onopenchange = () => {},
    onchange,
    disabled = false,
    ariaLabel = '',
    tooltip = '',
    iconOnly = false,
    triggerIcon = null,
  } = $props();

  let internalOpen = $state(false);
  const isOpen = $derived(open ?? internalOpen);

  const selected = $derived(options.find((opt) => opt.value === value) ?? options[0]);

</script>

<div class="select-dd" class:select-dd--icon={iconOnly} class:is-open={isOpen}>
  <Select.Root
    type="single"
    bind:value={() => value, (v) => { value = v; onchange?.(v); }}
    bind:open={() => isOpen, (v) => { internalOpen = v; onopenchange(v); }}
    {disabled}
  >
    <Select.Trigger {id}>
      {#snippet child({ props })}
        <button
          {...props}
          type="button"
          class="select-dd-trigger"
          aria-label={ariaLabel || undefined}
          data-tooltip={tooltip || undefined}
        >
          {#if triggerIcon}
            {@render triggerIcon()}
          {:else}
            <span class="select-dd-text">{selected.label}</span>
            <Icon name="caret-down" />
          {/if}
        </button>
      {/snippet}
    </Select.Trigger>
    <Select.ContentStatic class="select-dd-menu">
      {#each options as opt (opt.value)}
        <Select.Item
          value={opt.value}
          label={opt.label}
          class={value === opt.value ? 'select-dd-opt is-selected' : 'select-dd-opt'}
        >
          <span>{opt.label}</span>
          <Icon name="check" class="select-dd-opt-check" />
        </Select.Item>
      {/each}
    </Select.ContentStatic>
  </Select.Root>
</div>
