<script>
  import './lib/tooltip.js';
  import './lib/side-tip.js';
  import './lib/native-menu.js';
  import { tick } from 'svelte';
  import { t } from './lib/i18n.js';
  import { durationText } from './lib/playtime.js';
  import { api } from './lib/api.js';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from './lib/confirm.svelte.js';
  import { quitGuard } from './lib/quit-guard.svelte.js';
  import { toast } from './lib/toast.svelte.js';
  import { notifyLibraryChanged } from './lib/library-events.js';
  import { subscribe } from './lib/events.js';
  import { loadTagDict } from './lib/tag-label.svelte.js';
  import { loadConfig } from './lib/config-store.svelte.js';
  import {
    syncTheme,
    syncAccent,
    syncFontScale,
    setTheme,
    subscribeTheme,
    viewTransition,
  } from './lib/theme.js';
  import { syncShowCardTitle } from './lib/card-prefs.js';
  import { notifyConfigChanged } from './lib/config-events.js';
  import { getScrollViewport } from './lib/scrollbars.js';
  import Sidebar from './components/Sidebar.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import Titlebar from './components/chrome/Titlebar.svelte';
  import Toaster from './components/Toaster.svelte';
  import MyList from './views/MyList.svelte';
  import Query from './views/Query.svelte';
  import Dashboard from './views/Dashboard.svelte';
  import Stats from './views/Stats.svelte';
  import Detail from './views/Detail.svelte';
  import TaskDock from './components/TaskDock.svelte';

  let settingsOpen = $state(false);
  let dockVisible = $state(false);
  let dark = $state(document.documentElement.dataset.theme === 'dark');

  $effect(() => subscribeTheme((value) => (dark = value)));

  $effect(() => {
    syncFontScale();
    syncShowCardTitle();
    loadTagDict();
    loadConfig();
    api
      .getConfig()
      .then((config) => {
        syncTheme(config.theme);
        syncAccent(config.accent);
      })
      .catch(() => {});
  });

  async function onAppQuitRequested() {
    if (quitGuard.dirty) {
      let ok = false;
      try {
        ok = await confirm('discardChanges');
      } catch {
        return;
      }
      if (!ok) return;
    }
    await invoke('resolve_app_quit');
  }

  $effect(() => {
    let unlisten = null;
    listen('app-quit-requested', () => {
      onAppQuitRequested();
    })
      .then((fn) => (unlisten = fn))
      .catch(() => {});
    return () => unlisten?.();
  });

  $effect(() => {
    let unlisten = null;
    subscribe('session-ended', (payload) => {
      notifyLibraryChanged();
      const seconds = Number(payload?.seconds ?? 0);
      toast(t('toast.sessionEnded', { duration: durationText(t, seconds) }), 'success');
    })
      .then((fn) => (unlisten = fn))
      .catch(() => {});
    return () => unlisten?.();
  });

  const NAV = [
    {
      id: 'home',
      label: t('nav.home'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>',
    },
    {
      id: 'mylist',
      label: t('nav.mylist'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"></path></svg>',
    },
    {
      id: 'query',
      label: t('nav.query'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon></svg>',
    },
    {
      id: 'stats',
      label: t('nav.stats'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><line x1="18" y1="20" x2="18" y2="10"></line><line x1="12" y1="20" x2="12" y2="4"></line><line x1="6" y1="20" x2="6" y2="14"></line></svg>',
    },
  ];

  const ICON_SUN =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>';
  const ICON_MOON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>';

  function toggleTheme() {
    const next = dark ? 'light' : 'dark';
    viewTransition(async () => {
      setTheme(next);
      await tick();
    });
    api
      .updateConfig({ theme: next })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  const BOTTOM = $derived([
    {
      id: 'theme',
      label: dark ? t('nav.themeToLight') : t('nav.themeToDark'),
      icon: dark ? ICON_SUN : ICON_MOON,
      onclick: toggleTheme,
    },
    {
      id: 'settings',
      label: t('nav.settings'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="3"></circle><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path></svg>',
      onclick: () => (settingsOpen = true),
    },
  ]);

  let current = $state('home');
  let detailId = $state(null);
  let autoLaunch = $state(false);
  let detailTitle = $state('');
  let listScroll = $state(0);
  let listFilter = $state('all');
  let dashScroll = $state(0);
  let queryScroll = $state(0);

  function saveViewScroll() {
    if (detailId !== null) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    if (current === 'mylist') listScroll = viewport.scrollTop;
    else if (current === 'home') dashScroll = viewport.scrollTop;
    else if (current === 'query') queryScroll = viewport.scrollTop;
  }

  function openDetail(id, autostart = false) {
    saveViewScroll();
    detailId = id;
    autoLaunch = autostart;
  }

  function closeDetail() {
    detailId = null;
    autoLaunch = false;
  }

  function leaveView() {
    saveViewScroll();
    closeDetail();
  }

  $effect(() => {
    function onOpenGame(e) {
      saveViewScroll();
      current = 'mylist';
      detailId = e.detail;
    }
    document.addEventListener('search-open-game', onOpenGame);
    return () => document.removeEventListener('search-open-game', onOpenGame);
  });
</script>

<div class="shell">
  <Titlebar
    mode={detailId !== null ? 'detail' : 'default'}
    title={detailTitle}
    onback={closeDetail}
    onsettings={() => (settingsOpen = true)}
  />

  <div class="shell__body">
    <Sidebar items={NAV} bottom={BOTTOM} bind:current onnavigate={leaveView} />

    <main class="shell__main" class:has-dock={dockVisible && !detailId} data-overlayscrollbars-initialize>
      {#if detailId !== null}
        <Detail
          gameId={detailId}
          autostart={autoLaunch}
          onname={(name) => (detailTitle = name ?? '')}
          onclose={closeDetail}
        />
      {:else if current === 'home'}
        <Dashboard onopen={openDetail} scroll={dashScroll} />
      {:else if current === 'mylist'}
        <MyList
          onopen={(id) => openDetail(id)}
          scroll={listScroll}
          bind:filter={listFilter}
        />
      {:else if current === 'query'}
        <Query onopen={(id) => openDetail(id)} scroll={queryScroll} />
      {:else if current === 'stats'}
        <Stats />
      {:else}
        <section class="view">
          <header class="view-header">
            <h1>{NAV.find((n) => n.id === current)?.label}</h1>
          </header>
        </section>
      {/if}
    </main>
  </div>
</div>

<TaskDock bind:visible={dockVisible} hidden={detailId !== null} />
<SettingsDialog bind:open={settingsOpen} />
<Toaster />
