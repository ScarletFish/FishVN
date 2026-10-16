<script>
  import { tick } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { api } from '../lib/api.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { displayName } from '../lib/characters.js';
  import { stripBbcode } from '../lib/vn-merge.js';

  // VNDB 的 trait 分组无序号字段，固定一个展示顺序；不在这张表里的新分组排到末尾
  const TRAIT_GROUP_ORDER = [
    'Hair',
    'Eyes',
    'Body',
    'Clothes',
    'Items',
    'Personality',
    'Role',
    'Engages in',
    'Subject of',
  ];
  const groupRank = (name) => {
    const i = TRAIT_GROUP_ORDER.indexOf(name);
    return i < 0 ? TRAIT_GROUP_ORDER.length : i;
  };

  let { gameId } = $props();

  let scrollEl = $state(null);
  let headerEl = $state(null);
  let panelEl = $state(null);
  let expandedId = $state(null);
  let characters = $state([]);
  let artUrls = $state({});

  $effect(() => {
    characters = [];
    expandedId = null;
    let live = true;
    api
      .listGameCharacters(gameId)
      .then((list) => {
        if (live) characters = list;
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  });

  // 立绘走封面那条 coverSrc 管线：缩图与 asset 协议授权都在那边做过了
  $effect(() => {
    let live = true;
    for (const character of characters) {
      const path = character.image_path?.trim();
      if (!path || artUrls[path]) continue;
      coverSrc(path)
        .then((url) => {
          if (live) artUrls = { ...artUrls, [path]: url };
        })
        .catch(() => {});
    }
    return () => {
      live = false;
    };
  });

  function art(path) {
    if (!path) return null;
    return artUrls[path.trim()] ?? null;
  }

  const altName = (c) => {
    const name = (c.name ?? '').trim();
    return name === displayName(c) ? '' : name;
  };
  const vaName = (v) => v?.original || v?.name || '';

  const expanded = $derived(characters.find((c) => c.id === expandedId) ?? null);

  $effect(() => {
    if (!scrollEl || !headerEl) return;
    return initScrollDots({
      scroll: scrollEl,
      total: characters.length,
      dotsParent: headerEl,
    });
  });

  // 缺失词条回落 key 会显示成 detail.xxx，靠它区分并保留原文
  const localized = (key, fallback) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  const traitGroups = $derived(
    (() => {
      const map = new Map();
      // spoiler ≥ 1 的不展示；成人向分组不进正文
      for (const tr of expanded?.traits ?? []) {
        if (tr.spoiler || tr.group_name.includes('Sexual')) continue;
        if (!map.has(tr.group_name)) map.set(tr.group_name, []);
        map.get(tr.group_name).push(tr.name);
      }
      return [...map.entries()]
        .sort((a, b) => groupRank(a[0]) - groupRank(b[0]))
        .map(([group, names]) => ({
          label: localized(`detail.traitGroups.${group}`, group),
          names,
        }));
    })()
  );

  const facts = $derived(
    [
      expanded?.gender?.length && {
        label: t('detail.charGender'),
        value: expanded.gender
          .map((g) => localized(`detail.gender.${g}`, g))
          .join(' / '),
      },
      (expanded?.height || expanded?.weight) && {
        label: t('detail.charDimensions'),
        value: [expanded?.height && `${expanded.height} ${t('detail.unitCm')}`, expanded?.weight && `${expanded.weight} ${t('detail.unitKg')}`]
          .filter(Boolean)
          .join(' / '),
      },
      expanded?.age && { label: t('detail.charAge'), value: String(expanded.age) },
      expanded?.va?.length && {
        label: t('detail.charVoice'),
        value: expanded.va.map(vaName).filter(Boolean).join(' / '),
      },
    ].filter(Boolean)
  );

  async function toggle(id) {
    expandedId = expandedId === id ? null : id;
    if (!expandedId) return;
    await tick();
    panelEl?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
</script>

{#if characters.length}
  <section class="detail-chars section-card">
    <div class="detail-chars-header" bind:this={headerEl}>
      <h3>{t('detail.characters')}</h3>
      <span class="detail-chars-count">{characters.length}</span>
    </div>

    <div class="hscroll-section detail-chars-scroll" bind:this={scrollEl}>
      {#each characters as c (c.id)}
        <div
          class="hscroll-card detail-char-card"
          class:expanded={expandedId === c.id}
          role="button"
          tabindex="0"
          aria-expanded={expandedId === c.id}
          onclick={() => toggle(c.id)}
          onkeydown={(e) => {
            if (e.key === 'Enter') toggle(c.id);
          }}
        >
          <div class="detail-char-portrait">
            {#if art(c.image_path)}
              <img src={art(c.image_path)} alt={displayName(c)} />
            {:else}
              <div class="detail-char-portrait-ph">
                <span class="detail-char-ph-name">{displayName(c)}</span>
                {#if vaName(c.va?.[0])}
                  <span class="detail-char-ph-cv">{vaName(c.va[0])}</span>
                {/if}
              </div>
            {/if}
          </div>
          {#if art(c.image_path)}
            <div class="detail-char-foot">
              <span class="detail-char-name">{displayName(c)}</span>
              {#if vaName(c.va?.[0])}
                <span class="detail-char-cv">{vaName(c.va[0])}</span>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if expanded}
      <div class="detail-char-panel" bind:this={panelEl}>
        <div class="detail-char-panel-body">
          <h4>{displayName(expanded)}</h4>
          {#if altName(expanded)}
            <p class="detail-char-original">{altName(expanded)}</p>
          {/if}

          {#if facts.length}
            <dl class="detail-char-facts">
              {#each facts as f (f.label)}
                <dt>{f.label}</dt>
                <dd>{f.value}</dd>
              {/each}
            </dl>
          {/if}

          {#if stripBbcode(expanded.description)}
            <p class="detail-char-desc">{stripBbcode(expanded.description)}</p>
          {/if}

          {#if traitGroups.length}
            <dl class="detail-char-traits">
              {#each traitGroups as g (g.label)}
                <dt>{g.label}</dt>
                <dd>{g.names.join(', ')}</dd>
              {/each}
            </dl>
          {/if}
        </div>
      </div>
    {/if}
  </section>
{/if}
