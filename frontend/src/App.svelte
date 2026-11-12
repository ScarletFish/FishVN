<script>
  import IconContext from 'phosphor-svelte/lib/IconContext';
  import './lib/tooltip.js';
  import './lib/side-tip.js';
  import './lib/native-menu.js';
  import { onMount, tick } from 'svelte';
  import { t } from './lib/i18n.js';
  import { durationText } from './lib/playtime.js';
  import { api } from './lib/api.js';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { confirm } from './lib/confirm.svelte.js';
  import { quitGuard } from './lib/quit-guard.svelte.js';
  import { toast } from './lib/toast.svelte.js';
  import {
    notifyLibraryChanged,
    notifyLibraryItemUpdated,
    notifyLibraryTagsChanged,
    notifyLibraryTraitsChanged,
    notifyPlaySessionStarted,
    notifyPlaySessionTick,
    notifyPlaySessionEnded,
  } from './lib/library-events.js';
  import { subscribe } from './lib/events.js';
  import { report } from './lib/frontend-log.js';
  import { backfillSummaryText } from './lib/backfill-summary.js';
  import { backfillIndicator } from './lib/backfill-indicator.js';
  import { createRun, reduce as reduceRun, seedFromStatus } from './lib/backfill-status.js';
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
  import { syncReduceMotion } from './lib/motion.js';
  import { notifyConfigChanged } from './lib/config-events.js';
  import { getScrollViewport } from './lib/scrollbars.js';
  import { addGame } from './lib/add-game.svelte.js';
  import { setFormTab } from './lib/form-tab-pref.js';
  import Sidebar from './components/Sidebar.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import Titlebar from './components/chrome/Titlebar.svelte';
  import GameForm from './components/GameForm.svelte';
  import ScanAddDialog from './components/ScanAddDialog.svelte';
  import Toaster from './components/Toaster.svelte';
  import MyList from './views/MyList.svelte';
  import Backfill from './views/Backfill.svelte';
  import Query from './views/Query.svelte';
  import Dashboard from './views/Dashboard.svelte';
  import Stats from './views/Stats.svelte';
  import Detail from './views/Detail.svelte';

  let settingsOpen = $state(false);
  let dark = $state(document.documentElement.dataset.theme === 'dark');
  let run = $state(createRun());
  let seenRun = $state(0);
  let handledRun = 0;
  let current = $state('home');

  $effect(() => subscribeTheme((value) => (dark = value)));

  $effect(() => {
    syncFontScale();
    syncShowCardTitle();
    syncReduceMotion();
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

  $effect(() => {
    setFormTab(addGame.formTab);
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
    let offStart = null;
    let offTick = null;
    let offEnd = null;
    const start = subscribe('session-started', (payload) => {
      if (payload?.game_id) notifyPlaySessionStarted(payload.game_id);
    });
    const tick = subscribe('session-tick', (payload) => {
      if (payload?.game_id) notifyPlaySessionTick(payload);
    });
    const end = subscribe('session-ended', (payload) => {
      const gameId = payload?.game_id ?? null;
      const seconds = Number(payload?.seconds ?? 0);
      if (gameId) notifyPlaySessionEnded(gameId, seconds);
      notifyLibraryChanged();
      toast(t('toast.sessionEnded', { duration: durationText(t, seconds) }), 'success');
    });
    Promise.all([start, tick, end])
      .then(([a, b, c]) => {
        offStart = a;
        offTick = b;
        offEnd = c;
      })
      .catch(() => {});
    return () => {
      offStart?.();
      offTick?.();
      offEnd?.();
    };
  });

  onMount(() => {
    let alive = true;
    let unsub = null;
    (async () => {
      try {
        const s = await api.backfillStatus();
        if (alive) run = seedFromStatus(run, s);
      } catch {}
      try {
        const off = await subscribe('vndb-backfill', onBackfillEvent);
        if (alive) unsub = off;
        else off();
      } catch (e) {
        report('error', `任务事件订阅失败：${e?.message ?? e}`);
      }
    })();
    return () => {
      alive = false;
      unsub?.();
    };
  });

  function onBackfillEvent(ev) {
    if (!ev?.phase) return;
    run = reduceRun(run, ev);
    if (ev.phase === 'backfill' && ev.game_id) notifyLibraryItemUpdated(ev.game_id);
    if (ev.phase === 'finished') finishBackfill(ev);
  }

  async function finishBackfill(ev) {
    try {
      const s = await api.backfillStatus();
      if (!s?.finished || s.run_id !== ev.run_id || run.runId !== ev.run_id) return;
      run = { ...run, running: false, paused: false, stage: '', finished: s.finished };
      if (handledRun === s.run_id) return;
      handledRun = s.run_id;
      notifyLibraryChanged();
      notifyLibraryTagsChanged();
      notifyLibraryTraitsChanged();
      if (current === 'backfill') return;
      const cancelled = s.finished.cancelled;
      toast(
        `${t(cancelled ? 'mylist.backfillCancelledToast' : 'mylist.backfillDoneToast')} · ${backfillSummaryText(s.finished)}`,
        cancelled ? 'info' : 'success',
      );
    } catch {}
  }

  const indicator = $derived(backfillIndicator(run, { current, seenRun }));

  $effect(() => {
    if (current === 'backfill' && run.finished) seenRun = run.runId;
  });

  const NAV = $derived([
    {
      id: 'home',
      label: t('nav.home'),
      icon: 'house',
    },
    {
      id: 'mylist',
      label: t('nav.mylist'),
      icon: 'book',
    },
    {
      id: 'query',
      label: t('nav.query'),
      icon: 'funnel',
    },
    {
      id: 'backfill',
      label: t('nav.backfill'),
      icon: 'download',
      dot: indicator.dot,
      tip: indicator.tip,
    },
    {
      id: 'stats',
      label: t('nav.stats'),
      icon: 'chart-bar',
    },
  ]);

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
      icon: dark ? 'sun' : 'moon',
      onclick: toggleTheme,
    },
    {
      id: 'settings',
      label: t('nav.settings'),
      icon: 'gear',
      onclick: () => (settingsOpen = true),
    },
  ]);

  let detailId = $state(null);
  let autoLaunch = $state(false);
  let focusCharacter = $state(null);
  let detailNeighbors = $state([]);
  let detailMode = $state('enter');
  let detailTitle = $state('');
  let listScroll = $state(0);
  let listFilter = $state('all');
  let dashScroll = $state(0);
  let queryScroll = $state(0);
  let backfillScroll = $state(0);

  function saveViewScroll() {
    if (detailId !== null) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    if (current === 'mylist') listScroll = viewport.scrollTop;
    else if (current === 'home') dashScroll = viewport.scrollTop;
    else if (current === 'query') queryScroll = viewport.scrollTop;
    else if (current === 'backfill') backfillScroll = viewport.scrollTop;
  }

  function openDetail(id, opts = {}) {
    saveViewScroll();
    detailId = id;
    autoLaunch = opts.autostart ?? false;
    focusCharacter = opts.characterId ?? null;
    detailNeighbors = opts.neighbors ?? [];
    detailMode = 'enter';
  }

  function navigateDetail(delta) {
    if (detailNeighbors.length < 2) return;
    const idx = detailNeighbors.indexOf(detailId);
    if (idx < 0) return;
    const len = detailNeighbors.length;
    const next = (idx + delta + len) % len;
    autoLaunch = false;
    focusCharacter = null;
    detailMode = 'switch';
    detailId = detailNeighbors[next];
  }

  function closeDetail() {
    detailId = null;
    autoLaunch = false;
    focusCharacter = null;
    detailNeighbors = [];
  }

  function leaveView() {
    saveViewScroll();
    closeDetail();
  }

  $effect(() => {
    function onOpenGame(e) {
      openDetail(e.detail);
      current = 'mylist';
    }
    document.addEventListener('search-open-game', onOpenGame);
    return () => document.removeEventListener('search-open-game', onOpenGame);
  });
