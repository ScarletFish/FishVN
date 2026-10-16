<script>
  import { t } from '../lib/i18n.js';
  import Menu from './Menu.svelte';
  import { coverSrc } from '../lib/cover-src.js';

  let { item, onopen, onedit } = $props();

  let coverBroken = $state(false);
  let coverUrl = $state(null);

  $effect(() => {
    const path = item.cover_path;
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

  const menuItems = $derived([
    { label: t('common.actionEdit'), onclick: () => onedit?.(item.id) },
  ]);
</script>

<div class="cover-card">
  <div class="cover-card__frame">
    <div class="cover-card__media">
      {#if coverUrl && !coverBroken}
        <img
          src={coverUrl}
          alt=""
          loading="lazy"
          decoding="async"
          onerror={() => (coverBroken = true)}
        />
      {:else}
        <div class="cover-ph">{initial}</div>
      {/if}
    </div>

    <div class="cover-card__overlay">
      <h3 class="cover-card__title">{item.name}</h3>
    </div>
  </div>

  <button
    type="button"
    class="cover-card__hit"
    aria-label={item.name}
    onclick={() => onopen?.(item.id)}
  ></button>

  <div class="cover-card__menu">
    <Menu items={menuItems} label={t('common.actionMore')} variant="plain" />
  </div>
</div>