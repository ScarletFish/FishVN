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
    // 原样取自 vndb-sample/vn.json；真实数据可能含 [url] 这类 BBCode，需清洗后渲染
    description: `In the bustling city of Fuyuki, a secret battle rages in the shadows.
The Holy Grail War, a brutal battle royale, pits seven mages, known as Masters, against each other. Each Master summons a formidable warrior from various eras as their Servant, all vying for the ultimate prize: the Holy Grail, a wish-granting artifact.

Emiya Shirou, an ordinary high school student driven by an unwavering sense of justice, finds himself unwillingly pulled into this deadly conflict. One evening, Shirou witnesses a fierce battle between Servants. The clang of steel and the crackle of magic fill the air as they clash. Mortally wounded in the chaos, Shirou is saved by the enigmatic Tohsaka Rin and her Servant Archer. In a desperate attempt to protect himself, Shirou accidentally summons his own Servant, Saber—a noble and beautiful knight with a mysterious past.

As Shirou and Saber navigate the dangerous Holy Grail War, they form alliances and face numerous adversaries, each with their own ambitions and secrets. The bond between Master and Servant deepens as they confront perilous challenges and uncover hidden truths.`,
  };

  // 临时：字段形态对齐 VNDB Kana 内嵌的 vn[].tags（Tag 全字段），后端元数据落地后删掉本常量。
  // category 不用来分区：cont 占 57%，分了会失控。
  const SAMPLE_TAGS = [
    { id: 'g1', name: 'Masterpiece', category: 'cont', rating: 3, spoiler: 0, lie: false, vn_count: 320 },
    { id: 'g2', name: 'Award Winning', category: 'cont', rating: 2.6, spoiler: 0, lie: false, vn_count: 480 },
    { id: 'g3', name: 'Classic', category: 'cont', rating: 2.8, spoiler: 0, lie: false, vn_count: 960 },
    { id: 'g4', name: 'True Ending', category: 'cont', rating: 2.6, spoiler: 0, lie: false, vn_count: 540 },
    { id: 'g5', name: 'Multiple Endings', category: 'tech', rating: 2.2, spoiler: 0, lie: false, vn_count: 6210 },
    { id: 'g6', name: 'Fate', category: 'cont', rating: 2.4, spoiler: 0, lie: false, vn_count: 640 },
    { id: 'g7', name: 'Tragedy', category: 'cont', rating: 2.2, spoiler: 0, lie: false, vn_count: 1740 },
    { id: 'g8', name: 'Good Ending', category: 'cont', rating: 2.2, spoiler: 0, lie: false, vn_count: 1980 },
    { id: 'g9', name: 'Ensemble Cast', category: 'cont', rating: 2.5, spoiler: 0, lie: false, vn_count: 2640 },
    { id: 'g10', name: 'ADV', category: 'tech', rating: 2, spoiler: 0, lie: false, vn_count: 36032 },
    { id: 'g11', name: 'Visual Novel', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 30210 },
    { id: 'g12', name: 'Japan', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 20480 },
    { id: 'g13', name: 'Female Protagonist', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 19870 },
    { id: 'g14', name: 'Drama', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 15320 },
    { id: 'g15', name: 'Primarily 2D Art', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 14260 },
    { id: 'g16', name: 'Choice', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1420 },
    { id: 'g17', name: 'Reviewed', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 7420 },
    { id: 'g18', name: 'Male Protagonist', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 12470 },
    { id: 'g19', name: 'School Life', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 10240 },
    { id: 'g20', name: 'Romance', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 9840 },
    { id: 'g21', name: 'Fantasy', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 8940 },
    { id: 'g22', name: 'Slice of Life', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 8760 },
    { id: 'g23', name: 'Magic', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 9140 },
    { id: 'g24', name: 'Supernatural', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 7120 },
    { id: 'g25', name: 'Comedy', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 6320 },
    { id: 'g26', name: 'Action', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 5620 },
    { id: 'g27', name: 'Multiple Heroines', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 5240 },
    { id: 'g28', name: 'Mystery', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 4380 },
    { id: 'g29', name: 'Combat', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 4310 },
    { id: 'g30', name: 'Seinen', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 4180 },
    { id: 'g31', name: 'Parody', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 4180 },
    { id: 'g32', name: 'Magic User', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 3870 },
    { id: 'g33', name: 'Battles', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 3620 },
    { id: 'g34', name: 'Harem', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 3260 },
    { id: 'g35', name: 'Heroine Focus', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 2980 },
    { id: 'g36', name: 'Tragedy Drama', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 2860 },
    { id: 'g37', name: 'Heroic Fantasy', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 2280 },
    { id: 'g38', name: 'Amnesia', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 2140 },
    { id: 'g39', name: 'Bad Ending', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1640 },
    { id: 'g40', name: 'Love Polygon', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1320 },
    { id: 'g41', name: 'Memory Loss', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1420 },
    { id: 'g42', name: 'Superhuman', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1420 },
    { id: 'g43', name: 'England', category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1180 },
    { id: 'g44', name: 'Japanese History', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 1120 },
    { id: 'g45', name: "Protagonist's Female Friend", category: 'cont', rating: 2, spoiler: 0, lie: false, vn_count: 1180 },
    { id: 'g46', name: 'Sitcom', category: 'cont', rating: 1.8, spoiler: 0, lie: false, vn_count: 960 },
    { id: 'g47', name: 'Trauma', category: 'cont', rating: 2, spoiler: 1, lie: false, vn_count: 860 },
    { id: 'g48', name: 'Mind Control', category: 'cont', rating: 2, spoiler: 1, lie: false, vn_count: 620 },
  ];

  const TAGS_PREVIEW = 8;
  let tagsExpanded = $state(false);

  let summaryParas = $derived(SAMPLE_META.description.split('\n\n').filter(Boolean));

  // 中文优先、无中文回落原文；元数据落地后把 SAMPLE_META 换成 game 即可，本段不动
  let title = $derived(SAMPLE_META.title_cn || SAMPLE_META.title);
  let alias = $derived([SAMPLE_META.title_jp, SAMPLE_META.title].filter(Boolean).join(' / '));

  // VNDB 的 rating 是百分制，界面按十分制展示
  let ratingScore = $derived(SAMPLE_META.rating ? (SAMPLE_META.rating / 10).toFixed(2) : '');
  let releaseYear = $derived(SAMPLE_META.released?.slice(0, 4) ?? '');
  let developer = $derived(SAMPLE_META.developers?.[0] ?? '');

  // spoiler ≥ 1 的不展示；rating 集中在一个值上，同值再按 vn_count 排
  let tags = $derived(
    SAMPLE_TAGS.filter((tag) => !tag.spoiler)
      .sort((a, b) => b.rating - a.rating || b.vn_count - a.vn_count)
  );
  let shownTags = $derived(tagsExpanded ? tags : tags.slice(0, TAGS_PREVIEW));
  let tagsRemaining = $derived(tags.length - TAGS_PREVIEW);

  $effect(() => {
    loading = true;
    failed = false;
    game = null;
    tagsExpanded = false;

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
              <span class="pill pill--info">{releaseYear}</span>
            {/if}
            {#if developer}
              <span class="pill pill--info">{developer}</span>
            {/if}
          </span>
        </div>
        <div class="detail-tags">
          <div class="detail-tags-list">
            {#each shownTags as tag (tag.id)}
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
  {/if}
</section>