<script>
  import { Select } from 'bits-ui';

  let {
    id = undefined,
    options,
    value,
    open,
    onopenchange,
    onchange,
    disabled = false,
    ariaLabel = '',
  } = $props();

  const selected = $derived(options.find((opt) => opt.value === value) ?? options[0]);
</script>

<div class="select-dd" class:is-open={open}>
  <Select.Root
    type="single"
    bind:value={() => value, onchange}
    bind:open={() => open, onopenchange}
    {disabled}
  >
    <Select.Trigger {id}>
      {#snippet child({ props })}
        <button
          {...props}
          type="button"
          class="select-dd-trigger"
          aria-label={ariaLabel || undefined}
        >
          <span class="select-dd-text">{selected.label}</span>
          <svg
            class="select-dd-chevron"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
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
          <svg
            class="select-dd-opt-check"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </Select.Item>
      {/each}
    </Select.ContentStatic>
  </Select.Root>
</div>
