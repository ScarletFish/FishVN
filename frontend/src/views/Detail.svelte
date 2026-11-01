<script>
  import { tick, onMount } from 'svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { joinLaunchPath } from '../lib/installations.js';
  import { bbcodeParagraphs } from '../lib/vn-merge.js';
  import { openExternal } from '../lib/open-external.js';
  import { toast } from '../lib/toast.svelte.js';
  import { confirm } from '../lib/confirm.svelte.js';
  import { compactDuration, durationText } from '../lib/playtime.js';
  import { formatScore } from '../lib/format.js';
  import { playHero, discardHero } from '../lib/hero-transition.js';
  import { tagLabel } from '../lib/tag-label.svelte.js';
  import { requestSearchQuery } from '../lib/search-query.js';
  import { getHideR18Tags, getSpoilerLevel, getExcludedTags } from '../lib/config-store.svelte.js';
  import { classifyTags } from '../lib/tag-order.svelte.js';
  import { LIBRARY_CHANGED, notifyLibraryTagsChanged, onLibraryItemUpdated } from '../lib/library-events.js';
  import DetailCharacters from './DetailCharacters.svelte';
  import DetailStaff from './DetailStaff.svelte';
  import DetailPlayLog from './DetailPlayLog.svelte';
  import DetailNotes from './DetailNotes.svelte';
  import GameForm from '../components/GameForm.svelte';
  import Menu from '../components/Menu.svelte';

  let { gameId, onname, onclose, autostart = false } = $props();

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
  let playLogEl = $state(null);

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

  async function refreshMeta() {
    try {
      const [g, m] = await Promise.all([api.getGame(gameId), api.getGameMetadata(gameId)]);
      if (!g) return;
      game = g;
      meta = m;
      onname?.(g.name ?? '');
    } catch {}
  }

  onMount(() => {
    const onItem = (id) => {
      if (loading || failed) return;
      if (id !== gameId) return;
      refreshMeta();
    };
    const onLibrary = () => {
      if (loading || failed) return;
      refreshMeta();
    };
    const unwatch = onLibraryItemUpdated(onItem);
    window.addEventListener(LIBRARY_CHANGED, onLibrary);
    return () => {
      unwatch();
      window.removeEventListener(LIBRARY_CHANGED, onLibrary);
    };
  });

  async function removeGame() {
    menuOpen = false;
    try {
      const ok = await confirm('deleteGame', { name: game?.name ?? '' });
      if (!ok) return;
      await api.deleteGame(gameId);
      notifyLibraryTagsChanged();
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

  let autostartDone = $state(false);

  $effect(() => {
    if (!autostart || autostartDone) return;
    if (loading || failed || !game || !canLaunch || running || launching) return;
    autostartDone = true;
    launch();
  });

  const TAGS_PREVIEW = 8;
  let tagsExpanded = $state(false);
  let spoilersRevealed = $state(false);

  let summaryParas = $derived(bbcodeParagraphs(meta?.description));

  let alias = $derived(meta?.alias ?? '');

  let ratingMineChosen = $state(true);

  let mineRating = $derived(
    gameState?.user_rating != null && gameState.user_rating > 0 ? gameState.user_rating : null,
  );
  let vndbRating = $derived(meta?.rating ?? null);
  let ratingShown = $derived(mineRating != null || vndbRating != null);
  let ratingSwitchable = $derived(mineRating != null && vndbRating != null);
  let ratingIsMine = $derived(ratingSwitchable ? ratingMineChosen : mineRating != null);
  let ratingValue = $derived(ratingIsMine ? mineRating : vndbRating);
  let ratingScore = $derived(ratingValue != null ? formatScore(ratingValue) : '');
  let ratingSub = $derived(
    ratingIsMine ? t('detail.ratingMine') : t('detail.voteCount', { count: meta?.votecount ?? 0 }),
  );
  let ratingTip = $derived(
    ratingSwitchable
      ? t(ratingIsMine ? 'detail.ratingSwitchToVndb' : 'detail.ratingSwitchToMine')
      : undefined,
  );

  let lengthEl = $state(null);

  async function toggleRating() {
    if (!lengthEl) {
      ratingMineChosen = !ratingMineChosen;
      return;
    }
    const before = lengthEl.getBoundingClientRect().left;
    lengthEl.style.transition = 'none';
    lengthEl.style.transform = '';
    ratingMineChosen = !ratingMineChosen;
    await tick();
    const shift = before - lengthEl.getBoundingClientRect().left;
    if (shift) lengthEl.style.transform = `translateX(${shift}px)`;
    void lengthEl.offsetWidth;
    lengthEl.style.transition = '';
    lengthEl.style.transform = '';
  }

  let tagsEl = $state(null);
  let heightAnim = null;
  let heightAnimEl = null;
  let heightAnimDone = null;

  function finishHeightAnim() {
    if (heightAnimEl) {
      heightAnimEl.style.height = '';
      heightAnimEl.style.overflow = '';
    }
    const done = heightAnimDone;
    heightAnim = null;
    heightAnimEl = null;
    heightAnimDone = null;
    done?.();
  }

  function stopHeightAnim() {
    if (!heightAnim) return;
    heightAnim.cancel();
    finishHeightAnim();
  }

  function runHeightAnim(el, from, to, done) {
    const style = getComputedStyle(el);
    const duration = parseFloat(style.getPropertyValue('--duration-normal')) || 240;
    const easing = style.getPropertyValue('--ease-out').trim() || 'ease-out';
    el.style.overflow = 'hidden';
    el.style.height = `${from}px`;
    void el.offsetHeight;
    const anim = el.animate([{ height: `${from}px` }, { height: `${to}px` }], {
      duration,
      easing,
    });
    heightAnim = anim;
    heightAnimEl = el;
    heightAnimDone = done ?? null;
    anim.finished
      .then(() => {
        if (heightAnim !== anim) return;
        finishHeightAnim();
      })
      .catch(() => {});
  }

  async function animateHeight(el, mutate) {
    if (!el) {
      mutate();
      return;
    }
    stopHeightAnim();
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
      mutate();
      return;
    }
    const before = el.offsetHeight;
    mutate();
    await tick();
    const after = el.offsetHeight;
    if (before === after) return;
    runHeightAnim(el, before, after);
  }

  function toggleTags() {
    void animateHeight(tagsEl, () => {
      tagsExpanded = !tagsExpanded;
    });
  }

  function toggleSpoilers() {
    void animateHeight(tagsEl, () => {
      spoilersRevealed = !spoilersRevealed;
    });
  }

  let summaryEl = $state(null);
  let summaryOpen = $state(false);
  let summaryFolding = $state(false);
  let summaryClamped = $state(false);

  function summaryOverflows() {
    const el = summaryEl;
    if (!el) return false;
    el.classList.add('detail-summary--open');
    const full = el.offsetHeight;
    el.classList.remove('detail-summary--open');
    const folded = el.offsetHeight;
    return full > folded + 1;
  }

  function toggleSummary() {
    if (!summaryOpen) {
      summaryFolding = false;
      void animateHeight(summaryEl, () => {
        summaryOpen = true;
      });
      return;
    }
    void foldSummary();
  }

  async function foldSummary() {
    const el = summaryEl;
    if (!el) {
      summaryOpen = false;
      return;
    }
    stopHeightAnim();
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
      summaryOpen = false;
      return;
    }
    const full = el.offsetHeight;
    el.classList.remove('detail-summary--open');
    const folded = el.offsetHeight;
    el.classList.add('detail-summary--open');
    if (full <= folded + 1) {
      summaryOpen = false;
      return;
    }
    summaryOpen = false;
    summaryFolding = true;
    await tick();
    runHeightAnim(el, full, folded, () => {
      summaryFolding = false;
    });
  }

  $effect(() => {
    const el = summaryEl;
    void meta?.description;
    summaryOpen = false;
    summaryFolding = false;
    summaryClamped = false;
    if (!el) return;
    let width = -1;
    const measure = () => {
      if (summaryOpen || summaryFolding) return;
      const current = el.clientWidth;
      if (current === width) return;
      width = current;
      summaryClamped = summaryOverflows();
    };
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  });

  let lengthText = $derived(meta?.length_minutes != null ? compactDuration(meta.length_minutes) : '');
  let releaseYear = $derived(meta?.released?.slice(0, 4) ?? '');
  let developer = $derived(meta?.developers?.[0] ?? '');

  let revealSpoilers = $derived(tagsExpanded && spoilersRevealed);
  let tagView = $derived(
    classifyTags(meta?.tags, {
      hideR18: getHideR18Tags(),
      spoilerLevel: getSpoilerLevel(),
      excludedTags: getExcludedTags(),
      revealSpoilers,
    }),
  );
  let spoilerCount = $derived(tagView.spoilers.length);
  let shownTags = $derived(
    tagsExpanded ? tagView.shown : tagView.shown.slice(0, TAGS_PREVIEW),
  );
  let tagsHiddenCount = $derived(
    Math.max(tagView.shown.length - TAGS_PREVIEW, 0) + (revealSpoilers ? 0 : spoilerCount),
  );

  function searchTag(tag) {
    requestSearchQuery(tagLabel(tag));
  }

  $effect(() => {
    loading = true;
    failed = false;
    game = null;
    meta = null;
    dirs = [];
    defaultDirId = null;
    tagsExpanded = false;
    spoilersRevealed = false;
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

  let lastCoverPath = null;

  $effect(() => {
    const path = meta?.image_path;
    if (path === lastCoverPath) return;
    lastCoverPath = path;
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

{#snippet ratingFace()}
  <span class="info-rating-mark" aria-hidden="true">{ratingIsMine ? '★' : '☆'}</span>
  <span class="info-rating-num">
    {#each ratingScore.split('') as char, index (index)}
      <span class="info-rating-digit" class:info-rating-dot={char === '.'}>{char}</span>
    {/each}
  </span>
  <span class="info-rating-sub">{ratingSub}</span>
{/snippet}

<section class="detail">
  {#if loading}
    <p class="detail-note">{t('common.loading')}</p>
  {:else if failed}
    <p class="detail-note">{t('detail.error')}</p>
  {:else if !game}
    <p class="detail-note">{t('detail.notFound')}</p>
  {:else}
    <div class="detail-ambience detail-fade" class:detail-ambience--empty={!coverUrl || coverBroken} aria-hidden="true">
      {#if coverUrl && !coverBroken}
        <img src={coverUrl} alt="" onerror={() => (coverBroken = true)} />
      {/if}
    </div>
    <div class="detail-banner">
      <div class="detail-banner-left">
        <div class="detail-cover">
          {#if coverUrl && !coverBroken}
            <img
              src={coverUrl}
              alt={game.name}
              style:object-position="{meta?.cover_focal_x ?? 50}% {meta?.cover_focal_y ?? 50}%"
              onerror={() => (coverBroken = true)}
            />
          {:else}
            <div class="detail-cover__ph">
              <span class="detail-cover__initial" aria-hidden="true">{(game.name || '?')[0]}</span>
            </div>
          {/if}
        </div>
        <div class="detail-actions detail-rise">
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
            {#if playSeconds > 0}
              <button
                type="button"
                class="detail-stat detail-stat--link"
                data-tooltip={t('detail.playtimeJump')}
                onclick={() => playLogEl?.scrollTo()}
              >
                {durationText(t, playSeconds)}
              </button>
            {:else}
              <span class="detail-stat" data-tooltip={t('detail.playtime')}>--</span>
            {/if}
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
      </div>
      <div class="detail-info">
        <h1 class="detail-rise" style="animation-delay: 0.04s">{game.name}</h1>
        {#if alias}
          <p class="detail-alias detail-rise" style="animation-delay: 0.09s">{alias}</p>
        {/if}
        {#if ratingShown || lengthText || releaseYear || developer}
          <div class="detail-info-line detail-rise" style="animation-delay: 0.14s">
            {#if ratingShown || lengthText}
              <span class="info-left">
                {#if ratingShown}
                  {#if ratingSwitchable}
                    <button
                      type="button"
                      class="info-rating"
                      class:info-rating--mine={ratingIsMine}
                      class:info-rating--vndb={!ratingIsMine}
                      aria-label={`${ratingScore} ${ratingSub}`}
                      data-tooltip={ratingTip}
                      onclick={toggleRating}
                    >
                      {@render ratingFace()}
                    </button>
                  {:else}
                    <span
                      class="info-rating"
                      class:info-rating--mine={ratingIsMine}
                      class:info-rating--vndb={!ratingIsMine}
                    >
                      {@render ratingFace()}
                    </span>
                  {/if}
                {/if}
                {#if lengthText}
                  <span class="info-length" data-tooltip={t('detail.lengthHint')} bind:this={lengthEl}>
                    <span class="info-length-mark">#</span>{lengthText}
                  </span>
                {/if}
              </span>
            {/if}
            <span class="info-tags">
              {#if releaseYear}
                <span class="pill pill--tag">{releaseYear}</span>
              {/if}
              {#if developer}
                <span class="pill pill--tag">{developer}</span>
              {/if}
            </span>
          </div>
        {/if}
        <div class="detail-tags detail-rise" style="animation-delay: 0.19s" bind:this={tagsEl}>
          <div class="detail-tags-list">
            {#each shownTags as tag (tag.id ?? tag.name)}
              <button
                type="button"
                class="pill pill--tag pill--link"
                aria-label={t('detail.tagSearch', { tag: tagLabel(tag) })}
                onclick={() => searchTag(tag)}
              >{tagLabel(tag)}</button>
            {/each}
            {#if tagsHiddenCount > 0 || tagsExpanded}
              <button
                type="button"
                class="pill pill--more"
                aria-expanded={tagsExpanded}
                onclick={toggleTags}
              >
                {tagsExpanded ? t('detail.fewerTags') : t('detail.moreTags', { count: tagsHiddenCount })}
              </button>
            {/if}
          </div>
          {#if tagsExpanded && (spoilerCount > 0 || tagView.poolCount > 0)}
            <div class="toggle-row">
              {#if spoilerCount > 0}
                <span>{t('detail.tagSpoilersCount', { count: spoilerCount })}</span>
                <label class="toggle-switch">
                  <input
                    type="checkbox"
                    checked={spoilersRevealed}
                    aria-label={t('detail.tagSpoilersToggle')}
                    onchange={toggleSpoilers}
                  />
                  <span class="toggle-slider"></span>
                </label>
              {/if}
              {#if tagView.poolCount > 0}
                <span class="detail-tags-excluded"
                  >{t('detail.tagsExcluded', { count: tagView.poolCount })}</span
                >
              {/if}
            </div>
          {/if}
          {#if revealSpoilers && spoilerCount > 0}
            <div class="detail-tags-list detail-tags-spoiler">
              {#each tagView.spoilers as tag (tag.id ?? tag.name)}
                <button
                  type="button"
                  class="pill pill--tag pill--link"
                  aria-label={t('detail.tagSearch', { tag: tagLabel(tag) })}
                  onclick={() => searchTag(tag)}
                >{tagLabel(tag)}</button>
              {/each}
            </div>
          {/if}
        </div>
        <div
          class="detail-summary detail-rise"
          class:detail-summary--open={summaryOpen || summaryFolding}
          style="animation-delay: 0.24s"
          bind:this={summaryEl}
        >
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
        {#if summaryClamped}
          <button
            type="button"
            class="detail-summary-toggle"
            aria-expanded={summaryOpen}
            onclick={toggleSummary}
          >
            {summaryOpen ? t('detail.summaryLess') : t('detail.summaryMore')}
          </button>
        {/if}
      </div>
      {#if launchPath}
        <button
          type="button"
          class="detail-install detail-rise"
          style="animation-delay: 0.29s"
          data-tooltip={t('detail.installDefaultTip')}
          onclick={() => {
            formTab = 'installations';
            formOpen = true;
          }}
        >
          <span class="detail-install-label">{t('detail.installDefault')}</span>
          <span class="detail-install-path">{launchPath}</span>
          <svg
            class="detail-install-edit"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
            <path d="m15 5 4 4" />
          </svg>
        </button>
      {/if}
    </div>
    <DetailCharacters gameId={gameId} />
    <DetailStaff {gameId} />
    <DetailPlayLog {gameId} bind:this={playLogEl} />
    <DetailNotes {gameId} />
  {/if}
</section>

<GameForm bind:open={formOpen} gameId={gameId} initialTab={formTab} />