</script>

<IconContext values={{ weight: 'regular', color: 'currentColor' }}>
<div class="shell">
  <Titlebar
    mode={detailId !== null ? 'detail' : 'default'}
    title={detailTitle}
    onback={closeDetail}
    onsettings={() => (settingsOpen = true)}
  />

  <div class="shell__body">
    <Sidebar items={NAV} bottom={BOTTOM} bind:current onnavigate={leaveView} />

    <main class="shell__main" data-overlayscrollbars-initialize>
      {#if detailId !== null}
        {#key detailId}
          <Detail
            gameId={detailId}
            autostart={autoLaunch}
            focusCharacter={focusCharacter}
            neighbors={detailNeighbors}
            mode={detailMode}
            onnavigate={navigateDetail}
            onname={(name) => (detailTitle = name ?? '')}
            onclose={closeDetail}
          />
        {/key}
      {:else if current === 'home'}
        <Dashboard onopen={openDetail} scroll={dashScroll} />
      {:else if current === 'mylist'}
        <MyList onopen={openDetail} scroll={listScroll} bind:filter={listFilter} />
      {:else if current === 'query'}
        <Query onopen={openDetail} scroll={queryScroll} />
      {:else if current === 'backfill'}
        <Backfill scroll={backfillScroll} />
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

<SettingsDialog bind:open={settingsOpen} onbackfill={() => (current = 'backfill')} />
<GameForm
  bind:open={addGame.formOpen}
  bind:gameId={addGame.formGameId}
  initialTab={addGame.formTab}
  bind:tab={addGame.formTab}
  prefill={addGame.prefill}
/>
<ScanAddDialog bind:open={addGame.scanOpen} path={addGame.scanPath} ondone={notifyLibraryChanged} />
<Toaster />
</IconContext>
