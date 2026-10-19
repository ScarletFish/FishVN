<script>
  import './lib/tooltip.js';
  import { t } from './lib/i18n.js';
  import { api } from './lib/api.js';
  import { syncTheme, syncAccent } from './lib/theme.js';
  import Sidebar from './components/Sidebar.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import Titlebar from './components/chrome/Titlebar.svelte';
  import Toaster from './components/Toaster.svelte';
  import MyList from './views/MyList.svelte';
  import Dashboard from './views/Dashboard.svelte';
  import Detail from './views/Detail.svelte';

  let settingsOpen = $state(false);

  $effect(() => {
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
      id: 'scan',
      label: t('nav.scan'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>',
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
  let detailTitle = $state('');
  let listScroll = $state(0);

  $effect(() => {
    current;
    if (detailId !== null && current !== 'mylist') detailId = null;
  });

  $effect(() => {
    function onOpenGame(e) {
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
    onback={() => (detailId = null)}
    onsettings={() => (settingsOpen = true)}
  />

  <div class="shell__body">
    <Sidebar items={NAV} bottom={BOTTOM} bind:current />

    <main class="shell__main" data-overlayscrollbars-initialize>
      {#if detailId !== null}
        <Detail
          gameId={detailId}
          onname={(name) => (detailTitle = name ?? '')}
          onclose={() => (detailId = null)}
        />
      {:else if current === 'home'}
        <Dashboard onopen={(id) => (detailId = id)} />
      {:else if current === 'mylist'}
        <MyList
          onopen={(id) => (detailId = id)}
          scroll={listScroll}
          onscroll={(top) => (listScroll = top)}
        />
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

<SettingsDialog bind:open={settingsOpen} />
<Toaster />
