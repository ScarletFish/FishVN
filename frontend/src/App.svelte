<script>
  import { onMount, onDestroy } from 'svelte';
  import './lib/tooltip.js';
  import { t } from './lib/i18n.js';
  import { api } from './lib/api.js';
  import { toast } from './lib/toast.svelte.js';
  import { loadTagDict } from './lib/tag-label.svelte.js';
  import { loadConfig } from './lib/config-store.svelte.js';
  import { syncTheme, syncAccent, syncFontScale } from './lib/theme.js';
  import { subscribe } from './lib/events.js';
  import { backfillSummaryText } from './lib/backfill-summary.js';
  import Sidebar from './components/Sidebar.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import Titlebar from './components/chrome/Titlebar.svelte';
  import Toaster from './components/Toaster.svelte';
  import MyList from './views/MyList.svelte';
  import Dashboard from './views/Dashboard.svelte';
  import Stats from './views/Stats.svelte';
  import Detail from './views/Detail.svelte';
  import TaskDock from './components/TaskDock.svelte';

  let settingsOpen = $state(false);
  let dockVisible = $state(false);
  let dockUnsub = null;

  $effect(() => {
    syncFontScale();
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
      id: 'stats',
      label: t('nav.stats'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><line x1="18" y1="20" x2="18" y2="10"></line><line x1="12" y1="20" x2="12" y2="4"></line><line x1="6" y1="20" x2="6" y2="14"></line></svg>',
    },
  ];

  const BOTTOM = [
    {
      id: 'settings',
      label: t('nav.settings'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><circle cx="12" cy="12" r="3"></circle><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path></svg>',
      onclick: () => (settingsOpen = true),
    },
  ];

  let current = $state('home');
  let detailId = $state(null);
  let autoLaunch = $state(false);
  let detailTitle = $state('');
  let listScroll = $state(0);
  let listFilter = $state('all');

  function openDetail(id, autostart = false) {
    detailId = id;
    autoLaunch = autostart;
  }

  function closeDetail() {
    detailId = null;
    autoLaunch = false;
  }

  $effect(() => {
    function onOpenGame(e) {
      current = 'mylist';
      detailId = e.detail;
    }
    document.addEventListener('search-open-game', onOpenGame);
    return () => document.removeEventListener('search-open-game', onOpenGame);
  });

  onMount(async () => {
    dockUnsub = await subscribe('vndb-backfill', async (ev) => {
      if (ev?.phase !== 'finished') return;
      if (detailId === null) return;
      try {
        const s = await api.backfillStatus();
        if (s?.finished) toast(backfillSummaryText(s.finished), 'success');
      } catch {}
    });
  });

  onDestroy(() => dockUnsub?.());
</script>

<div class="shell">
  <Titlebar
    mode={detailId !== null ? 'detail' : 'default'}
    title={detailTitle}
    onback={closeDetail}
    onsettings={() => (settingsOpen = true)}
  />

  <div class="shell__body">
    <Sidebar items={NAV} bottom={BOTTOM} bind:current onnavigate={closeDetail} />

    <main class="shell__main" class:has-dock={dockVisible && !detailId} data-overlayscrollbars-initialize>
      {#if detailId !== null}
        <Detail
          gameId={detailId}
          autostart={autoLaunch}
          onname={(name) => (detailTitle = name ?? '')}
          onclose={closeDetail}
        />
      {:else if current === 'home'}
        <Dashboard onopen={openDetail} />
      {:else if current === 'mylist'}
        <MyList
          onopen={(id) => openDetail(id)}
          scroll={listScroll}
          onscroll={(top) => (listScroll = top)}
          bind:filter={listFilter}
        />
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
