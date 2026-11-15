<script>
  import { api } from '../../lib/api.js';
  import { t } from '../../lib/i18n.js';
  import { compactDuration } from '../../lib/playtime.js';
  import { useStat } from '../../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';
  import VisXYContainer from '@unovis/svelte/containers/xy-container';
  import VisScatter from '@unovis/svelte/components/scatter';
  import VisAxis from '@unovis/svelte/components/axis';
  import VisTooltip from '@unovis/svelte/components/tooltip';
  import { Scatter } from '@unovis/ts/components/scatter';

  const PLOT_HEIGHT = 222;
  const MARGIN = { top: 8, right: 0, bottom: 0, left: 0 };
  const OVER_RATIO = 2;

  let plotBox = $state(null);

  const stat = useStat(api.dashboardLengthCompare);

  const points = $derived(
    (stat.data ?? []).map((p) => ({ ...p, ratio: p.expected > 0 ? p.actual / p.expected : 0 }))
  );
  const plotted = $derived(
    [...points].sort(
      (a, b) => (a.ratio >= OVER_RATIO ? 1 : 0) - (b.ratio >= OVER_RATIO ? 1 : 0)
    )
  );

  const ticks = $derived.by(() => {
    const maxHours = points.reduce((m, p) => Math.max(m, p.expected, p.actual), 0) / 3600;
    if (maxHours <= 0) return { niceMax: 3600, values: [0, 3600] };
    const rough = maxHours / 4;
    const mag = 10 ** Math.floor(Math.log10(rough));
    const norm = rough / mag;
    const step = (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
    const niceHours = Math.ceil(maxHours / step) * step;
    const values = [];
    for (let h = 0; h <= niceHours + step / 1000; h += step) values.push(Math.round(h * 3600));
    return { niceMax: Math.round(niceHours * 3600), values };
  });

  const tickText = (seconds) => {
    const minutes = Math.round(seconds / 60);
    if (minutes <= 0) return '0';
    if (minutes < 60) return `${minutes}m`;
    const hours = minutes / 60;
    return Number.isInteger(hours) ? `${hours}h` : compactDuration(minutes);
  };

  const x = (d) => d.expected;
  const y = (d) => d.actual;
  const pointColor = (d) => (d.ratio >= OVER_RATIO ? 'var(--accent)' : 'var(--chart-main)');

  const escapeHtml = (text) =>
    String(text).replace(
      /[&<>"]/g,
      (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]
    );

  const tooltipTriggers = {
    [Scatter.selectors.point]: (d) =>
      d
        ? `${escapeHtml(d.name)} · ${t('dashboard.length.expected')} ${tickText(d.expected)} · ${t('dashboard.length.actual')} ${tickText(d.actual)}`
        : '',
  };

  function onRenderComplete(svg, margin, bleed, containerWidth, containerHeight, width, height) {
    const next = {
      w: containerWidth,
      h: containerHeight,
      x0: margin.left + bleed.left,
      x1: margin.left + width - bleed.right,
      y0: margin.top + height - bleed.bottom,
      y1: margin.top + bleed.top,
    };
    const same =
      plotBox && Object.keys(next).every((k) => Math.abs(plotBox[k] - next[k]) < 0.5);
    if (!same) plotBox = next;
  }
</script>

<StatCard
  title={t('dashboard.length.title')}
  sub={t('dashboard.length.sum', { n: points.length })}
  loading={stat.loading}
  error={stat.error}
  empty={points.length === 0}
  emptyText={t('dashboard.length.empty')}
>
  <div class="length-plot">
    <VisXYContainer
      data={plotted}
      height={PLOT_HEIGHT}
      margin={MARGIN}
      xDomain={[0, ticks.niceMax]}
      yDomain={[0, ticks.niceMax]}
      ariaLabel={t('dashboard.length.title')}
      {onRenderComplete}
    >
      <VisScatter {x} {y} size={6} color={pointColor} />
      <VisAxis
        type="x"
        tickValues={ticks.values}
        tickFormat={tickText}
        label={t('dashboard.length.expected')}
        tickLine={false}
        domainLine={false}
      />
      <VisAxis
        type="y"
        tickValues={ticks.values}
        tickFormat={tickText}
        label={t('dashboard.length.actual')}
        tickLine={false}
        domainLine={false}
      />
      <VisTooltip triggers={tooltipTriggers} />
    </VisXYContainer>

    {#if plotBox}
      <svg
        class="length-reference"
        width={plotBox.w}
        height={plotBox.h}
        aria-hidden="true"
      >
        <line x1={plotBox.x0} y1={plotBox.y0} x2={plotBox.x1} y2={plotBox.y1} />
        <text x={plotBox.x1 - 4} y={plotBox.y1 + 14}>1:1</text>
      </svg>
    {/if}
  </div>
</StatCard>

<style>
  .length-plot {
    --vis-font-family: var(--font-sans);
    --vis-axis-tick-color: var(--fg-muted);
    --vis-axis-grid-color: var(--border-default);
    --vis-axis-domain-color: var(--border-default);
    --vis-axis-tick-label-color: var(--fg-muted);
    --vis-axis-tick-label-font-size: var(--text-xs);
    --vis-axis-label-color: var(--fg-muted);
    --vis-axis-label-font-size: var(--text-xs);
    --vis-scatter-stroke-color: transparent;
    --vis-scatter-stroke-width: 8px;
    --vis-scatter-hover-stroke-width: 8px;
    --vis-tooltip-padding: var(--space-2) var(--space-3);
    --vis-tooltip-border-radius: var(--radius-md);
    --vis-tooltip-transition-duration: 150ms;
    --vis-tooltip-background-color: var(--bg-surface);
    --vis-tooltip-text-color: var(--fg-default);
    --vis-tooltip-border-color: var(--border-default);
    position: relative;
    height: 222px;
    min-width: 0;
  }

  .length-reference {
    position: absolute;
    inset: 0;
    overflow: visible;
    pointer-events: none;
  }

  .length-reference line {
    stroke: var(--border-strong);
    stroke-width: 1;
    stroke-dasharray: 4 4;
  }

  .length-reference text {
    fill: var(--fg-muted);
    font-size: var(--text-xs);
    text-anchor: end;
  }
</style>
