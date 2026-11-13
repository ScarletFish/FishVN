<script>
  import { tick } from 'svelte';
  import Icon from '../Icon.svelte';
  import { api } from '../../lib/api.js';
  import { t } from '../../lib/i18n.js';
  import { tagLabel } from '../../lib/tag-label.svelte.js';
  import { panelScrollbars } from '../../lib/scrollbars.js';
  import { LIBRARY_TAGS_CHANGED } from '../../lib/library-events.js';
  import { shouldReduceMotion } from '../../lib/motion.js';

  const CLOSE_MS = 180;

  let { onsettings } = $props();

  let query = $state('');
  let games = $state([]);
  let open = $state(false);
  let expanded = $state(false);
  let closing = $state(false);
  let closeTimer = null;
  let highlighted = $state(-1);
  let suppressNextDocClick = false;
  let containerEl = $state(null);
  let inputEl = $state(null);

  let tagPool = $state(null);
  let tagLoadPromise = null;

  let results = $derived(filterByQuery(query));

  function asText(v) {
    return v == null ? '' : String(v);
  }

  function matchGame(game, q) {
    return [game.name, game.alias, game.vndb_id, game.va_names, game.tag_names]
      .map((f) => asText(f).toLowerCase())
      .some((f) => f.includes(q));
  }

  function filterByQuery(raw) {
    const q = raw.trim().toLowerCase();
    if (!q) return [];
    return games.filter((game) => matchGame(game, q));
  }

  const SETTINGS_MAP = () => [
    { label: t('settings.theme'), keywords: '主题 外观 暗色 深色 浅色 跟随系统 黑夜 theme dark light system' },
    { label: t('settings.accent'), keywords: '强调色 主题色 颜色 accent color' },
    { label: t('settings.fontSize'), keywords: '字体 字号 大小 缩放 font size' },
    { label: t('settings.timeMetric'), keywords: '时长 统计 口径 前台 存活 计时' },
    { label: t('settings.dashboardLayout'), keywords: '首页 布局 模块 排序 dashboard layout' },
    { label: t('settings.dashboardIncludeUninstalled'), keywords: '首页 未安装 显示 作品' },
    { label: t('settings.hideR18'), keywords: 'r18 成人 隐藏 过滤 标签' },
    { label: t('settings.spoilerLevel'), keywords: '剧透 剧情隐藏 spoiler' },
    { label: t('settings.excludedTags'), keywords: '排除 标签 黑名单 排除池 excluded tags' },
    { label: t('settings.tools.magpie'), keywords: 'magpie 放大 超分 缩放 画质 工具 路径' },
    { label: t('settings.tools.le'), keywords: '转区 locale emulator leproc 日文 乱码 工具 路径' },
    { label: t('settings.vndb.mode'), keywords: 'vndb 同步 模式 上传 下载 sync' },
    { label: t('settings.vndb.syncWishlist'), keywords: 'vndb 同步 想玩 愿望单 导入 wishlist' },
    { label: t('settings.vndb.token'), keywords: 'vndb token 令牌 凭证 连接 授权' },
    { label: t('settings.vndb.syncNow'), keywords: 'vndb 同步 立即 sync now' },
  ];

  let settingHits = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return SETTINGS_MAP().filter((s) => `${s.label} ${s.keywords}`.toLowerCase().includes(q));
  });

  const isTagMode = $derived(query.startsWith('#'));
  const tagQuery = $derived(isTagMode ? query.slice(1).trim().toLowerCase() : '');

  const tagList = $derived.by(() => {
    if (!tagPool) return [];
    const out = [];
    for (const tg of tagPool) {
      const label = tagLabel(tg);
      if (!label) continue;
      if (tagQuery && !label.toLowerCase().includes(tagQuery)) continue;
      out.push({ id: tg.id, name: tg.name, count: tg.count, label });
    }
    return out;
  });

  let flatItems = $derived.by(() => {
    if (isTagMode) {
      return tagList.map((tg) => ({ type: 'tag', id: tg.id, label: tg.label }));
    }
    return [
      ...results.map((g) => ({ type: 'game', id: g.id, label: g.name })),
      ...settingHits.map((s) => ({ type: 'settings', label: s.label })),
    ];
  });

  function highlightParts(text, q) {
    if (!text || !q) return [{ text: text || '', match: false }];
    const lower = text.toLowerCase();
    const parts = [];
    let idx = 0;
    while (idx < text.length) {
      const found = lower.indexOf(q, idx);
      if (found === -1) {
        parts.push({ text: text.slice(idx), match: false });
        break;
      }
      if (found > idx) parts.push({ text: text.slice(idx, found), match: false });
      parts.push({ text: text.slice(found, found + q.length), match: true });
      idx = found + q.length;
    }
    return parts;
  }

  function navigateTo(item) {
    if (item.type === 'tag') {
      pickTag(item.label);
      return;
    }
    closePanel();
    if (item.type === 'game') {
      document.dispatchEvent(new CustomEvent('search-open-game', { detail: item.id }));
    } else if (item.type === 'settings') {
      onsettings?.();
    }
  }

  function pickTag(label) {
    if (!label) return;
    applyQuery(label);
  }

  function ensureTags() {
    if (tagPool) return;
    if (!tagLoadPromise) {
      tagLoadPromise = api
        .libraryTags()
        .then((list) => (Array.isArray(list) ? list : []))
        .catch(() => []);
    }
    tagLoadPromise.then((list) => {
      tagPool = list;
    });
  }

  $effect(() => {
    const refill = () => {
      tagPool = null;
      tagLoadPromise = null;
      ensureTags();
    };
    window.addEventListener(LIBRARY_TAGS_CHANGED, refill);
    return () => window.removeEventListener(LIBRARY_TAGS_CHANGED, refill);
  });

  async function openPanel({ select = true } = {}) {
    clearTimeout(closeTimer);
    closing = false;
    const wasClosed = !expanded;
    open = true;
    if (wasClosed) {
      expanded = true;
      highlighted = -1;
      refreshGames();
      if (query.startsWith('#')) ensureTags();
      await tick();
    }
    inputEl?.focus();
    if (select) {
      inputEl?.select();
      return;
    }
    const end = inputEl?.value.length ?? 0;
    inputEl?.setSelectionRange(end, end);
  }

  function closePanel() {
    if (!expanded || closing) return;
    open = false;
    highlighted = -1;
    if (shouldReduceMotion()) {
      expanded = false;
      return;
    }
    closing = true;
    clearTimeout(closeTimer);
    closeTimer = setTimeout(() => {
      expanded = false;
      closing = false;
    }, CLOSE_MS);
  }

  function applyQuery(name) {
    if (!name) return;
    suppressNextDocClick = true;
    query = name;
    openPanel({ select: false });
  }

  async function refreshGames() {
    try {
      games = await api.listGames();
    } catch (_) {
      games = [];
    }
  }

  function onInput() {
    if (query.startsWith('#')) ensureTags();
  }

  function onKeydown(e) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closePanel();
      return;
    }
    const items = flatItems;
    if (items.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlighted = highlighted < items.length - 1 ? highlighted + 1 : 0;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlighted = highlighted <= 0 ? items.length - 1 : highlighted - 1;
    } else if (e.key === 'Enter') {
      const target = highlighted >= 0 ? items[highlighted] : items[0];
      if (target) navigateTo(target);
    }
  }

  $effect(() => {
    if (highlighted < 0 || !open) return;
    const el = containerEl?.querySelectorAll('.titlebar__search-item')[highlighted];
    el?.scrollIntoView({ block: 'nearest' });
  });

  function onDocClick(e) {
    if (!open) return;
    if (suppressNextDocClick) {
      suppressNextDocClick = false;
      return;
    }
    if (containerEl && !containerEl.contains(e.target)) closePanel();
  }

  function onScroll(e) {
    if (!open) return;
    if (containerEl && e.target && containerEl.contains(e.target)) return;
    closePanel();
  }

  function onDocKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
      e.preventDefault();
      openPanel();
      return;
    }
    if (e.key === '/' && !e.ctrlKey && !e.metaKey) {
      const tag = e.target?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || e.target?.isContentEditable) return;
      e.preventDefault();
      openPanel();
    }
  }

  function onSetQuery(e) {
    applyQuery(e.detail);
  }

  $effect(() => {
    document.addEventListener('click', onDocClick);
    document.addEventListener('scroll', onScroll, true);
    document.addEventListener('keydown', onDocKeydown);
    document.addEventListener('search-set-query', onSetQuery);
    return () => {
      document.removeEventListener('click', onDocClick);
      document.removeEventListener('scroll', onScroll, true);
      document.removeEventListener('keydown', onDocKeydown);
      document.removeEventListener('search-set-query', onSetQuery);
    };
  });

  $effect(() => () => clearTimeout(closeTimer));
