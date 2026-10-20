<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';

  const H = 230;
  const LEFT = 8;
  const RIGHT = 8;
  const TOP = 26;
  const BASE = 192;
  const LABEL_Y = 214;

  const KEYS = ['pre2000', 'y2000', 'y2010', 'y2020'];

  const stat = useStat(api.dashboardReleaseEras);
  const points = $derived(stat.data ?? []);
  let chartW = $state(0);

  const total = $derived(points.reduce((sum, p) => sum + p.count, 0));
  const max = $derived(points.reduce((m, p) => Math.max(m, p.count), 0));
  const W = $derived(chartW > 0 ? chartW : 520);
  const band = $derived((W - LEFT - RIGHT) / KEYS.length);
  const barW = 44;

  const x = (i) => LEFT + i * band + (band - barW) / 2;
  const y = (count) => BASE - (count / Math.max(1, max)) * (BASE - TOP);

</script>

<StatCard
  title={t('dashboard.era.title')}
  sub={t('dashboard.era.sub')}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.era.empty')}
>
  <svg
    class="era-chart"
    viewBox="0 0 {W} {H}"
    height={H}
    bind:clientWidth={chartW}
    role="img"
    aria-label={t('dashboard.era.title')}
  >
        <line class="era-grid" x1={LEFT} x2={W - RIGHT} y1={BASE} y2={BASE} />

        {#each points as p, i (p.key)}
          {#if p.count > 0}
            <rect
              x={x(i)}
              y={y(p.count)}
              width={barW}
              height={Math.max(2, BASE - y(p.count))}
              rx="4"
              style="fill: var(--accent)"
            />
            <text class="era-value" x={x(i) + barW / 2} y={y(p.count) - 6} text-anchor="middle">
              {p.count}
            </text>
          {/if}
          <text class="era-label" x={x(i) + barW / 2} y={LABEL_Y} text-anchor="middle">
            {t(`dashboard.era.${p.key}`)}
          </text>
        {/each}
  </svg>
</StatCard>

<style>
  .era-chart {
    display: block;
    width: 100%;
  }

  .era-grid {
    stroke: var(--border-default);
    stroke-width: 1;
  }

  .era-value {
    fill: var(--fg-secondary);
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
  }

  .era-label {
    fill: var(--fg-muted);
    font-size: var(--text-xs);
  }
</style>
