<script>
  import { onMount, tick } from 'svelte';
  import Icon from '../components/Icon.svelte';
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { durationText } from '../lib/playtime.js';
  import { formatDateTime } from '../lib/format.js';
  import { coverSrc, fullSrc } from '../lib/cover-src.js';
  import { captureHero } from '../lib/hero-transition.js';
  import { shouldReduceMotion } from '../lib/motion.js';
  import { buildShowcaseItems, stageDescriptor } from '../lib/showcase-items.js';
  import {
    LIBRARY_CHANGED,
    notifyAddMenuSummon,
    onLibraryItemsThrottled,
  } from '../lib/library-events.js';

  let { onopen, selectedId = $bindable(null) } = $props();

  let items = $state([]);
  let loading = $state(true);
  let error = $state(null);
  let covers = $state({});
  let layers = $state([]);
  let stripEl = $state(null);
  let layerSeq = 0;
  let requestSeq = 0;

  const current = $derived(items.find((item) => item.id === selectedId) ?? items[0] ?? null);
  const neighborIds = $derived(items.map((item) => item.id));
  const descriptor = $derived(current ? stageDescriptor(current) : null);
  const isClear = $derived(layers.length > 0 && layers[layers.length - 1].mode === 'clear');
  const lastPlayedText = $derived(formatDateTime(current?.last_played_at));
  const playSeconds = $derived(Number(current?.play_seconds) || 0);

  function resolveUrl(desc) {
    if (!desc?.path) return Promise.resolve(null);
    return desc.mode === 'clear' ? fullSrc(desc.path) : coverSrc(desc.path);
  }

  function decode(url) {
    return new Promise((resolve) => {
      if (!url) return resolve(false);
      const img = new Image();
      img.onload = () => resolve(true);
      img.onerror = () => resolve(false);
      img.src = url;
    });
  }

  function applyLayer(next) {
    const previous = layers.map((layer) => ({ ...layer, entering: false })).slice(-1);
    layers = [...previous, next];
  }

  async function centerSelected(id, animated = true) {
    if (!stripEl || !id) return;
    await tick();
    const card = stripEl.querySelector(`[data-showcase-id="${CSS.escape(String(id))}"]`);
    if (!card) return;
    const stripRect = stripEl.getBoundingClientRect();
    const cardRect = card.getBoundingClientRect();
    const delta = cardRect.left - stripRect.left - (stripRect.width - cardRect.width) / 2;
    stripEl.scrollTo({
      left: stripEl.scrollLeft + delta,
      behavior: animated && !shouldReduceMotion() ? 'smooth' : 'auto',
    });
  }

  async function load() {
    try {
      const rows = await api.showcaseItems();
      const next = buildShowcaseItems(rows);
      items = next;
      selectedId = next.some((item) => item.id === selectedId)
        ? selectedId
        : (next[0]?.id ?? null);
      error = null;
    } catch (e) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
    centerSelected(selectedId, false);
  }

  function select(item) {
    selectedId = item.id;
    centerSelected(item.id);
  }

  function onWindowKeydown(event) {
    if (event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey) return;
    const active = document.activeElement;
    if (
      active &&
      (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable)
    )
      return;
    if (document.getElementById('app')?.inert) return;
    if (!items.length) return;

    if (event.key === 'Enter') {
      if (event.target instanceof Element && event.target.closest('button, a')) return;
      if (!current?.can_launch) return;
      event.preventDefault();
      launch();
      return;
    }

    const index = items.findIndex((item) => item.id === selectedId);
    let nextIndex = null;
    if (event.key === 'ArrowLeft') nextIndex = Math.max(0, index - 1);
    else if (event.key === 'ArrowRight') nextIndex = Math.min(items.length - 1, index + 1);
    else if (event.key === 'Home') nextIndex = 0;
    else if (event.key === 'End') nextIndex = items.length - 1;
    else return;
    event.preventDefault();
    const item = items[nextIndex];
    if (!item) return;
    selectedId = item.id;
    centerSelected(item.id);
    if (stripEl?.contains(document.activeElement)) {
      stripEl
        .querySelector(`[data-showcase-id="${CSS.escape(String(item.id))}"]`)
        ?.focus({ preventScroll: true });
    }
  }

  function hero() {
    const thumb = stripEl?.querySelector('.showcase-thumb.is-selected');
    if (!thumb) return;
    captureHero(thumb, { img: 'img', frame: 'self' });
  }

  function viewDetail() {
    if (!current) return;
    hero();
    onopen(current.id, { neighbors: neighborIds });
  }

  function launch() {
    if (!current) return;
    hero();
    onopen(current.id, { autostart: Boolean(current.can_launch), neighbors: neighborIds });
  }

  $effect(() => {
    const desc = descriptor;
    const seq = ++requestSeq;
    if (!desc) {
      layers = [];
      return;
    }
    let alive = true;
    (async () => {
      const url = await resolveUrl(desc).catch(() => null);
      if (!alive || seq !== requestSeq) return;
      const ok = await decode(url);
      if (!alive || seq !== requestSeq) return;
      applyLayer({
        id: ++layerSeq,
        url: ok ? url : null,
        mode: desc.mode,
        focal: desc.focal,
        entering: !shouldReduceMotion(),
      });
    })();
    return () => {
      alive = false;
    };
  });

  $effect(() => {
    for (const item of items) {
      const path = String(item.cover_path ?? '').trim();
      if (!path || covers[item.id]) continue;
      coverSrc(path)
        .then((url) => {
          if (url) covers = { ...covers, [item.id]: url };
        })
        .catch(() => {});
    }
  });

  $effect(() => {
    const list = items;
    const index = list.findIndex((item) => item.id === selectedId);
    if (index < 0) return;
    for (const step of [-1, 1]) {
      const near = list[index + step];
      if (!near) continue;
      resolveUrl(stageDescriptor(near))
        .then((url) => {
          if (url) {
            const img = new Image();
            img.src = url;
          }
        })
        .catch(() => {});
    }
  });

  onMount(() => {
    load();
    window.addEventListener(LIBRARY_CHANGED, load);
    const unwatch = onLibraryItemsThrottled(load);
    return () => {
      window.removeEventListener(LIBRARY_CHANGED, load);
      unwatch();
    };
  });
