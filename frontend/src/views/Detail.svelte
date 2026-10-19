<script>
  import { tick } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { joinLaunchPath } from '../lib/installations.js';
  import { bbcodeParagraphs } from '../lib/vn-merge.js';
  import { openExternal } from '../lib/open-external.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { compactDuration, durationText } from '../lib/playtime.js';
  import { playHero, discardHero } from '../lib/hero-transition.js';
  import DetailCharacters from './DetailCharacters.svelte';
  import DetailStaff from './DetailStaff.svelte';
  import GameForm from '../components/GameForm.svelte';
  import Menu from '../components/Menu.svelte';

  let { gameId, onname, onclose } = $props();

  let game = $state(null);
  let meta = $state(null);
  let dirs = $state([]);
  let defaultDirId = $state(null);
  let loading = $state(true);
  let failed = $state(false);
  let formOpen = $state(false);
  let formTab = $state('basic');
  let menuOpen = $state(false);
  let coverUrl = $state(null);
  let coverBroken = $state(false);
  let gameState = $state(null);
  let activeSession = $state(null);
  let launching = $state(false);

  const launchTarget = $derived(
    dirs.find((dir) => dir.id === defaultDirId && dir.entry_path) ??
      dirs.find((dir) => dir.entry_path) ??
      dirs[0] ??
      null,
  );
  const launchPath = $derived(
    launchTarget ? joinLaunchPath(launchTarget.dir_path, launchTarget.entry_path) : '',
  );

  const POLL_MS = 5000;

  const canLaunch = $derived(dirs.some((dir) => dir.entry_path));
  const running = $derived(activeSession != null);
  const playSeconds = $derived(gameState?.play_seconds ?? 0);

  function applyState(state, active) {
    gameState = state;
    defaultDirId = state.default_installation_id;
    activeSession = active.find((item) => item.game_id === gameId) ?? null;
  }

  async function refreshPlay() {
    const [state, active] = await Promise.all([
      api.getGameState(gameId),
      api.listActiveSessions(),
    ]);
    applyState(state, active);
  }

  async function removeGame() {
    menuOpen = false;
    try {
      const ok = await confirm('deleteGame', game?.name ?? '');
      if (!ok) return;
      await api.deleteGame(gameId);
      toast(t('detail.deleted'));
      onclose?.();
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  async function launch() {
    if (!canLaunch || launching || running) return;
    launching = true;
    try {
      await api.launchGame(gameId);
      await refreshPlay();
    } catch (err) {
      toast(err.message, 'error');
      refreshPlay().catch(() => {});
    } finally {
      launching = false;
    }
  }

  const TAGS_PREVIEW = 8;
  let tagsExpanded = $state(false);

  let summaryParas = $derived(bbcodeParagraphs(meta?.description));

  let alias = $derived(meta?.alias ?? '');

  function trimScore(value) {
    return String(Number(value.toFixed(2)));
  }

  let ratingScore = $derived(meta?.rating != null ? trimScore(meta.rating / 10) : '');
  let lengthText = $derived(meta?.length_minutes != null ? compactDuration(meta.length_minutes) : '');
  let releaseYear = $derived(meta?.released?.slice(0, 4) ?? '');
  let developer = $derived(meta?.developers?.[0] ?? '');

  let tags = $derived(
    (meta?.tags ?? [])
      .filter((tag) => !tag.spoiler)
      .sort((a, b) => (b.rating ?? 0) - (a.rating ?? 0) || (b.vn_count ?? 0) - (a.vn_count ?? 0))
  );
  let shownTags = $derived(tagsExpanded ? tags : tags.slice(0, TAGS_PREVIEW));
  let tagsRemaining = $derived(tags.length - TAGS_PREVIEW);

  $effect(() => {
    loading = true;
    failed = false;
    game = null;
    meta = null;
    dirs = [];
    defaultDirId = null;
    tagsExpanded = false;
    menuOpen = false;

    let live = true;
    Promise.all([
      api.getGame(gameId),
      api.getGameMetadata(gameId),
      api.getGameState(gameId),
      api.listInstallations(gameId),
      api.listActiveSessions(),
    ])
      .then(([g, m, s, d, active]) => {
        if (!live) return;
        game = g;
        meta = m;
        dirs = d;
        onname?.(g?.name ?? '');
        applyState(s, active);
      })
      .catch(() => {
        if (live) failed = true;
      })
      .finally(() => {
        if (live) loading = false;
      });

    return () => {
      live = false;
    };
  });

  $effect(() => {
    const path = meta?.image_path;
    coverBroken = false;
    coverUrl = null;
    if (!path) return;
    let live = true;
    coverSrc(path)
      .then((url) => {
        if (live) coverUrl = url;
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  });

  $effect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      refreshPlay().catch(() => {});
    }, POLL_MS);
    return () => clearInterval(timer);
  });

  $effect(() => {
    if (loading) return;
    if (failed || !game || coverBroken) {
      discardHero();
      return;
    }
    tick().then(() => {
      playHero(document.querySelector('.detail-cover'));
    });
  });

  let formWasOpen = false;

  $effect(() => {
    if (formWasOpen && !formOpen) refreshPlay().catch(() => {});
    formWasOpen = formOpen;
  });

  function handleSideMouse(e) {
    if (e.button === 3) onclose?.();
  }
