<script>
  import { portal } from '../lib/portal.js';
  import {
    currentEditMenu,
    closeEditMenu,
    placeMenu,
    runEditAction,
  } from '../lib/edit-menu.svelte.js';
  import { holdTips } from '../lib/tip-guard.js';
  import { t } from '../lib/i18n.js';

  const menu = $derived(currentEditMenu());

  const items = $derived.by(() => {
    if (!menu) return [];
    if (menu.kind === 'editable') {
      return [
        { id: 'cut', label: t('editMenu.cut'), disabled: !menu.hasSelection },
        { id: 'copy', label: t('editMenu.copy'), disabled: !menu.hasSelection },
        { id: 'paste', label: t('editMenu.paste') },
        { sep: true },
        { id: 'selectAll', label: t('editMenu.selectAll') },
      ];
    }
    return [
      { id: 'copy', label: t('editMenu.copy') },
      { id: 'selectAll', label: t('editMenu.selectAll') },
    ];
  });

  let el = $state(null);
  let pos = $state({ left: 0, top: 0 });

  $effect(() => {
    const m = menu;
    if (!m) return;
    const width = el?.offsetWidth || 176;
    const height = el?.offsetHeight || 232;
    pos = placeMenu(m.x, m.y, width, height, {
      width: window.innerWidth,
      height: window.innerHeight,
    });
  });

  $effect(() => {
    if (!menu) return;
    return holdTips();
  });

  $effect(() => {
    if (!menu) return;
    const onDown = (event) => {
      if (el && !el.contains(event.target)) closeEditMenu();
    };
    const onKey = (event) => {
      if (event.key === 'Escape') closeEditMenu();
    };
    const onHide = () => closeEditMenu();
    window.addEventListener('mousedown', onDown, true);
    window.addEventListener('keydown', onKey, true);
    window.addEventListener('resize', onHide);
    window.addEventListener('wheel', onHide, true);
    return () => {
      window.removeEventListener('mousedown', onDown, true);
      window.removeEventListener('keydown', onKey, true);
      window.removeEventListener('resize', onHide);
      window.removeEventListener('wheel', onHide, true);
    };
  });
</script>

{#if menu}
  <div
    class="context-menu edit-menu"
    role="menu"
    tabindex="-1"
    bind:this={el}
    style:left="{pos.left}px"
    style:top="{pos.top}px"
    onmousedown={(event) => event.stopPropagation()}
    use:portal
  >
    {#each items as item, index (item.sep ? `sep-${index}` : item.id)}
      {#if item.sep}
        <div class="context-menu__sep"></div>
      {:else}
        <button
          type="button"
          class="menu__item"
          role="menuitem"
          disabled={item.disabled}
          onclick={() => runEditAction(item.id)}>{item.label}</button
        >
      {/if}
    {/each}
  </div>
{/if}

<style>
  .edit-menu {
    position: fixed;
    display: flex;
    flex-direction: column;
    min-width: 10rem;
    margin: 0;
  }

  .menu__item {
    color: var(--fg-default);
  }

  .menu__item:hover:not(:disabled) {
    background: var(--bg-hover-strong);
  }

  .menu__item:disabled {
    color: var(--fg-muted);
    opacity: 0.35;
    cursor: default;
  }
</style>
