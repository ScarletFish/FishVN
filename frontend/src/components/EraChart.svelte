<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { useStat } from '../lib/stat-loader.svelte.js';
  import Seg from './Seg.svelte';
  import StatCard from './StatCard.svelte';
  import VisXYContainer from '@unovis/svelte/containers/xy-container';
  import VisGroupedBar from '@unovis/svelte/components/grouped-bar';
  import VisAxis from '@unovis/svelte/components/axis';
  import VisXYLabels from '@unovis/svelte/components/xy-labels';

  const STATUS_LABEL = {
    playing: 'dashboard.statPlaying',
    finished: 'dashboard.statFinished',
    stalled: 'dashboard.statStalled',
    dropped: 'dashboard.statDropped',
    wishlist: 'dashboard.statWishlist',
  };

  let scope = $state('all');

  const stat = useStat(
    () => api.dashboardReleaseEras(scope === 'all' ? null : scope),
    () => scope
  );

  const points = $derived((stat.data ?? []).map((p, i) => ({ ...p, i })));
  const total = $derived(points.reduce((sum, p) => sum + p.count, 0));
  const max = $derived(points.reduce((m, p) => Math.max(m, p.count), 0));
  const headroom = $derived(Math.max(1, max) * 1.15);

  const scopeOptions = $derived([
    { value: 'all', label: t('dashboard.era.all') },
    ...Object.entries(STATUS_LABEL).map(([value, key]) => ({ value, label: t(key) })),
  ]);

  const x = (d) => d.i;
  const y = (d) => d.count;
  const labelY = (d) => d.count + Math.max(1, max) * 0.055;
  const valueLabel = (d) => String(d.count);
  const eraText = (v) => t(`dashboard.era.${points[Math.round(v)]?.key}`);
</script>

{#snippet scopeFilter()}
  <Seg bind:value={scope} options={scopeOptions} label={t('dashboard.era.scope')} />
{/snippet}

<StatCard
  title={t('dashboard.era.title')}
  actions={scopeFilter}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.era.empty')}
>
  <div class="era-plot">
    <VisXYContainer
      data={points}
      height={186}
      margin={{ top: 16, right: 8, bottom: 22, left: 8 }}
      xDomain={[-0.5, Math.max(0, points.length - 0.5)]}
      yDomain={[0, headroom]}
      ariaLabel={t('dashboard.era.title')}
    >
      <VisGroupedBar
        {x}
        {y}
        color="var(--chart-main)"
        roundedCorners={4}
        groupMaxWidth={112}
      />
      <VisAxis
        type="x"
        tickValues={points.map((p) => p.i)}
        tickFormat={eraText}
        gridLine={false}
        tickLine={false}
      />
      <VisXYLabels
        {x}
        y={labelY}
        label={valueLabel}
        color="var(--fg-default)"
        backgroundColor="transparent"
        labelFontSize={12}
      />
    </VisXYContainer>
  </div>

  <p class="era-sum">{t('dashboard.era.sum', { n: total })}</p>
</StatCard>

<style>
  .era-plot {
    --vis-color0: var(--chart-main);
    --vis-font-family: var(--font-sans);
    --vis-axis-tick-color: var(--fg-muted);
    --vis-axis-domain-color: var(--border-default);
    --vis-axis-tick-label-color: var(--fg-muted);
    --vis-axis-tick-label-font-size: var(--text-xs);
    height: 186px;
    min-width: 0;
  }

  .era-sum {
    margin: var(--space-3) 0 0;
    color: var(--fg-muted);
    font-size: var(--text-xs);
  }
</style>
