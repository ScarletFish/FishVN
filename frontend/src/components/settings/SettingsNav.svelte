<script>
  let { sections = [], active = '', ongoto } = $props();

  let navEl = $state(null);
  let indicator = $state({ top: 0, height: 0, ready: false });

  function place() {
    if (!navEl) return;
    const index = sections.findIndex((section) => section.id === active);
    const node = navEl.querySelectorAll('.settings-nav__item')[index];
    if (!node) return;
    indicator = { top: node.offsetTop, height: node.offsetHeight, ready: true };
  }

  $effect(() => {
    place();
  });

  $effect(() => {
    if (!navEl) return;
    const ro = new ResizeObserver(place);
    ro.observe(navEl);
    return () => ro.disconnect();
  });
</script>

<nav class="settings-nav" bind:this={navEl}>
  <span
    class="settings-nav__indicator"
    class:ready={indicator.ready}
    style:transform={`translateY(calc(${indicator.top}px + var(--space-2)))`}
    style:height={`calc(${indicator.height}px - var(--space-2) * 2)`}
    aria-hidden="true"
  ></span>
  {#each sections as section (section.id)}
    <button
      type="button"
      class="settings-nav__item"
      class:active={active === section.id}
      aria-current={active === section.id ? 'true' : undefined}
      onclick={() => ongoto?.(section.id)}
    >
      {section.label}
    </button>
  {/each}
</nav>

<style>
  .settings-nav {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .settings-nav__indicator {
    position: absolute;
    top: 0;
    left: 0;
    width: 2px;
    background: var(--accent);
    border-radius: var(--radius-pill);
    opacity: 0;
    pointer-events: none;
    transition:
      transform var(--duration-normal) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }

  .settings-nav__indicator.ready {
    opacity: 1;
  }

  .settings-nav__item {
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    font-weight: var(--fw-normal);
    color: var(--fg-muted);
    text-align: left;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  .settings-nav__item:hover {
    color: var(--fg-default);
    background: var(--bg-hover);
  }

  .settings-nav__item.active {
    color: var(--accent);
    font-weight: var(--fw-medium);
  }

  .settings-nav__item:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: -2px;
  }
</style>