</script>

<div
  class="titlebar__search"
  class:is-open={expanded}
  class:is-closing={closing}
  bind:this={containerEl}
>
  {#if expanded}
    <div class="titlebar__search-field">
      <span class="titlebar__search-mark">
        <Icon name="magnifying-glass" />
      </span>
      <input
        bind:this={inputEl}
        bind:value={query}
        oninput={onInput}
        onkeydown={onKeydown}
        placeholder={t('titlebar.searchPlaceholder')}
        autocomplete="off"
        spellcheck="false"
      />
    </div>

    {#if query.trim()}
      <div class="titlebar__search-results" use:panelScrollbars>
        {#if isTagMode}
          {#if tagList.length === 0}
            <div class="titlebar__search-empty">{t('titlebar.tagSearchEmpty')}</div>
          {:else}
            <div class="titlebar__search-group">{t('titlebar.searchGroupTags')}</div>
            {#each tagList as tag (tag.id)}
              <button
                type="button"
                class="titlebar__search-item"
                class:highlighted={flatItems[highlighted]?.type === 'tag' && flatItems[highlighted]?.id === tag.id}
                onclick={() => pickTag(tag.label)}
              >
                <span class="titlebar__search-item-text">
                  <span class="titlebar__search-item-label">
                    {#each highlightParts(tag.label, tagQuery) as part}
                      <span class:titlebar__search-item-mark={part.match}>{part.text}</span>
                    {/each}
                  </span>
                </span>
                <span class="titlebar__search-item-count">{tag.count}</span>
              </button>
            {/each}
          {/if}
        {:else}
          {#if flatItems.length === 0}
            <div class="titlebar__search-empty">{t('titlebar.searchEmpty')}</div>
          {:else}
            {#if results.length}
              <div class="titlebar__search-group">
                {t('titlebar.searchGroupGames')}
                <span class="titlebar__search-count">{t('titlebar.searchCount', { count: results.length })}</span>
              </div>
              {#each results as game (game.id)}
                <button
                  type="button"
                  class="titlebar__search-item"
                  class:highlighted={flatItems[highlighted]?.id === game.id && flatItems[highlighted]?.type === 'game'}
                  onclick={() => navigateTo({ type: 'game', id: game.id, label: game.name })}
                >
                  <span class="titlebar__search-item-avatar">{(game.name || '?')[0]}</span>
                  <span class="titlebar__search-item-text">
                    <span class="titlebar__search-item-label">
                      {#each highlightParts(game.name, query.trim().toLowerCase()) as part}
                        <span class:titlebar__search-item-mark={part.match}>{part.text}</span>
                      {/each}
                    </span>
                    {#if asText(game.alias)}
                      <span class="titlebar__search-item-sublabel">
                        {#each highlightParts(game.alias, query.trim().toLowerCase()) as part}
                          <span class:titlebar__search-item-mark={part.match}>{part.text}</span>
                        {/each}
                      </span>
                    {/if}
                  </span>
                </button>
              {/each}
            {/if}
            {#if settingHits.length}
              <div class="titlebar__search-group">{t('titlebar.searchGroupSettings')}</div>
              {#each settingHits as s (s.label)}
                <button
                  type="button"
                  class="titlebar__search-item"
                  class:highlighted={flatItems[highlighted]?.type === 'settings' && flatItems[highlighted]?.label === s.label}
                  onclick={() => navigateTo({ type: 'settings', label: s.label })}
                >
                  <span class="titlebar__search-item-icon">
                    <Icon name="gear" />
                  </span>
                  <span class="titlebar__search-item-text">
                    <span class="titlebar__search-item-label">
                      {#each highlightParts(s.label, query.trim().toLowerCase()) as part}
                        <span class:titlebar__search-item-mark={part.match}>{part.text}</span>
                      {/each}
                    </span>
                  </span>
                </button>
              {/each}
            {/if}
          {/if}
        {/if}
      </div>
      {/if}
  {:else}
    <button
      type="button"
      class="titlebar__iconbtn"
      aria-label={t('titlebar.search')}
      data-tooltip={t('titlebar.search')}
      onclick={(e) => {
        e.stopPropagation();
        openPanel();
      }}
    >
      <Icon name="magnifying-glass" />
    </button>
  {/if}
</div>
