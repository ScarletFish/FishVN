<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import SearchBar from './SearchBar.svelte';
  import Menu from '../Menu.svelte';
  import { openForm, addFromFile, addFromScan } from '../../lib/add-game.svelte.js';
  import { ADD_MENU_SUMMON } from '../../lib/library-events.js';
  import { t } from '../../lib/i18n.js';

  let { mode = 'default', title = '', onback, onsettings } = $props();

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
    {#if mode === 'detail'}
      <button type="button" class="titlebar__back" onclick={onback}>
        <svg viewBox="0 0 16 16">
          <polyline
            points="10 4 6 8 10 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>{t('titlebar.back')}</span>
      </button>
      <span class="titlebar__title">{title}</span>
    {:else}
      <Menu
        label={t('mylist.add')}
        align="start"
        bind:open={addMenuOpen}
        items={[
          { label: t('mylist.addManual'), onclick: () => openForm(null) },
          { label: t('mylist.addFromFile'), onclick: addFromFile },
          { label: t('mylist.addScan'), onclick: addFromScan },
        ]}
      >
        {#snippet trigger(props)}
          <button
            {...props}
            type="button"
            class="titlebar__brandbtn"
            aria-label={t('mylist.add')}
            data-tooltip={t('mylist.add')}
          >
            <span class="titlebar__mark">
              <svg viewBox="18 18 547 307"><use href="#fishvn-fish"></use></svg>
            </span>
            <span class="titlebar__brand"
              ><span class="titlebar__brand-fish">Fish</span><span class="titlebar__brand-vn">VN</span></span
            >
          </button>
        {/snippet}
      </Menu>
    {/if}
  </div>

  <SearchBar {onsettings} />

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