</script>

<svelte:window onmouseup={handleSideMouse} />

<section class="detail">
  {#if loading}
    <p class="detail-note">{t('common.loading')}</p>
  {:else if failed}
    <p class="detail-note">{t('detail.error')}</p>
  {:else if !game}
    <p class="detail-note">{t('detail.notFound')}</p>
  {:else}
    <div class="detail-ambience" class:detail-ambience--empty={!coverUrl || coverBroken} aria-hidden="true">
      {#if coverUrl && !coverBroken}
        <img src={coverUrl} alt="" onerror={() => (coverBroken = true)} />
      {/if}
    </div>
    <div class="detail-banner">
      <div class="detail-cover">
        {#if coverUrl && !coverBroken}
          <img src={coverUrl} alt={game.name} onerror={() => (coverBroken = true)} />
        {:else}
          <div class="detail-cover__ph">
            <span class="detail-cover__initial" aria-hidden="true">{(game.name || '?')[0]}</span>
          </div>
        {/if}
      </div>
      <div class="detail-info">
        <h1>{game.name}</h1>
        {#if alias}
          <p class="detail-alias">{alias}</p>
        {/if}
        {#if meta?.rating != null || lengthText || releaseYear || developer}
          <div class="detail-info-line">
            {#if meta?.rating != null || lengthText}
              <span class="info-left">
                {#if meta?.rating != null}
                  <span class="info-rating-num">★ {ratingScore}</span>
                  <span class="info-rating-sub">{t('detail.voteCount', { count: meta.votecount ?? 0 })}</span>
                {/if}
                {#if lengthText}
                  <span class="info-length" data-tooltip={t('detail.lengthHint')}>
                    <span class="info-length-mark">#</span>{lengthText}
                  </span>
                {/if}
              </span>
            {/if}
            <span class="info-tags">
              {#if releaseYear}
                <span class="pill pill--info">{releaseYear}</span>
              {/if}
              {#if developer}
                <span class="pill pill--info">{developer}</span>
              {/if}
            </span>
          </div>
        {/if}
        <div class="detail-tags">
          <div class="detail-tags-list">
            {#each shownTags as tag (tag.id ?? tag.name)}
              <span class="pill pill--tag">{tag.name}</span>
            {/each}
            {#if tagsRemaining > 0}
              <button
                type="button"
                class="pill pill--more"
                aria-expanded={tagsExpanded}
                onclick={() => (tagsExpanded = !tagsExpanded)}
              >
                {tagsExpanded ? t('detail.fewerTags') : t('detail.moreTags', { count: tagsRemaining })}
              </button>
            {/if}
          </div>
        </div>
        <div class="detail-summary">
          {#each summaryParas as para, i (i)}
            <p>{#each para as seg, j (j)}{#if seg.type === 'link'}<a
                href={seg.href}
                onclick={(e) => {
                  e.preventDefault();
                  openExternal(seg.href);
                }}>{seg.text}</a
              >{:else}{seg.text}{/if}{/each}</p>
          {/each}
        </div>
      </div>
      <div class="detail-actions">
        <span
          class="detail-action"
          data-tooltip={canLaunch ? null : t('detail.playNoInstall')}
        >
          <button
            type="button"
            class="btn btn-primary"
            disabled={!canLaunch || launching || running}
            onclick={launch}
          >
            {running ? t('status.playing') : t('detail.actionPlay')}
          </button>
        </span>
        <div class="detail-launch-meta">
          <span class="detail-stat" data-tooltip={t('detail.playtime')}
            >{playSeconds > 0 ? durationText(t, playSeconds) : '--'}</span
          >
          <Menu
            bind:open={menuOpen}
            label={t('common.actionMore')}
            items={[
              {
                label: t('common.actionEdit'),
                onclick: () => {
                  formTab = 'basic';
                  formOpen = true;
                },
              },
              { label: t('detail.actionDelete'), onclick: removeGame },
            ]}
          />
        </div>
      </div>
      {#if launchPath}
        <button
          type="button"
          class="detail-install"
          data-tooltip={launchPath}
          onclick={() => {
            formTab = 'installations';
            formOpen = true;
          }}
        >
          {launchPath}
        </button>
      {/if}
    </div>
    <DetailCharacters gameId={gameId} />
    <DetailStaff {gameId} />
  {/if}
</section>

<GameForm bind:open={formOpen} gameId={gameId} initialTab={formTab} />