</script>

<svelte:window onkeydown={onWindowKeydown} />

<section class="showcase" class:showcase--clear={isClear} aria-label={t('nav.showcase')}>
  <div
    class="showcase-stage"
    class:showcase-stage--empty={layers.length === 0}
  >
    <div class="showcase-stage__layers">
      {#each layers as layer (layer.id)}
        <div
          class="showcase-stage__layer"
          class:showcase-stage__layer--empty={!layer.url}
          class:is-entering={layer.entering}
          aria-hidden="true"
        >
          {#if layer.url}
            <img
              class="showcase-stage__blur"
              src={layer.url}
              alt=""
              style:object-position={layer.focal}
              onerror={(e) => (e.currentTarget.style.visibility = 'hidden')}
            />
            {#if layer.mode === 'clear'}
              <img
                class="showcase-stage__clear"
                src={layer.url}
                alt=""
                style:object-position={layer.focal}
                onerror={(e) => (e.currentTarget.style.visibility = 'hidden')}
              />
            {/if}
          {/if}
        </div>
      {/each}
    </div>
    <div class="showcase-stage__wash"></div>
    <div class="showcase-stage__scrim"></div>
  </div>

  {#if !loading && !error && items.length > 0}
    <div class="showcase-ui">
      <div class="showcase-bottom">
        <div class="showcase-meta">
          <h1 class="showcase-title" title={current?.name ?? ''}>{current?.name ?? ''}</h1>

          <div class="showcase-meta-row">
            <div class="showcase-facts">
              {#if lastPlayedText}
                <span class="showcase-fact">{t('showcase.lastPlayed')} {lastPlayedText}</span>
              {:else}
                <span class="showcase-fact">{t('showcase.noRecord')}</span>
              {/if}
              {#if playSeconds > 0}
                <span class="showcase-fact">
                  {t('showcase.totalTime')} {durationText(t, playSeconds)}
                </span>
              {/if}
            </div>

            <div class="showcase-actions">
              <button type="button" class="btn btn-ghost showcase-action" onclick={viewDetail}>
                <Icon name="info" size="lg" />
                {t('showcase.detail')}
              </button>
              <button
                type="button"
                class="btn btn-primary showcase-action"
                disabled={!current?.can_launch}
                onclick={launch}
              >
                <Icon name="play" size="lg" />
                {t('showcase.launch')}
              </button>
            </div>
          </div>
        </div>

        <div class="showcase-strip">
          <div
            class="showcase-strip__scroll"
            bind:this={stripEl}
            role="group"
            aria-label={t('showcase.stripLabel')}
          >
            {#each items as item (item.id)}
              <button
                type="button"
                class="showcase-thumb"
                class:is-selected={item.id === selectedId}
                data-showcase-id={item.id}
                aria-label={item.name}
                aria-pressed={item.id === selectedId}
                onclick={() => select(item)}
              >
                {#if covers[item.id]}
                  <img
                    src={covers[item.id]}
                    alt=""
                    style:object-position={stageDescriptor(item).coverFocal}
                  />
                {:else}
                  <span class="showcase-thumb__initial" aria-hidden="true">
                    {(item.name || '?')[0]}
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="showcase-state"><p>{t('common.loading')}</p></div>
  {:else if error}
    <div class="showcase-state"><p>{t('showcase.error')}</p></div>
  {:else if items.length === 0}
    <div class="showcase-state showcase-empty">
      <Icon name="image" size="xl" />
      <p>{t('showcase.emptyTitle')}</p>
      <p class="showcase-empty__hint">{t('showcase.emptyHint')}</p>
      <button type="button" class="btn btn-primary" onclick={notifyAddMenuSummon}>
        {t('mylist.add')}
      </button>
    </div>
  {/if}
</section>
