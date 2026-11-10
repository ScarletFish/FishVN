<script>
  let {
    value = $bindable(),
    options = [],
    role = 'group',
    label,
    onchange,
  } = $props();
</script>

<div class="seg" role={role} aria-label={label}>
  {#each options as opt (opt.value)}
    <button
      type="button"
      class="seg__item"
      class:is-active={value === opt.value}
      role={role === 'tablist' ? 'tab' : undefined}
      aria-pressed={role === 'tablist' ? undefined : value === opt.value}
      aria-selected={role === 'tablist' ? value === opt.value : undefined}
      disabled={opt.disabled}
      data-tooltip={opt.tooltip}
      onclick={() => {
        if (opt.disabled) return;
        value = opt.value;
        onchange?.(opt.value);
      }}
    >
      {opt.label}{#if opt.count != null}<b>{opt.count}</b>{/if}
    </button>
  {/each}
</div>
