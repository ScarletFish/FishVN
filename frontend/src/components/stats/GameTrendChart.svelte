<script>
  import { api } from '../../lib/api.js';
  import { t } from '../../lib/i18n.js';
  import { compactDuration } from '../../lib/playtime.js';
  import { useStat } from '../../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';
  import VisXYContainer from '@unovis/svelte/containers/xy-container';
  import VisLine from '@unovis/svelte/components/line';
  import VisAxis from '@unovis/svelte/components/axis';

  const PLOT_HEIGHT = 340;

  const stat = useStat(api.dashboardGameTrend);
  const buckets = $derived(stat.data?.buckets ?? []);
  const series = $derived(stat.data?.series ?? []);
  const total = $derived(series.reduce((sum, s) => sum + s.total, 0));

  const rows = $derived(
    buckets.map((_, i) => ({ i, values: series.map((s) => s.values[i] ?? 0) }))
  );
  const yAccessors = $derived(series.map((_, k) => (d) => d.values[k]));
  const colors = $derived(series.map((_, i) => `var(--chart-cat-${i + 1})`));

  const max = $derived(rows.reduce((m, r) => Math.max(m, ...r.values), 0));
  const ticks = $derived.by(() => {
    if (max <= 0) return { niceMax: 1, values: [0, 1] };
    const rough = max / 4;
    const mag = 10 ** Math.floor(Math.log10(rough));
    const norm = rough / mag;
    const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
    const niceMax = Math.ceil(max / step) * step;
    const values = [];
    for (let v = 0; v <= niceMax + step / 1000; v += step) values.push(v);
    return { niceMax, values };
  });

  const x = (d) => d.i;
  const axisText = (seconds) =>
    seconds <= 0 ? '0' : compactDuration(Math.round(seconds / 60));

  const bucketLabel = (i) => {
    const bucket = buckets[i];
    if (!bucket) return '';
    const year = bucket.slice(0, 4);
    const month = Number(bucket.slice(5, 7));
    const newYear = i === 0 || year !== buckets[i - 1].slice(0, 4);
    return newYear ? `${year}年${month}月` : `${month}月`;
  };
  const xTickFormat = (v) => bucketLabel(Math.round(v));
</script>

<StatCard
  title={t('dashboard.trend.title')}
  sub={total > 0 ? t('dashboard.trend.sub', { n: series.length }) : ''}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.trend.empty')}
>
  <div class="trend-legend">
    {#each series as item, i (item.game_id)}
      <span class="trend-legend-item">
        <i class="trend-swatch" style:background={colors[i]}></i>
        <span class="trend-name">{item.name}</span>
      </span>
    {/each}
  </div>

  <div class="trend-plot">
    <VisXYContainer
      data={rows}
      height={PLOT_HEIGHT}
      margin={{ top: 12, right: 16, bottom: 26, left: 52 }}
      xDomain={[0, Math.max(1, rows.length - 1)]}
      yDomain={[0, ticks.niceMax]}
      ariaLabel={t('dashboard.trend.title')}
    >
      <VisLine
        {x}
        y={yAccessors}
        color={colors}
        lineWidth={2}
        highlightOnHover
      />
      <VisAxis
        type="x"
        tickValues={rows.map((r) => r.i)}
        tickFormat={xTickFormat}
        gridLine={false}
        tickLine={false}
        domainLine={false}
      />
      <VisAxis
        type="y"
        tickValues={ticks.values}
        tickFormat={axisText}
        tickLine={false}
        domainLine={false}
      />
    </VisXYContainer>
  </div>
</StatCard>

<style>
  .trend-legend {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-4);
    margin-bottom: var(--space-2);
  }

  .trend-legend-item {
    display: flex;
    gap: var(--space-1);
    align-items: center;
    min-width: 0;
  }

  .trend-swatch {
    width: 8px;
    height: 8px;
    border-radius: 2px;
    flex: none;
  }

  .trend-name {
    overflow: hidden;
    max-width: 220px;
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trend-plot {
    --vis-font-family: var(--font-sans);
    --vis-axis-tick-color: var(--fg-muted);
    --vis-axis-grid-color: var(--border-default);
    --vis-axis-domain-color: var(--border-default);
    --vis-axis-tick-label-color: var(--fg-muted);
    --vis-axis-tick-label-font-size: var(--text-xs);
    flex: 1;
    min-width: 0;
    height: 340px;
  }
</style>
