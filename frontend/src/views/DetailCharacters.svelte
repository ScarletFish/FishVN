<script>
  import { tick, onMount } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { api } from '../lib/api.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { displayName } from '../lib/characters.js';
  import { stripBbcode, bbcodeSegments } from '../lib/vn-merge.js';
  import { traitLabel } from '../lib/tag-label.svelte.js';
  import { openExternal } from '../lib/open-external.js';
  import { getSpoilerLevel } from '../lib/config-store.svelte.js';
  import { spoilerThreshold } from '../lib/tag-order.svelte.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { LIBRARY_TAGS_CHANGED, onLibraryItemUpdated } from '../lib/library-events.js';
  import DetailSection from '../components/DetailSection.svelte';

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
  let showSpoilers = $state(false);
  let characters = $state([]);
  let artUrls = $state({});

  $effect(() => {
    characters = [];
    expandedId = null;
    showSpoilers = false;
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

  async function refresh() {
    try {
      characters = await api.listGameCharacters(gameId);
    } catch {
      characters = [];
    }
  }

  $effect(() => {
    window.addEventListener(LIBRARY_TAGS_CHANGED, refresh);
    return () => window.removeEventListener(LIBRARY_TAGS_CHANGED, refresh);
  });

  onMount(() =>
    onLibraryItemUpdated((id) => {
      if (id !== gameId) return;
      refresh();
    }),
  );

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
  const vaKey = (v) => `${v?.staff_id ?? ''}|${v?.note ?? ''}`;

  function searchVa(va) {
    const name = vaName(va);
    if (!name) return;
    document.dispatchEvent(new CustomEvent('search-set-query', { detail: name }));
  }

  const expanded = $derived(characters.find((c) => c.id === expandedId) ?? null);

  $effect(() => {
    if (!scrollEl || !headerEl) return;
    return initScrollDots({
      scroll: scrollEl,
      total: characters.length,
      dotsParent: headerEl,
    });
  });

  const localized = (key, fallback) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  const traitData = $derived(
    (() => {
      const map = new Map();
      const threshold = spoilerThreshold(getSpoilerLevel());
      let spoilerCount = 0;
      for (const tr of expanded?.traits ?? []) {
        const group = tr.group_name || 'uncategorized';
        if (group.includes('Sexual')) continue;
        if ((tr.spoiler ?? 0) >= threshold) {
          spoilerCount += 1;
          if (!showSpoilers) continue;
        }
        if (!map.has(group)) map.set(group, []);
        map.get(group).push(tr);
      }
      const groups = [...map.entries()]
        .sort((a, b) => groupRank(a[0]) - groupRank(b[0]))
        .map(([group, traits]) => ({
          label: localized(`detail.traitGroups.${group}`, group),
          traits,
        }));
      return { groups, spoilerCount };
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
      expanded?.blood_type?.length && {
        label: t('detail.charBloodType'),
        value: localized(`detail.bloodType.${expanded.blood_type}`, expanded.blood_type.toUpperCase()),
      },
      (expanded?.bust != null || expanded?.waist != null || expanded?.hips != null || expanded?.cup?.length) && {
        label: t('detail.charBwh'),
        value: [
          expanded?.bust != null && `${expanded.bust}`,
          expanded?.waist != null && `${expanded.waist}`,
          expanded?.hips != null && `${expanded.hips}`,
          expanded?.cup?.length && `(${expanded.cup})`,
        ]
          .filter(Boolean)
          .join(' / '),
      },
      expanded?.birthday?.length && {
        label: t('detail.charBirthday'),
        value: expanded.birthday,
      },
    ].filter(Boolean)
  );

  async function toggle(id) {
    expandedId = expandedId === id ? null : id;
    showSpoilers = false;
    if (!expandedId) return;
    await tick();
    panelEl?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }
</script>

{#if characters.length}
  <DetailSection title={t('detail.characters')} count={characters.length} bind:headerEl class="detail-enter-chars">
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
                  <button type="button" class="detail-char-ph-cv detail-char-cv--link" onclick={() => searchVa(c.va?.[0])} data-tooltip={t('detail.charSearchVa')}>CV：{vaName(c.va?.[0])}</button>
                {/if}
              </div>
            {/if}
          </div>
          {#if art(c.image_path)}
            <div class="detail-char-foot">
              <span class="detail-char-name">{displayName(c)}</span>
              {#if vaName(c.va?.[0])}
                <button type="button" class="detail-char-cv detail-char-cv--link" onclick={() => searchVa(c.va?.[0])} data-tooltip={t('detail.charSearchVa')}>CV：{vaName(c.va?.[0])}</button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if expanded}
      <div class="detail-char-panel" bind:this={panelEl}>
        <div class="detail-char-panel-body">
          <div class="detail-char-title">
            <h4>{displayName(expanded)}</h4>
            {#if altName(expanded)}
              <span class="detail-char-original">{altName(expanded)}</span>
            {/if}
            {#if expanded?.aliases?.length}
              <span class="detail-char-aliases">{expanded.aliases.join(' / ')}</span>
            {/if}
          </div>

          {#if facts.length}
            <div class="detail-char-section">
              <p class="detail-char-section-label">{t('detail.charSectionFacts')}</p>
              <dl class="detail-char-facts">
                {#each facts as f (f.label)}
                  <div class="detail-char-fact">
                    <dt>{f.label}</dt>
                    <dd>{f.value}</dd>
                  </div>
                {/each}
              </dl>
            </div>
          {/if}

          {#if expanded?.va?.length}
            <div class="detail-char-section">
              <p class="detail-char-section-label">{t('detail.charVoice')}</p>
              <div class="detail-char-vas">
                {#each expanded.va as v (vaKey(v))}
                  <button
                    type="button"
                    class="detail-char-va"
                    onclick={() => searchVa(v)}
                    data-tooltip={t('detail.charSearchVa')}
                  >{vaName(v)}{#if v.note}<span class="detail-char-va-note">· {v.note}</span>{/if}</button>
                {/each}
              </div>
            </div>
          {/if}

          {#if stripBbcode(expanded.description)}
            <div class="detail-char-section">
              <p class="detail-char-section-label">{t('detail.charSectionDesc')}</p>
              <div class="detail-char-scroll" use:panelScrollbars>
                <p class="detail-char-desc">
                  {#each bbcodeSegments(expanded.description) as seg, i (i)}{#if seg.type === 'link'}<a
                      href={seg.href}
                      onclick={(e) => {
                        e.preventDefault();
                        openExternal(seg.href);
                      }}>{seg.text}</a
                    >{:else}{seg.text}{/if}{/each}
                </p>
              </div>
            </div>
          {/if}

          {#if traitData.groups.length || traitData.spoilerCount}
            <div class="detail-char-section">
              <p class="detail-char-section-label">{t('detail.charSectionTraits')}</p>
              {#if traitData.groups.length}
                <dl class="detail-char-traits">
                  {#each traitData.groups as g (g.label)}
                    <dt>{g.label}</dt>
                    <dd>
                      {#each g.traits as tr (tr.id ?? tr.name)}
                        <span class="pill pill--tag">{traitLabel(tr)}</span>
                      {/each}
                    </dd>
                  {/each}
                </dl>
              {/if}
              {#if traitData.spoilerCount}
                <div class="detail-char-spoilers">
                  <span>
                    {showSpoilers
                      ? t('detail.charSpoilersShown')
                      : t('detail.charSpoilersHidden', { count: traitData.spoilerCount })}
                  </span>
                  <label class="toggle-switch">
                    <input
                      type="checkbox"
                      bind:checked={showSpoilers}
                      aria-label={t('detail.charSpoilersShown')}
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
              {/if}
            </div>
          {/if}

          {#if /^c\d+$/.test(expanded.id)}
            <a
              class="detail-char-vndb"
              href={`https://vndb.org/${expanded.id}`}
              onclick={(e) => {
                e.preventDefault();
                openExternal(`https://vndb.org/${expanded.id}`);
              }}
            >
              {t('detail.charVndbLink')}
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path
                  d="M3 9L9 3M9 3H4.5M9 3V7.5"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </a>
          {/if}
        </div>
      </div>
    {/if}
  </DetailSection>
{/if}
