<script>
  import './lib/tooltip.js';
  import { t } from './lib/i18n.js';
  import Sidebar from './components/Sidebar.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import Toaster from './components/Toaster.svelte';
  import MyList from './views/MyList.svelte';

  let settingsOpen = $state(false);

  const NAV = [
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
    {
      id: 'stats',
      label: t('nav.stats'),
      icon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M22 12h-4l-3 9L9 3l-3 9H2"></path></svg>',
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

  let current = $state('mylist');
</script>

<div class="shell">
  <Sidebar items={NAV} bottom={BOTTOM} bind:current />

  <main class="shell__main" data-overlayscrollbars-initialize>
    {#if current === 'mylist'}
      <MyList />
    {:else}
      <section class="view">
        <header class="view-header">
          <h1>{NAV.find((n) => n.id === current)?.label}</h1>
        </header>
      </section>
    {/if}
  </main>
</div>

<SettingsDialog bind:open={settingsOpen} />
<Toaster />
