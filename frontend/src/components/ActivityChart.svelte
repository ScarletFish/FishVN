<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';

  const MONTHS = 6;
  const H = 230;
  const LEFT = 40;
  const RIGHT = 16;
  const TOP = 26;
  const BASE = 192;
  const LABEL_Y = 214;

  const stat = useStat(() => api.dashboardMonthlyActivity(MONTHS));
  const points = $derived(stat.data ?? []);
  let chartW = $state(0);

  const W = $derived(chartW > 0 ? chartW : 1080);

  const total = $derived(points.reduce((sum, p) => sum + p.seconds, 0));
  const max = $derived(points.reduce((m, p) => Math.max(m, p.seconds), 0));
  const peak = $derived(
    points.reduce((best, p) => (p.seconds > best.seconds ? p : best), points[0] ?? { seconds: 0, month: '' })
  );
  const peakIndex = $derived(points.findIndex((p) => p === peak));

  const ticks = $derived.by(() => {
    const raw = Math.max(0, max);
    if (raw <= 0) return { niceMax: 1, values: [0, 1] };
    const rough = raw / 4;
    const mag = 10 ** Math.floor(Math.log10(rough));
    const norm = rough / mag;
    const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
    const niceMax = Math.ceil(raw / step) * step;
    const values = [];
    for (let v = 0; v <= niceMax + step / 1000; v += step) values.push(v);
    return { niceMax, values };
  });

  const x = (i) => LEFT + (i * (W - LEFT - RIGHT)) / Math.max(1, points.length - 1);
  const y = (v) => BASE - (v / Math.max(1, ticks.niceMax)) * (BASE - TOP);

  const line = $derived(
    points.length > 0
      ? points.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(i).toFixed(1)},${y(p.seconds).toFixed(1)}`).join(' ')
      : ''
  );

  const area = $derived(
    points.length > 0
      ? `${line} L${x(points.length - 1).toFixed(1)},${BASE} L${x(0).toFixed(1)},${BASE} Z`
      : ''
  );

  const peakX = $derived(
    peakIndex < 0 ? W / 2 : Math.min(W - RIGHT - 30, Math.max(LEFT + 30, x(peakIndex)))
  );

  const axisText = (seconds) => {
    const s = Math.round(seconds);
    if (s < 60) return `${s}s`;
    const minutes = Math.round(s / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const rest = minutes % 60;
    return rest === 0 ? `${hours}h` : `${hours}h${rest}m`;
  };

  const monthText = (p, i) => {
    const label = `${Number(p.month.slice(5, 7))}月`;
    const year = p.month.slice(0, 4);
    const prevYear = i > 0 ? points[i - 1].month.slice(0, 4) : null;
    return i === 0 || year !== prevYear ? `${year}年${label}` : label;
  };

</script>

<StatCard
  title={t('dashboard.activity.title')}
  sub={t('dashboard.activity.sub')}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.activity.empty')}
>
  <svg
    class="activity-chart"
    viewBox="0 0 {W} {H}"
    height={H}
    bind:clientWidth={chartW}
    role="img"
    aria-label={t('dashboard.activity.title')}
  >
        <defs>
          <linearGradient id="activity-fill" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" style="stop-color: var(--accent); stop-opacity: 0.28;" />
            <stop offset="100%" style="stop-color: var(--accent); stop-opacity: 0.02;" />
          </linearGradient>
        </defs>

        {#each ticks.values as tick}
          <line class="activity-grid" x1={LEFT} x2={W - RIGHT} y1={y(tick)} y2={y(tick)} />
          <text class="activity-axis" x={LEFT - 8} y={y(tick) + 3.5} text-anchor="end">
            {axisText(tick)}
          </text>
        {/each}

        <path d={area} fill="url(#activity-fill)" />
        <path
          d={line}
          fill="none"
          style="stroke: var(--accent)"
          stroke-width="2"
          stroke-linejoin="round"
          stroke-linecap="round"
        />

        {#each points as p, i}
          <circle
            cx={x(i)}
            cy={y(p.seconds)}
            r={p === peak ? 4.5 : 3}
            style:fill="var(--accent)"
            style:stroke="var(--bg-surface)"
            stroke-width="2"
          />
        {/each}

        {#if peak.seconds > 0 && peakIndex >= 0}
          <text class="activity-peak" x={peakX} y={y(peak.seconds) - 8} text-anchor="middle">
            {axisText(peak.seconds)}
          </text>
        {/if}

        {#each points as p, i}
          <text class="activity-label" x={x(i)} y={LABEL_Y} text-anchor="middle">
            {monthText(p, i)}
          </text>
        {/each}
  </svg>
</StatCard>

<style>
  .activity-chart {
    display: block;
    width: 100%;
  }

  .activity-grid {
    stroke: var(--border-default);
    stroke-width: 1;
  }

  .activity-axis {
    fill: var(--fg-muted);
    font-size: var(--text-xs);
  }

  .activity-label {
    fill: var(--fg-muted);
    font-size: var(--text-xs);
  }

  .activity-peak {
    fill: var(--fg-secondary);
    font-size: var(--text-xs);
    font-weight: var(--fw-semibold);
  }
</style>
