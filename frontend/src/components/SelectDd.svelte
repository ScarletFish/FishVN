<script>
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
  <button
    {id}
    type="button"
    class="select-dd-trigger"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={ariaLabel || undefined}
    {disabled}
    onclick={() => onopenchange(!open)}
    onblur={() => {
      if (open) onopenchange(false);
    }}
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
  {#if open}
    <div class="select-dd-menu" role="listbox">
      {#each options as opt (opt.value)}
        <button
          type="button"
          role="option"
          aria-selected={value === opt.value}
          class="select-dd-opt"
          class:is-selected={value === opt.value}
          onmousedown={(e) => e.preventDefault()}
          onclick={() => {
            onchange(opt.value);
            onopenchange(false);
          }}
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
        </button>
      {/each}
    </div>
  {/if}
</div>
