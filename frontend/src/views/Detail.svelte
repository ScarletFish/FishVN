<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { coverSrc } from '../lib/cover-src.js';
  import DetailCharacters from './DetailCharacters.svelte';
  import GameForm from '../components/GameForm.svelte';
  import Menu from '../components/Menu.svelte';

  let { gameId, onclose } = $props();

  let game = $state(null);
  let meta = $state(null);
  let loading = $state(true);
  let failed = $state(false);
  let formOpen = $state(false);
  let menuOpen = $state(false);
  let coverUrl = $state(null);
  let coverBroken = $state(false);

  const TAGS_PREVIEW = 8;
  let tagsExpanded = $state(false);

  let summaryParas = $derived((meta?.description ?? '').split('\n\n').filter(Boolean));

  let alias = $derived(meta?.alias ?? '');

  // VNDB 的 rating 是百分制，界面按十分制展示。91 要显示 9.1 而不是 9.10：
  // 固定两位会补出一个没有信息量的 0
  function trimScore(value) {
    return String(Number(value.toFixed(2)));
  }

  let ratingScore = $derived(meta?.rating != null ? trimScore(meta.rating / 10) : '');
  let releaseYear = $derived(meta?.released?.slice(0, 4) ?? '');
  let developer = $derived(meta?.developers?.[0] ?? '');

  // spoiler ≥ 1 的不展示；rating 集中在一个值上，同值再按 vn_count 排。
  // 用户手打的标签没有这两项，当 0 排，自然落到 VNDB 标签后面
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
    tagsExpanded = false;
    menuOpen = false;

    let live = true;
    // 元数据行可能不存在（没绑过 VNDB），那不是失败，页面按空值渲染
    Promise.all([api.getGame(gameId), api.getGameMetadata(gameId)])
      .then(([g, m]) => {
        if (!live) return;
        game = g;
        meta = m;
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
    // 3 = 鼠标后退侧键；4 是前进，不接
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
              <span class="detail-cover__label">{t('detail.coverEmpty')}</span>
            </div>
          {/if}
        </div>
        <!-- 两个按钮只占位不接动作：启动需要后端有 launch command，删除同理 -->
        <div class="detail-actions">
          <button type="button" class="btn btn-primary">{t('detail.actionPlay')}</button>
          <button type="button" class="btn btn-danger">{t('detail.actionDelete')}</button>
          <Menu
            bind:open={menuOpen}
            label={t('common.actionMore')}
            items={[{ label: t('common.actionEdit'), onclick: () => (formOpen = true) }]}
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
            <p>{para}</p>
          {/each}
        </div>
      </div>
    </div>
    <DetailCharacters />
  {/if}
</section>

<GameForm bind:open={formOpen} gameId={gameId} />
