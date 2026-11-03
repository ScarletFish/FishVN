<script>
  import { onMount, onDestroy } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { STATUS_KEYS, statusLabel } from '../lib/status.js';
  import { captureHero } from '../lib/hero-transition.js';
  import { onLibraryItemsThrottled } from '../lib/library-events.js';
  import { dimFilled, emptyQueryUi, hasActive, uiToSpec } from '../lib/query-spec.js';
  import SelectDd from '../components/SelectDd.svelte';
  import GameCard from '../components/GameCard.svelte';
  import WindowedGrid from '../components/WindowedGrid.svelte';
  import ConditionCard from '../components/query/ConditionCard.svelte';
  import TagCondition from '../components/query/TagCondition.svelte';
  import DateRangeField from '../components/query/DateRangeField.svelte';
  import NumberStepper from '../components/query/NumberStepper.svelte';

  let { onopen } = $props();

  const ui = $state(emptyQueryUi());
  const spec = $derived(uiToSpec(ui));
  const active = $derived(hasActive(ui));

  let results = $state([]);
  let loading = $state(true);
  let error = $state(null);
  let sortBy = $state('vndb');
  let reqSeq = 0;
  let unwatch = null;

  const SORT_OPTIONS = $derived([
    { value: 'vndb', label: t('query.sortVndb') },
    { value: 'user', label: t('query.sortUser') },
    { value: 'name', label: t('query.sortName') },
    { value: 'added', label: t('query.sortAdded') },
  ]);

  const sorted = $derived.by(() => {
    const list = results.slice();
    if (sortBy === 'name') list.sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'));
    else if (sortBy === 'user') list.sort((a, b) => (b.user_rating ?? -1) - (a.user_rating ?? -1));
    else if (sortBy === 'added') list.sort((a, b) => b.created_at - a.created_at);
    else list.sort((a, b) => (b.vndb_rating ?? -1) - (a.vndb_rating ?? -1));
    return list;
  });

  $effect(() => {
    const s = spec;
    const seq = ++reqSeq;
    const timer = setTimeout(async () => {
      try {
        const rows = await api.queryGames(s);
        if (seq !== reqSeq) return;
        results = rows;
        error = null;
      } catch (e) {
        if (seq !== reqSeq) return;
        error = e.message;
      } finally {
        if (seq === reqSeq) loading = false;
      }
    }, 100);
    return () => clearTimeout(timer);
  });

  onMount(() => {
    unwatch = onLibraryItemsThrottled(reload);
  });

  onDestroy(() => unwatch?.());

  async function reload() {
    const seq = ++reqSeq;
    try {
      const rows = await api.queryGames(spec);
      if (seq !== reqSeq) return;
      results = rows;
      error = null;
    } catch (e) {
      if (seq !== reqSeq) return;
      error = e.message;
    }
  }

  function openFromCard(item) {
    const card = document.querySelector(`.cover-card[data-game-id="${CSS.escape(String(item.id))}"]`);
    if (card) captureHero(card);
    onopen(item.id);
  }

  function clearDim(dim) {
    ui[dim] = emptyQueryUi()[dim];
  }

  function clearAll() {
    Object.assign(ui, emptyQueryUi());
  }

  function toggleIn(list, value) {
    const idx = list.indexOf(value);
    if (idx >= 0) list.splice(idx, 1);
    else list.push(value);
  }

  const SIDE_MIN = 240;
  const SIDE_MAX = 460;
  const SIDE_DEFAULT = 315;
  const SIDE_STORE_KEY = 'fishvn.querySideWidth';

  let sideWidth = $state(
    (() => {
      const n = Number(localStorage.getItem(SIDE_STORE_KEY));
      return Number.isFinite(n) && n >= SIDE_MIN && n <= SIDE_MAX ? n : SIDE_DEFAULT;
    })(),
  );
  let resizing = $state(false);

  function startResize(e) {
    e.preventDefault();
    resizing = true;
    document.body.classList.add('is-col-resizing');
    const startX = e.clientX;
    const startW = sideWidth;
    const onMove = (ev) => {
      sideWidth = Math.min(SIDE_MAX, Math.max(SIDE_MIN, startW + ev.clientX - startX));
    };
    const onUp = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      document.body.classList.remove('is-col-resizing');
      resizing = false;
      localStorage.setItem(SIDE_STORE_KEY, String(sideWidth));
    };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  }

  function resetResize() {
    sideWidth = SIDE_DEFAULT;
    localStorage.setItem(SIDE_STORE_KEY, String(SIDE_DEFAULT));
  }
</script>

