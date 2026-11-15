<script>
  import { api } from '../../lib/api.js';
  import { t } from '../../lib/i18n.js';
  import { useStat } from '../../lib/stat-loader.svelte.js';
  import Seg from '../Seg.svelte';
  import StatCard from './StatCard.svelte';
  import VisXYContainer from '@unovis/svelte/containers/xy-container';
  import VisArea from '@unovis/svelte/components/area';
  import VisLine from '@unovis/svelte/components/line';
  import VisAxis from '@unovis/svelte/components/axis';
  import VisScatter from '@unovis/svelte/components/scatter';
  import VisTooltip from '@unovis/svelte/components/tooltip';
  import { Scatter } from '@unovis/ts/components/scatter';

  const GRANULARITIES = ['week', 'month', 'year'];
  const BUCKETS = 12;

  let granularity = $state('month');
  const stat = useStat(
    () => api.dashboardActivity(granularity),
    () => granularity
  );

  const shortUnit = $derived(t(`dashboard.activity.granularity.${granularity}`));
  const longUnit = $derived(t(`dashboard.activity.unit.${granularity}`));

  const granularityOptions = $derived(
    GRANULARITIES.map((value) => ({
      value,
      label: t(`dashboard.activity.granularity.${value}`),
    }))
  );

  const bucketLabel = (list, p, i) => {
    const year = p.bucket.slice(0, 4);
    const newYear = i === 0 || year !== list[i - 1].bucket.slice(0, 4);
    if (granularity === 'year') return year;
    if (granularity === 'month') {
      const label = `${Number(p.bucket.slice(5, 7))}月`;
      return newYear ? `${year}年${label}` : label;
    }
    return `${Number(p.bucket.slice(5, 7))}/${Number(p.bucket.slice(8, 10))}`;
  };

  const points = $derived(
    (stat.data ?? []).map((p, i, list) => ({ ...p, i, label: bucketLabel(list, p, i) }))
  );

  const total = $derived(points.reduce((sum, p) => sum + p.seconds, 0));
  const max = $derived(points.reduce((m, p) => Math.max(m, p.seconds), 0));
  const peak = $derived(
    points.reduce(
      (best, p) => (p.seconds > best.seconds ? p : best),
      { seconds: 0, bucket: '', label: '', i: -1 }
    )
  );
  const avg = $derived(points.length ? Math.round(total / points.length) : 0);

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

  const axisText = (seconds) => {
    const s = Math.round(seconds);
    if (s < 60) return `${s}s`;
    const minutes = Math.round(s / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const rest = minutes % 60;
    return rest === 0 ? `${hours}h` : `${hours}h${rest}m`;
  };

  const x = (d) => d.i;
  const y = (d) => d.seconds;
  const pointColor = (d) => (d.i === peak.i ? 'var(--accent)' : 'var(--chart-main)');
  const peakLabel = (d) => (d.i === peak.i && d.seconds > 0 ? axisText(d.seconds) : '');
  const xTickFormat = (v) => points[Math.round(v)]?.label ?? '';
  const tooltipTriggers = { [Scatter.selectors.point]: (d) => (d ? axisText(d.seconds) : '') };
</script>

{#snippet granularityFilter()}
  <Seg bind:value={granularity} options={granularityOptions} label={t('dashboard.activity.scope')} />
{/snippet}

<StatCard
  title={t('dashboard.activity.title')}
  actions={granularityFilter}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.activity.empty')}
>
  <div class="activity-layout">
    <div class="activity-plot">
      <VisXYContainer
        data={points}
        height={220}
        margin={{ top: 8, right: 4, bottom: 0, left: 0 }}
        xDomain={[0, Math.max(1, points.length - 1)]}
        yDomain={[0, ticks.niceMax]}
        ariaLabel={t('dashboard.activity.title')}
      >
        <VisArea {x} {y} color="var(--chart-fill)" />
        <VisLine {x} {y} color="var(--chart-main)" />
        <VisScatter
          {x}
          {y}
          size={5}
          color={pointColor}
          label={peakLabel}
          labelColor="var(--accent)"
          labelPosition="top"
        />
        <VisAxis
          type="x"
          tickValues={points.map((p) => p.i)}
          tickFormat={xTickFormat}
          gridLine={false}
          tickLine={false}
          domainLine={false}
        />
        <VisAxis
          type="y"
          tickValues={ticks.values}
          tickFormat={axisText}
          label={t('dashboard.axis.duration')}
          tickLine={false}
          domainLine={false}
        />
        <VisTooltip triggers={tooltipTriggers} />
      </VisXYContainer>
    </div>

    <div class="activity-summary">
      <div class="activity-metric">
        <span class="activity-metric-label">{t('dashboard.activity.total', { n: BUCKETS, unit: longUnit })}</span>
        <b class="activity-metric-value">{axisText(total)}</b>
      </div>
      <div class="activity-metric">
        <span class="activity-metric-label">{t('dashboard.activity.avg', { unit: shortUnit })}</span>
        <b class="activity-metric-value">{axisText(avg)}</b>
      </div>
    </div>
  </div>
</StatCard>

<style>
  .activity-layout {
    display: grid;
    grid-template-columns: minmax(0, 2.4fr) minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-height: 0;
  }

  .activity-plot {
    --vis-color0: var(--chart-main);
    --vis-font-family: var(--font-sans);
    --vis-axis-tick-color: var(--fg-muted);
    --vis-axis-grid-color: var(--border-default);
    --vis-axis-domain-color: var(--border-default);
    --vis-axis-tick-label-color: var(--fg-muted);
    --vis-axis-tick-label-font-size: var(--text-xs);
    --vis-axis-label-color: var(--fg-muted);
    --vis-axis-label-font-size: var(--text-xs);
    --vis-scatter-point-label-text-font-size: var(--text-xs);
    --vis-scatter-stroke-color: transparent;
    --vis-scatter-stroke-width: 10px;
    --vis-scatter-hover-stroke-width: 10px;
    --vis-tooltip-padding: var(--space-2) var(--space-3);
    --vis-tooltip-border-radius: var(--radius-md);
    --vis-tooltip-transition-duration: 150ms;
    --vis-tooltip-background-color: var(--bg-surface);
    --vis-tooltip-text-color: var(--fg-default);
    --vis-tooltip-border-color: var(--border-default);
    height: 220px;
    min-width: 0;
  }

  .activity-summary {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: var(--space-4);
    min-width: 0;
    height: 220px;
    padding-left: var(--space-3);
  }

  .activity-summary::before {
    content: '';
    position: absolute;
    left: 0;
    top: 12px;
    bottom: 24px;
    width: 1px;
    background: var(--border-default);
  }

  .activity-metric {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .activity-metric-label {
    color: var(--fg-muted);
    font-size: var(--text-xs);
  }

  .activity-metric-value {
    color: var(--fg-default);
    font-size: var(--text-base);
    font-weight: var(--fw-semibold);
    letter-spacing: -0.01em;
  }
</style>
