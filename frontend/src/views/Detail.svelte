<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { joinLaunchPath } from '../lib/installations.js';
  import { bbcodeParagraphs } from '../lib/vn-merge.js';
  import { openExternal } from '../lib/open-external.js';
  import DetailCharacters from './DetailCharacters.svelte';
  import GameForm from '../components/GameForm.svelte';
  import Menu from '../components/Menu.svelte';

  let { gameId, onclose } = $props();

  let game = $state(null);
  let meta = $state(null);
  let dirs = $state([]);
  let defaultDirId = $state(null);
  let loading = $state(true);
  let failed = $state(false);
  let formOpen = $state(false);
  let menuOpen = $state(false);
  let coverUrl = $state(null);
  let coverBroken = $state(false);

  const installRows = $derived(
    dirs.map((dir) => ({
      id: dir.id,
      launch: joinLaunchPath(dir.dir_path, dir.entry_path),
      isDefault: dir.id === defaultDirId,
    })),
  );

  const TAGS_PREVIEW = 8;
  let tagsExpanded = $state(false);

  let summaryParas = $derived(bbcodeParagraphs(meta?.description));

  let alias = $derived(meta?.alias ?? '');

  function trimScore(value) {
    return String(Number(value.toFixed(2)));
  }

  let ratingScore = $derived(meta?.rating != null ? trimScore(meta.rating / 10) : '');
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
    ])
      .then(([g, m, s, d]) => {
        if (!live) return;
        game = g;
        meta = m;
        defaultDirId = s.default_installation_id;
        dirs = d;
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
      <div class="detail-banner-left">
        <div class="detail-cover">
          {#if coverUrl && !coverBroken}
            <img src={coverUrl} alt={game.name} onerror={() => (coverBroken = true)} />
          {:else}
            <div class="detail-cover__ph">
              <span class="detail-cover__initial" aria-hidden="true">{(game.name || '?')[0]}</span>
            </div>
          {/if}
        </div>
        <div class="detail-actions">
          <button type="button" class="btn btn-primary">{t('detail.actionPlay')}</button>
          <span class="detail-stat">{t('detail.playtime')}</span>
          <Menu
            bind:open={menuOpen}
            label={t('common.actionMore')}
            items={[
              { label: t('common.actionEdit'), onclick: () => (formOpen = true) },
              { label: t('detail.actionDelete'), onclick: () => {} },
            ]}
          />
        </div>
      </div>
      <div class="detail-info">
        <h1>{game.name}</h1>
        {#if alias}
          <p class="detail-alias">{alias}</p>
        {/if}
        {#if meta?.rating != null || releaseYear || developer}
          <div class="detail-info-line">
            {#if meta?.rating != null}
              <span class="info-left">
                <span class="info-rating-num">★ {ratingScore}</span>
                <span class="info-rating-sub">{t('detail.voteCount', { count: meta.votecount ?? 0 })}</span>
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
    </div>
    {#if installRows.length > 0}
      <section class="detail-install section-card">
        <div class="detail-install-header">
          <h3>{t('detail.installs')}</h3>
          <span class="detail-install-count">{installRows.length}</span>
        </div>
        <ul class="detail-install-list">
          {#each installRows as row (row.id)}
            <li class="detail-install-row">
              <span class="detail-install-path" title={row.launch}>{row.launch}</span>
              {#if row.isDefault}
                <span class="pill pill--info">{t('gameForm.install.default')}</span>
              {/if}
            </li>
          {/each}
        </ul>
      </section>
    {/if}
    <DetailCharacters gameId={gameId} />
  {/if}
</section>

<GameForm bind:open={formOpen} gameId={gameId} />