{#snippet dimHeader(dim, label)}
  {#if ui[dim].open}
    <ConditionCard
      title={label}
      filled={dimFilled(dim, ui[dim])}
      enabled={ui[dim].enabled}
      ontoggle={() => (ui[dim].enabled = !ui[dim].enabled)}
      onclear={() => clearDim(dim)}
    >
      {@render dimBody(dim)}
    </ConditionCard>
  {:else}
    <button type="button" class="query-dim" onclick={() => (ui[dim].open = true)}>
      <span class="query-dim__plus">＋</span>{label}
    </button>
  {/if}
{/snippet}

{#snippet dimBody(dim)}
  {#if dim === 'status'}
    <div class="cond-chips">
      {#each STATUS_KEYS as s (s)}
        <button
          type="button"
          class="cond-chip"
          class:is-active={ui.status.values.includes(s)}
          onclick={() => toggleIn(ui.status.values, s)}
        >{statusLabel(s)}</button>
      {/each}
    </div>
  {:else if dim === 'rating' || dim === 'length'}
    <div class="cond-field">
      <div class="cond-source">
        <button
          type="button"
          class="cond-source__btn"
          class:is-active={ui[dim].source === 'vndb'}
          onclick={() => (ui[dim].source = 'vndb')}
        >{t('query.sourceVndb')}</button>
        <button
          type="button"
          class="cond-source__btn"
          class:is-active={ui[dim].source === 'user'}
          onclick={() => (ui[dim].source = 'user')}
        >{t('query.sourceUser')}</button>
      </div>
      <div class="cond-numbers">
        <NumberStepper
          bind:value={ui[dim].min}
          step={dim === 'rating' ? 0.1 : 1}
          max={dim === 'rating' ? 10 : null}
          placeholder={t('query.min')}
          ariaLabel={t('query.min')}
        />
        <span class="cond-numbers__sep">{t('query.to')}</span>
        <NumberStepper
          bind:value={ui[dim].max}
          step={dim === 'rating' ? 0.1 : 1}
          max={dim === 'rating' ? 10 : null}
          placeholder={t('query.max')}
          ariaLabel={t('query.max')}
        />
        {#if dim === 'length'}
          <span class="cond-numbers__unit">{t('query.hours')}</span>
        {/if}
      </div>
    </div>
  {:else if dim === 'released'}
    <DateRangeField bind:from={ui.released.from} bind:to={ui.released.to} />
  {:else if dim === 'developer'}
    <input
      type="text"
      class="cond-text"
      placeholder={t('query.devPlaceholder')}
      bind:value={ui.developer.name}
    />
  {/if}
{/snippet}

<section class="view view--query">
  <header class="view-header">
    <h1>{t('query.title')}</h1>
  </header>

  <div class="query-body">
    <aside class="query-side" style:width="{sideWidth}px">
      {#if ui.tags.open}
        <ConditionCard
          title={t('query.dim.tags')}
          filled={dimFilled('tags', ui.tags)}
          enabled={ui.tags.enabled}
          ontoggle={() => (ui.tags.enabled = !ui.tags.enabled)}
          onclear={() => clearDim('tags')}
        >
          <TagCondition tags={ui.tags} />
        </ConditionCard>
      {:else}
        <button type="button" class="query-dim" onclick={() => (ui.tags.open = true)}>
          <span class="query-dim__plus">＋</span>{t('query.dim.tags')}
        </button>
      {/if}

      {@render dimHeader('status', t('query.dim.status'))}
      {@render dimHeader('rating', t('query.dim.rating'))}
      {@render dimHeader('released', t('query.dim.released'))}
      {@render dimHeader('developer', t('query.dim.developer'))}
      {@render dimHeader('length', t('query.dim.length'))}

      <button type="button" class="query-side__clear" disabled={!active} onclick={clearAll}>
        {t('query.clearAll')}
      </button>
    </aside>

    <div
      class="query-resizer"
      class:is-active={resizing}
      role="separator"
      aria-orientation="vertical"
      data-tooltip={t('query.resizeHint')}
      onpointerdown={startResize}
      ondblclick={resetResize}
    ></div>

    <div class="query-results">
      <div class="query-toolbar">
        <span class="query-count"><b>{sorted.length}</b>{t('query.matchCount')}</span>
        <label class="query-sort">
          <span class="query-sort__label">{t('query.sort')}</span>
          <SelectDd options={SORT_OPTIONS} bind:value={sortBy} ariaLabel={t('query.sort')} />
        </label>
      </div>

      {#if loading}
        <p class="query-note">{t('common.loading')}</p>
      {:else if error}
        <p class="query-note">{t('query.error')}</p>
      {:else if sorted.length === 0}
        <div class="query-empty">
          <p>{t('query.empty')}</p>
          <p class="query-empty__hint">{t('query.emptyHint')}</p>
        </div>
      {:else if sorted.length >= 60}
        <WindowedGrid items={sorted}>
          {#snippet children(item)}
            <GameCard {item} onopen={() => openFromCard(item)} />
          {/snippet}
        </WindowedGrid>
      {:else}
        <div class="cover-grid">
          {#each sorted as item (item.id)}
            <GameCard {item} onopen={() => openFromCard(item)} />
          {/each}
        </div>
      {/if}
    </div>
  </div>
</section>
