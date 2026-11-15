<script>
  import Icon from '../Icon.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import SearchBar from './SearchBar.svelte';
  import Menu from '../Menu.svelte';
  import { getAddGameMenuItems } from '../../lib/add-game.svelte.js';
  import { ADD_MENU_SUMMON } from '../../lib/library-events.js';
  import { t } from '../../lib/i18n.js';

  let {
    mode = 'default',
    onback,
    onsettings,
    sidebarCollapsed = false,
    ontogglesidebar,
  } = $props();

  let addMenuOpen = $state(false);

  $effect(() => {
    function onSummon() {
      addMenuOpen = true;
    }

    window.addEventListener(ADD_MENU_SUMMON, onSummon);
    return () => window.removeEventListener(ADD_MENU_SUMMON, onSummon);
  });

  let tauriWin = null;
  try {
    if (window.__TAURI_INTERNALS__) tauriWin = getCurrentWindow();
  } catch (_) {}

  let maximized = $state(false);

  function updateMaximizeIcon() {
    if (!tauriWin) return;
    tauriWin
      .isMaximized()
      .then((v) => (maximized = v))
      .catch(() => {});
  }

  $effect(() => {
    if (!tauriWin) return;
    updateMaximizeIcon();
    let unlisten = null;
    tauriWin
      .onResized(() => updateMaximizeIcon())
      .then((fn) => (unlisten = fn))
      .catch(() => {});
    return () => unlisten?.();
  });

  function onTitlebarMouseDown(e) {
    if (!tauriWin) return;
    if (e.button !== 0) return;
    if (e.target.closest('button, input, select, textarea, a')) return;
    if (e.detail === 2) {
      tauriWin
        .toggleMaximize()
        .then(updateMaximizeIcon)
        .catch(() => {});
    } else {
      tauriWin.startDragging();
    }
  }
</script>

<div class="titlebar" role="presentation" onmousedown={onTitlebarMouseDown}>
  <div class="titlebar__left">
    <button
      type="button"
      class="titlebar__iconbtn titlebar__sidebar-toggle"
      class:is-collapsed={sidebarCollapsed}
      aria-label={sidebarCollapsed ? t('titlebar.expandSidebar') : t('titlebar.collapseSidebar')}
      data-tooltip={sidebarCollapsed ? t('titlebar.expandSidebar') : t('titlebar.collapseSidebar')}
      onclick={ontogglesidebar}
    >
      <Icon name="sidebar" size="lg" />
    </button>
    {#if mode === 'detail'}
      <button type="button" class="titlebar__back" onclick={onback}>
        <Icon name="caret-left" />
        <span>{t('titlebar.back')}</span>
      </button>
    {:else}
      <Menu
        label={t('mylist.add')}
        align="start"
        bind:open={addMenuOpen}
        items={getAddGameMenuItems()}
      >
        {#snippet trigger(props)}
          <button
            {...props}
            type="button"
            class="titlebar__iconbtn"
            aria-label={t('mylist.add')}
            data-tooltip={t('mylist.add')}
          >
            <Icon name="plus" size="lg" />
          </button>
        {/snippet}
      </Menu>
    {/if}
    <SearchBar {onsettings} />
  </div>

  <div class="titlebar__brandblock">
    <span class="titlebar__mark">
      <svg viewBox="18 18 547 307"><use href="#fishvn-fish"></use></svg>
    </span>
    <span class="titlebar__brand"
      ><span class="titlebar__brand-fish">Fish</span><span class="titlebar__brand-vn">VN</span></span
    >
  </div>

  <div class="titlebar__controls">
    <button
      type="button"
      class="titlebar__btn"
      aria-label={t('titlebar.minimize')}
      onclick={() => tauriWin?.minimize()}
    >
      <svg viewBox="0 0 14 14"><line x1="3" y1="7" x2="11" y2="7" stroke="currentColor" stroke-width="1.5" /></svg>
    </button>
    <button
      type="button"
      class="titlebar__btn"
      aria-label={maximized ? t('titlebar.restore') : t('titlebar.maximize')}
      onclick={() =>
        tauriWin
          ?.toggleMaximize()
          .then(updateMaximizeIcon)
          .catch(() => {})}
    >
      {#if maximized}
        <svg viewBox="0 0 14 14"><rect x="4.5" y="5.5" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.3" /><path d="M2 2.5h7.5v1H3v6.5H2V2.5z" fill="currentColor" /></svg>
      {:else}
        <svg viewBox="0 0 14 14"><rect x="3" y="3" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.3" /></svg>
      {/if}
    </button>
    <button
      type="button"
      class="titlebar__btn titlebar__btn--close"
      aria-label={t('common.close')}
      onclick={() => tauriWin?.close()}
    >
      <svg viewBox="0 0 14 14"><line x1="4" y1="4" x2="10" y2="10" stroke="currentColor" stroke-width="1.5" /><line x1="10" y1="4" x2="4" y2="10" stroke="currentColor" stroke-width="1.5" /></svg>
    </button>
  </div>
</div>
