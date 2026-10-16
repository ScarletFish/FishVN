<script>
  let { items = [], label, variant = 'btn', open = $bindable(false) } = $props();
</script>

<div class="menu" class:is-open={open} data-variant={variant}>
  <button
    type="button"
    class={variant === 'plain' ? 'menu__trigger' : 'menu__trigger btn btn-ghost'}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label={label}
    onclick={() => (open = !open)}
    onblur={(e) => {
      if (!e.currentTarget.parentElement.contains(e.relatedTarget)) open = false;
    }}
  >
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="5" r="1.8" />
      <circle cx="12" cy="12" r="1.8" />
      <circle cx="12" cy="19" r="1.8" />
    </svg>
  </button>
  {#if open}
    <div class="menu__list" role="menu">
      {#each items as item (item.label)}
        <button
          type="button"
          role="menuitem"
          class="menu__item"
          onclick={() => {
            open = false;
            item.onclick();
          }}
        >
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</div>
