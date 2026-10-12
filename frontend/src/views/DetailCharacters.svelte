<script>
  import { tick } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { initScrollDots } from '../lib/scroll-dots.svelte.js';
  import { SAMPLE_CHARACTERS } from '../lib/sample-characters.js';

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

  let scrollEl = $state(null);
  let headerEl = $state(null);
  let panelEl = $state(null);
  let expandedId = $state(null);

  const expanded = $derived(SAMPLE_CHARACTERS.find((c) => c.id === expandedId) ?? null);

  $effect(() => {
    if (!scrollEl || !headerEl) return;
    return initScrollDots({
      scroll: scrollEl,
      total: SAMPLE_CHARACTERS.length,
      dotsParent: headerEl,
    });
  });

  // 描述里的 BBCode 只保留可见文字；[spoiler] 连内容一起丢弃，
// 25 个角色的 description 里 15 个带 [spoiler]，只剥标签会让剧透内容原样露出
  const plain = (s) =>
    (s ?? '')
      .replace(/\[spoiler\][\s\S]*?\[\/spoiler\]/gi, '')
      .replace(/\[\/?[a-z]+(?:=[^\]]*)?\]/gi, '');

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
        value: expanded.va
          .map((v) => `${v.name}${v.original ? ` (${v.original})` : ''}`)
          .join(' / '),
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

<section class="detail-chars section-card">
  <div class="detail-chars-header" bind:this={headerEl}>
    <h3>{t('detail.characters')}</h3>
    <span class="detail-chars-count">{SAMPLE_CHARACTERS.length}</span>
  </div>

  <div class="hscroll-section detail-chars-scroll" bind:this={scrollEl}>
    {#each SAMPLE_CHARACTERS as c (c.id)}
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
          {#if c.image?.url}
            <img src={c.image.url} alt={c.name} />
          {/if}
        </div>
        <div class="detail-char-foot">
          <span class="detail-char-name">{c.original ?? c.name}</span>
          {#if c.va?.[0]?.original}
            <span class="detail-char-cv">{c.va[0].original}</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if expanded}
    <div class="detail-char-panel" bind:this={panelEl}>
      <div class="detail-char-panel-art">
        {#if expanded.image?.url}
          <img src={expanded.image.url} alt={expanded.name} />
        {/if}
      </div>
      <div class="detail-char-panel-body">
        <h4>{expanded.name}</h4>
        {#if expanded.original}
          <p class="detail-char-original">{expanded.original}</p>
        {/if}

        {#if facts.length}
          <dl class="detail-char-facts">
            {#each facts as f (f.label)}
              <dt>{f.label}</dt>
              <dd>{f.value}</dd>
            {/each}
          </dl>
        {/if}

        {#if plain(expanded.description)}
          <p class="detail-char-desc">{plain(expanded.description)}</p>
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