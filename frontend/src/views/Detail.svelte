<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';

  let { gameId, onclose } = $props();

  let game = $state(null);
  let loading = $state(true);
  let failed = $state(false);

  // 临时：后端元数据链落地前，借 vndb-sample 里 FSN 的真实封面验证视觉效果。
  // csp 为 null，远程图可直接加载。元数据就位后删掉本常量与下方 img。
  const SAMPLE_COVER = 'https://t.vndb.org/cv/98/75598.jpg';

  // 临时：字段形态对齐 VNDB 的 title / alttitle（原文、本地化标题），后端元数据落地后删掉本常量。
  const SAMPLE_META = {
    title: 'Fate/stay night',
    title_cn: '圣杯战争',
    title_jp: 'フェイト／ステイナイト',
    rating: 86.3,
    votecount: 19334,
    released: '2004-01-30',
    developers: ['TYPE-MOON'],
  };

  // 中文优先、无中文回落原文；元数据落地后把 SAMPLE_META 换成 game 即可，本段不动
  let title = $derived(SAMPLE_META.title_cn || SAMPLE_META.title);
  let alias = $derived([SAMPLE_META.title_jp, SAMPLE_META.title].filter(Boolean).join(' / '));

  // VNDB 的 rating 是百分制，界面按十分制展示
  let ratingScore = $derived(SAMPLE_META.rating ? (SAMPLE_META.rating / 10).toFixed(2) : '');
  let releaseYear = $derived(SAMPLE_META.released?.slice(0, 4) ?? '');
  let developer = $derived(SAMPLE_META.developers?.[0] ?? '');

  $effect(() => {
    loading = true;
    failed = false;
    game = null;

    let live = true;
    api
      .getGame(gameId)
      .then((g) => {
        if (!live) return;
        game = g;
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
    <div class="detail-ambience" aria-hidden="true">
      <img src={SAMPLE_COVER} alt="" />
    </div>
    <div class="detail-banner">
      <div class="detail-cover">
        <img src={SAMPLE_COVER} alt={game.name} />
      </div>
      <div class="detail-info">
        <h1>{title}</h1>
        {#if alias}
          <p class="detail-alias">{alias}</p>
        {/if}
        <div class="detail-info-line">
          <span class="info-left">
            <span class="info-rating-num">★ {ratingScore}</span>
            <span class="info-rating-sub">{t('detail.voteCount', { count: SAMPLE_META.votecount })}</span>
          </span>
          <span class="info-tags">
            {#if releaseYear}
              <span class="info-pill">{releaseYear}</span>
            {/if}
            {#if developer}
              <span class="info-pill">{developer}</span>
            {/if}
          </span>
        </div>
      </div>
    </div>
  {/if}
</section>