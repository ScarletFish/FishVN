<script>
  import { t } from '../lib/i18n.js';
  import Menu from './Menu.svelte';
  import { coverSrc } from '../lib/cover-src.js';
  import { formatScoreShort } from '../lib/format.js';

  let { item, onopen, onedit, ondelet } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);
  let lastCoverPath = null;

  $effect(() => {
    const path = item.cover_path;
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
      .catch(() => {
        if (live) coverBroken = true;
      });
    return () => {
      live = false;
    };
  });

  const initial = $derived((item.name || '?')[0].toUpperCase());

  const mineScore = $derived(
    item.user_rating != null && item.user_rating > 0 ? item.user_rating : null,
  );
  const vndbScore = $derived(
    item.vndb_rating != null && item.vndb_rating > 0 ? item.vndb_rating : null,
  );

  const menuItems = $derived([
    ...(onedit
      ? [{ label: t('common.actionEdit'), onclick: () => onedit?.(item.id) }]
      : []),
    ...(ondelet
      ? [{ label: t('detail.actionDelete'), onclick: () => ondelet?.(item) }]
      : []),
  ]);
</script>

<div class="cover-card" data-game-id={item.id}>
  <div class="cover-card__frame">
    <div class="cover-card__media">
      {#if coverUrl && !coverBroken}
        <img
          src={coverUrl}
          alt=""
          loading="lazy"
          decoding="async"
          style:object-position="{item.cover_focal_x ?? 50}% {item.cover_focal_y ?? 50}%"
          onerror={() => (coverBroken = true)}
        />
      {:else}
        <div class="cover-ph">{initial}</div>
      {/if}
    </div>

    <div class="cover-card__overlay">
      {#if vndbScore != null}
        <span
          class="cover-card__score cover-card__score--vndb"
          role="img"
          aria-label={t('common.scoreVndb', { score: formatScoreShort(vndbScore) })}
        >
          <span class="cover-card__score-star" aria-hidden="true">☆</span>{formatScoreShort(vndbScore)}
        </span>
      {/if}
      <h3 class="cover-card__title">{item.name}</h3>
    </div>
  </div>

  {#if mineScore != null}
    <span
      class="cover-card__score cover-card__score--mine"
      role="img"
      aria-label={t('common.scoreMine', { score: formatScoreShort(mineScore) })}
    >
      <span class="cover-card__score-star" aria-hidden="true">★</span>{formatScoreShort(mineScore)}
    </span>
  {/if}

  <button
    type="button"
    class="cover-card__hit"
    aria-label={item.name}
    onclick={() => onopen?.(item.id)}
  ></button>

  {#if menuItems.length}
    <div class="cover-card__menu">
      <Menu items={menuItems} label={t('common.actionMore')} variant="plain" />
    </div>
  {/if}
</div>