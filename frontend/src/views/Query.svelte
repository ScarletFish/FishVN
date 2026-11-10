<script>
  import { onMount, onDestroy, tick, untrack } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { report } from '../lib/frontend-log.js';
  import { STATUS_KEYS, statusLabel } from '../lib/status.js';
  import { captureHero } from '../lib/hero-transition.js';
  import { onLibraryItemsThrottled } from '../lib/library-events.js';
  import { createQueryRunner } from '../lib/query-runner.js';
  import { getScrollViewport } from '../lib/scrollbars.js';
  import { dimFilled, emptyQueryUi, hasActive, uiToSpec } from '../lib/query-spec.js';
  import { queryStore } from '../lib/query-store.svelte.js';
  import { getSpoilerLevel } from '../lib/config-store.svelte.js';
  import { spoilerThreshold } from '../lib/tag-order.svelte.js';
  import SortSelect from '../components/SortSelect.svelte';
  import GameCard from '../components/GameCard.svelte';
  import WindowedGrid from '../components/WindowedGrid.svelte';
  import ConditionCard from '../components/query/ConditionCard.svelte';
  import TagCondition from '../components/query/TagCondition.svelte';
  import TraitCondition from '../components/query/TraitCondition.svelte';
  import VaCondition from '../components/query/VaCondition.svelte';
  import DateRangeField from '../components/query/DateRangeField.svelte';
  import NumberStepper from '../components/query/NumberStepper.svelte';
  import { save } from '@tauri-apps/plugin-dialog';
  import { toast } from '../lib/toast.svelte.js';
  import { buildQueryMarkdown } from '../lib/query-export.js';
  import { loadTagDict } from '../lib/tag-label.svelte.js';
  import { loadLibraryTags } from '../lib/library-tags.svelte.js';
  import { loadLibraryTraits } from '../lib/library-traits.svelte.js';
  import { staffLabel } from '../lib/staff-label.js';

  let { onopen, scroll = 0 } = $props();

  const ui = queryStore.ui;
  const spec = $derived(uiToSpec(ui, spoilerThreshold(getSpoilerLevel())));
  const active = $derived(hasActive(ui));

  let loading = $state(queryStore.lastJson[queryStore.ui.subject] === null);
  let error = $state(null);
  let unwatch = null;

  const runner = createQueryRunner({
    query: (s) => (s.subject === 'character' ? api.queryCharacters(s) : api.queryGames(s)),
    store: queryStore,
    onSettled: (err, info) => {
      if (!info) return;
      loading = false;
      error = err;
      if (err) report('warn', `筛选页查询失败：${info.subject}：${err}`);
      else report('debug', `筛选页查询：${info.subject} ${info.rows} 行 ${info.ms}ms`);
    },
  });

  const isChar = $derived(ui.subject === 'character');
  const shownIsChar = $derived(queryStore.resultsSubject === 'character');

  const SORT_OPTIONS = $derived(
    isChar
      ? [
          { value: 'name', label: t('query.sortName') },
          { value: 'game', label: t('query.sortCharGame') },
        ]
      : [
          { value: 'vndb', label: t('query.sortVndb') },
          { value: 'user', label: t('query.sortUser') },
          { value: 'name', label: t('query.sortName') },
          { value: 'added', label: t('query.sortAdded') },
        ],
  );

  $effect(() => {
    if (isChar && queryStore.sortBy !== 'name' && queryStore.sortBy !== 'game')
      queryStore.sortBy = 'name';
  });

  const charName = (hit) => hit.name ?? hit.original ?? hit.character_id ?? '';

  const sorted = $derived.by(() => {
    const list = queryStore.results.slice();
    const sort = queryStore.sortBy;
    if (sort === 'name')
      list.sort((a, b) => String(a.name ?? '').localeCompare(String(b.name ?? ''), 'zh-CN'));
    else if (sort === 'game')
      list.sort(
        (a, b) =>
          String(a.game_title ?? '').localeCompare(String(b.game_title ?? ''), 'zh-CN') ||
          charName(a).localeCompare(charName(b), 'zh-CN'),
      );
    else if (sort === 'user') list.sort((a, b) => (b.user_rating ?? -1) - (a.user_rating ?? -1));
    else if (sort === 'added') list.sort((a, b) => b.created_at - a.created_at);
    else list.sort((a, b) => (b.vndb_rating ?? -1) - (a.vndb_rating ?? -1));
    return list;
  });

  const neighborIds = $derived(sorted.map((h) => h.game_id ?? h.id));

  async function exportResults() {
    if (sorted.length === 0) return;
    if (queryStore.lastJson[queryStore.resultsSubject] !== JSON.stringify(spec)) {
      toast('筛选结果正在更新，请稍候再导出');
      return;
    }
    try {
      const [staff] = await Promise.all([
        api.listStaff().catch(() => []),
        loadTagDict(),
        loadLibraryTags(),
        loadLibraryTraits(),
      ]);
      const staffById = new Map((staff ?? []).map((s) => [s.id, s]));
      const md = buildQueryMarkdown({
        subject: queryStore.resultsSubject,
        ui: queryStore.ui,
        rows: sorted,
        labels: { va: (id) => staffLabel(staffById.get(id)) },
      });
      const now = new Date();
      const p = (n) => String(n).padStart(2, '0');
      const stamp = `${now.getFullYear()}${p(now.getMonth() + 1)}${p(now.getDate())}`;
      const path = await save({
        defaultPath: `fishvn-query-${queryStore.resultsSubject}-${stamp}.md`,
        filters: [{ name: 'Markdown', extensions: ['md'] }],
      });
      if (!path) return;
      await api.writeTextFile(path, md);
      toast(`已导出 ${sorted.length} 条到 ${path}`);
    } catch (e) {
      report('warn', `导出筛选结果失败：${e}`);
      toast('导出失败，详见日志', 'error');
    }
  }

  $effect(() => {
    const s = spec;
    untrack(() => {
      loading = !runner.paint(s);
    });
    const timer = setTimeout(() => runner.load(s), 100);
    return () => clearTimeout(timer);
  });

  let restored = $state(false);

  $effect(() => {
    if (restored || scroll <= 0) return;
    if (loading || error) return;
    if (queryStore.resultsSubject !== ui.subject) return;
    const viewport = getScrollViewport();
    if (!viewport) return;
    restored = true;
    const target = scroll;
    tick().then(() => {
      viewport.scrollTop = target;
    });
  });

  onMount(() => {
    unwatch = onLibraryItemsThrottled(reload);
  });

  onDestroy(() => {
    unwatch?.();
  });

  function reload() {
    return runner.load(spec, { push: true });
  }

  function setSubject(next) {
    ui.subject = next;
  }

  function openFromCard(item) {
    const card = document.querySelector(`.cover-card[data-game-id="${CSS.escape(String(item.id))}"]`);
    if (card) captureHero(card);
    onopen(item.id, { neighbors: neighborIds });
  }

  function clearDim(dim) {
    ui[dim] = emptyQueryUi()[dim];
  }

  function clearAll() {
    const { subject, ...dims } = emptyQueryUi();
    Object.assign(ui, dims);
  }

  function toggleIn(list, value) {
    const idx = list.indexOf(value);
    if (idx >= 0) list.splice(idx, 1);
    else list.push(value);
  }

  const ROLE_KEYS = ['main', 'primary', 'side', 'appears'];
  const GENDER_KEYS = ['f', 'm', 't', 'i'];

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
  {:else if dim === 'va'}
    <VaCondition va={ui.va} />
  {:else if dim === 'role'}
    <div class="cond-chips">
      {#each ROLE_KEYS as r (r)}
        <button
          type="button"
          class="cond-chip"
          class:is-active={ui.role.values.includes(r)}
          onclick={() => toggleIn(ui.role.values, r)}
        >{t(`gameForm.charRole.${r}`)}</button>
      {/each}
    </div>
  {:else if dim === 'gender'}
    <div class="cond-chips">
      {#each GENDER_KEYS as g (g)}
        <button
          type="button"
          class="cond-chip"
          class:is-active={ui.gender.values.includes(g)}
          onclick={() => toggleIn(ui.gender.values, g)}
        >{t(`detail.gender.${g}`)}</button>
      {/each}
    </div>
  {:else if dim === 'charname'}
    <input
      type="text"
      class="cond-text"
      placeholder={t('query.charnamePlaceholder')}
      bind:value={ui.charname.value}
    />
  {/if}
{/snippet}

<section class="view view--query">
  <header class="view-header">
    <h1>{t('query.title')}</h1>
  </header>

  <div class="query-body">
    <aside class="query-side" style:width="{sideWidth}px">
      <div class="query-side__scroll">
      <div class="query-side__group">
        <span class="query-side__group-title">{t('query.groupGame')}</span>
        <span class="query-side__group-sub">
          {isChar ? t('query.groupGameSubChar') : t('query.groupGameSubGame')}
        </span>
      </div>

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

      <div class="query-side__group query-side__group--sep">
        <span class="query-side__group-title">{t('query.groupChar')}</span>
        <span class="query-side__group-sub">
          {isChar ? t('query.groupCharSubChar') : t('query.groupCharSubGame')}
        </span>
      </div>

      {#if ui.traits.open}
        <ConditionCard
          title={t('query.dim.traits')}
          filled={dimFilled('traits', ui.traits)}
          enabled={ui.traits.enabled}
          ontoggle={() => (ui.traits.enabled = !ui.traits.enabled)}
          onclear={() => clearDim('traits')}
        >
          <TraitCondition traits={ui.traits} />
        </ConditionCard>
      {:else}
        <button type="button" class="query-dim" onclick={() => (ui.traits.open = true)}>
          <span class="query-dim__plus">＋</span>{t('query.dim.traits')}
        </button>
      {/if}

      {@render dimHeader('role', t('query.dim.role'))}
      {@render dimHeader('va', t('query.dim.va'))}
      {@render dimHeader('gender', t('query.dim.gender'))}
      {@render dimHeader('charname', t('query.dim.charname'))}

      </div>
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
        <div class="query-subject" role="radiogroup" aria-label={t('query.subject')}>
          <button
            type="button"
            class="query-subject__btn"
            class:is-active={!isChar}
            onclick={() => setSubject('game')}
          >{t('query.subjectGame')}</button>
          <button
            type="button"
            class="query-subject__btn"
            class:is-active={isChar}
            onclick={() => setSubject('character')}
          >{t('query.subjectChar')}</button>
        </div>
        <span class="query-count">
          <b>{sorted.length}</b>{shownIsChar ? t('query.matchCountChar') : t('query.matchCount')}
        </span>
        <div class="query-toolbar__right">
          <SortSelect options={SORT_OPTIONS} bind:value={queryStore.sortBy} />
          <button
            type="button"
            class="btn btn-sm btn-primary"
            onclick={exportResults}
            disabled={sorted.length === 0}
            data-tooltip={t('query.exportHint')}
          >{t('query.export')}</button>
        </div>
      </div>

      {#if loading}
        <p class="query-note">{t('common.loading')}</p>
      {:else if error}
        <p class="query-note">{t('query.error')}</p>
      {:else if shownIsChar !== isChar}
        <p class="query-note">{t('common.loading')}</p>
      {:else if sorted.length === 0}
        <div class="query-empty">
          <p>{isChar ? t('query.emptyChar') : t('query.empty')}</p>
          <p class="query-empty__hint">{t('query.emptyHint')}</p>
        </div>
      {:else if shownIsChar}
        <div class="char-list">
          {#each sorted as hit ((hit.game_id ?? hit.id) + '/' + (hit.character_id ?? ''))}
            <button type="button" class="char-row" onclick={() => onopen(hit.game_id, { characterId: hit.character_id, neighbors: neighborIds })}>
              <span class="char-row__main">
                <span class="char-row__name">{charName(hit)}</span>
                {#if hit.name && hit.original}
                  <span class="char-row__original">{hit.original}</span>
                {/if}
              </span>
              <span class="char-row__game">{hit.game_title}</span>
            </button>
          {/each}
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
