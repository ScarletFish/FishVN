<script>
  import BrandMark from './BrandMark.svelte';
  import { sideTip } from '../lib/side-tip.js';

  let { items = [], bottom = [], current = $bindable(), onnavigate } = $props();
</script>

<aside class="sidebar">
  <div class="sidebar-brand">
    <div class="sidebar-brand-icon" data-tip="FishVN" use:sideTip>
      <BrandMark />
    </div>
  </div>

  <nav class="sidebar-nav">
    {#each items as item (item.id)}
      <button
        class="nav-btn"
        class:active={current === item.id}
        data-tip={item.label}
        aria-label={item.label}
        aria-current={current === item.id ? 'page' : undefined}
        use:sideTip
        onclick={() => {
          current = item.id;
          onnavigate?.();
        }}
      >
        {@html item.icon}
      </button>
    {/each}
  </nav>

  <div class="sidebar-bottom">
    {#each bottom as item (item.id)}
      <button class="nav-btn" data-tip={item.label} aria-label={item.label} use:sideTip onclick={item.onclick}>
        {@html item.icon}
      </button>
    {/each}
  </div>
</aside>