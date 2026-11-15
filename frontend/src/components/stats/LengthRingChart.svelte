<script>
  import { api } from '../../lib/api.js';
  import { t } from '../../lib/i18n.js';
  import { useStat } from '../../lib/stat-loader.svelte.js';
  import StatCard from './StatCard.svelte';
  import VisSingleContainer from '@unovis/svelte/containers/single-container';
  import VisDonut from '@unovis/svelte/components/donut';

  const stat = useStat(api.dashboardLengthBuckets);
  const buckets = $derived((stat.data ?? []).map((b, i) => ({ ...b, i })));
  const total = $derived(buckets.reduce((sum, b) => sum + b.count, 0));

  const value = (d) => d.count;
  const colorOf = (i) => `var(--chart-cat-${i + 1})`;
  const color = (d) => colorOf(d.i);
  const percent = (count) => (total > 0 ? Math.round((count / total) * 100) : 0);
</script>

<StatCard
  title={t('dashboard.ring.title')}
  sub={t('dashboard.ring.sub')}
  loading={stat.loading}
  error={stat.error}
  empty={total === 0}
  emptyText={t('dashboard.ring.empty')}
>
  <div class="ring-layout">
    <div class="ring-plot">
      <VisSingleContainer data={buckets} height={200} ariaLabel={t('dashboard.ring.title')}>
        <VisDonut {value} {color} arcWidth={36} padAngle={0.02} />
      </VisSingleContainer>
      <div class="ring-center" aria-hidden="true">
        <b>{total}</b>
        <span>{t('dashboard.ring.unit')}</span>
      </div>
    </div>

    <ul class="ring-legend">
      {#each buckets as bucket (bucket.key)}
        <li class="ring-row">
          <i class="ring-swatch" style:background={colorOf(bucket.i)}></i>
          <span class="ring-name">
            <b>{t(`dashboard.ring.buckets.${bucket.key}`)}</b>{t('dashboard.ring.hours')}
          </span>
          <span class="ring-count">{bucket.count}</span>
          <span class="ring-pct"><b>{percent(bucket.count)}</b>%</span>
        </li>
      {/each}
    </ul>
  </div>
</StatCard>

<style>
  .ring-layout {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
  }

  .ring-plot {
    --vis-font-family: var(--font-sans);
    --vis-donut-background-color: var(--bg-hover);
    position: relative;
    flex: none;
    width: 200px;
    height: 200px;
  }

  .ring-center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    pointer-events: none;
  }

  .ring-center b {
    color: var(--fg-default);
    font-size: var(--text-xl);
    font-weight: var(--fw-semibold);
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }

  .ring-center span {
    color: var(--fg-muted);
    font-size: var(--text-xs);
  }

  .ring-legend {
    display: flex;
    flex: 1 1 200px;
    flex-direction: column;
    gap: var(--space-2);
    max-width: 232px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .ring-row {
    display: grid;
    grid-template-columns: 8px minmax(0, 1fr) 3ch 4ch;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
  }

  .ring-swatch {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }

  .ring-name {
    overflow: hidden;
    color: var(--fg-secondary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ring-name b {
    display: inline-block;
    width: 5.5ch;
    font-family: var(--font-mono);
    font-weight: var(--fw-normal);
    text-align: right;
  }

  .ring-count {
    color: var(--fg-default);
    font-family: var(--font-mono);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .ring-pct {
    display: flex;
    justify-content: flex-end;
    gap: 1px;
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .ring-pct b {
    min-width: 2.4ch;
    font-weight: var(--fw-normal);
    text-align: right;
  }
</style>
