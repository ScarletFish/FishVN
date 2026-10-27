<script>
  import { onMount } from 'svelte';
  import { api } from '../lib/api.js';
  import { tagLabel } from '../lib/tag-label.svelte.js';
  import { t } from '../lib/i18n.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import { CONFIG_CHANGED } from '../lib/config-events.js';
  import { LIBRARY_TAGS_CHANGED } from '../lib/library-events.js';
  import StatCard from './StatCard.svelte';

  const MIN_SIZE = 0.85;
  const MAX_SIZE = 1.25;
  const GOLDEN = Math.PI * (3 - Math.sqrt(5));

  const stat = useStat(api.dashboardTagCloud);
  const tags = $derived(stat.data?.tags ?? []);
  const candidates = $derived(stat.data?.candidates ?? 0);
  const excluded = $derived(stat.data?.excluded ?? 0);
  let boxW = $state(0);
  let boxH = $state(0);

  onMount(() => {
    const refresh = () => stat.refresh();
    window.addEventListener(CONFIG_CHANGED, refresh);
    window.addEventListener(LIBRARY_TAGS_CHANGED, refresh);
    return () => {
      window.removeEventListener(CONFIG_CHANGED, refresh);
      window.removeEventListener(LIBRARY_TAGS_CHANGED, refresh);
    };
  });

  const ranked = $derived.by(() => {
    if (tags.length === 0) return [];
    const counts = tags.map((x) => x.count);
    const max = Math.max(...counts);
    const min = Math.min(...counts);
    const span = Math.max(1, max - min);
    return tags.map((x) => {
      const ratio = (x.count - min) / span;
      return {
        id: x.id,
        label: tagLabel({ id: x.id, name: x.tag }),
        count: x.count,
        size: MIN_SIZE + ratio * (MAX_SIZE - MIN_SIZE),
        weight: Math.round(45 + ratio * 55),
        bold: ratio >= 0.6,
      };
    });
  });

  const cloud = $derived(layoutCloud(ranked, boxW, boxH));

  function searchTag(word) {
    if (!word.label) return;
    document.dispatchEvent(new CustomEvent('search-set-query', { detail: word.label }));
  }

  function layoutCloud(words, W, H) {
    if (words.length === 0 || W <= 0 || H <= 0) return [];
    const ctx = document.createElement('canvas').getContext('2d');
    const sized = words.map((w) => {
      const px = w.size * 16;
      ctx.font = `${w.bold ? 600 : 400} ${px}px "Noto Sans SC", sans-serif`;
      const tw = ctx.measureText(w.label).width;
      return { ...w, px, tw, th: px * 1.35 };
    });
    const area = (w) => (w.tw + 6) * w.th;
    const total = sized.reduce((sum, w) => sum + area(w), 0) * 1.45;
    const Rx = W / 2;
    const Ry = H / 2;
    const placed = [];
    const out = [];
    let acc = 0;
    sized.forEach((w, i) => {
      let r = Math.sqrt(acc / total);
      acc += area(w);
      let a = i * GOLDEN + 0.6;
      for (let step = 0; step < 320; step++) {
        const x = Math.cos(a) * r * Rx;
        const y = Math.sin(a) * r * Ry;
        if (Math.abs(x) + w.tw / 2 <= Rx - 3 && Math.abs(y) + w.th / 2 <= Ry - 2) {
          const box = { x0: x - w.tw / 2, x1: x + w.tw / 2, y0: y - w.th / 2, y1: y + w.th / 2 };
          const hit = placed.some((p) => box.x0 < p.x1 && box.x1 > p.x0 && box.y0 < p.y1 && box.y1 > p.y0);
          if (!hit) {
            placed.push(box);
            out.push({ id: w.id, label: w.label, count: w.count, px: w.px, bold: w.bold, weight: w.weight, x, y });
            return;
          }
        }
        r += 0.028;
        a += 0.4;
      }
    });
    return out;
  }
</script>

<StatCard
  title={t('dashboard.tagCloud.title')}
  sub={candidates > 0
    ? excluded > 0
      ? t('dashboard.tagCloud.subExcluded', { count: candidates, excluded })
      : t('dashboard.tagCloud.sub', { count: candidates })
    : ''}
  loading={stat.loading}
  error={stat.error}
  empty={tags.length === 0 && excluded === 0}
  emptyText={excluded > 0 ? t('dashboard.tagCloud.allExcluded') : t('dashboard.tagCloud.empty')}
>
  <div class="tag-cloud-body" bind:clientWidth={boxW} bind:clientHeight={boxH}>
    {#each cloud as word (word.id ?? word.label)}
      <span
        class="tag-cloud-word"
        style:transform="translate(-50%, -50%) translate({word.x.toFixed(1)}px, {word.y.toFixed(1)}px)"
        style:font-size="{word.px}px"
      >
        <button
          type="button"
          class="tag-cloud-word-inner"
          style:font-weight={word.bold ? 'var(--fw-semibold)' : 'var(--fw-normal)'}
          style:color="color-mix(in srgb, var(--accent) {word.weight}%, var(--fg-muted))"
          onclick={() => searchTag(word)}
        >{word.label}</button>
      </span>
    {/each}
  </div>
</StatCard>

<style>
  .tag-cloud-body {
    flex: 1;
    min-height: 0;
  }

  .tag-cloud-word {
    position: absolute;
    left: 50%;
    top: 50%;
    white-space: nowrap;
  }

  .tag-cloud-word:hover {
    z-index: 1;
  }

  .tag-cloud-word-inner {
    display: inline-block;
    padding: 0;
    border: 0;
    background: none;
    cursor: pointer;
    font-size: 100%;
    transition: font-size var(--duration-fast) var(--ease-out);
  }

  .tag-cloud-word-inner:hover {
    font-size: 108%;
  }
</style>
