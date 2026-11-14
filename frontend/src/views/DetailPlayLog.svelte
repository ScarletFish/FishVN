<script>
  import { onMount } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { compactDuration, durationText } from '../lib/playtime.js';
  import { averageSeconds, buildCumulativeSeries, monthDay, shortDate } from '../lib/play-log.js';
  import { getTimeMetric } from '../lib/config-store.svelte.js';
  import { LIBRARY_CHANGED, onLibraryItemUpdated, onPlaySessionStarted, onPlaySessionEnded } from '../lib/library-events.js';
  import DetailSection from '../components/DetailSection.svelte';
  import VisXYContainer from '@unovis/svelte/containers/xy-container';
  import VisArea from '@unovis/svelte/components/area';
  import VisLine from '@unovis/svelte/components/line';
  import VisAxis from '@unovis/svelte/components/axis';
  import VisScatter from '@unovis/svelte/components/scatter';
  import VisXYLabels from '@unovis/svelte/components/xy-labels';
  import VisCrosshair from '@unovis/svelte/components/crosshair';
  import VisTooltip from '@unovis/svelte/components/tooltip';

  let { gameId } = $props();

  let log = $state(null);
  let root = $state(null);

  const STEP = 'stepAfter';

  async function load(live) {
    try {
      const next = await api.gamePlayLog(gameId);
      if (live() && next) log = next;
    } catch {}
  }

  async function reload() {
    try {
      const next = await api.gamePlayLog(gameId);
      if (next) log = next;
    } catch {}
  }

  $effect(() => {
    let live = true;
    log = null;
    load(() => live);
    return () => {
      live = false;
    };
  });

  onMount(() => {
    const unwatch = onLibraryItemUpdated((id) => {
      if (id === gameId) reload();
    });
    window.addEventListener(LIBRARY_CHANGED, reload);
    const offStart = onPlaySessionStarted((id) => {
      if (id === gameId) reload();
    });
    const offEnd = onPlaySessionEnded((id) => {
      if (id === gameId) reload();
    });
    return () => {
      unwatch();
      window.removeEventListener(LIBRARY_CHANGED, reload);
      offStart();
      offEnd();
    };
  });

  const aliveMode = $derived(getTimeMetric() === 'alive');
  const average = $derived(averageSeconds(log?.total_seconds, log?.count));
  const series = $derived(buildCumulativeSeries(log?.sessions ?? [], aliveMode ? 'alive' : 'foreground'));
  const points = $derived(
    series.points.map((p, i, list) => ({ ...p, isLast: i === list.length - 1 })),
  );

  const span = $derived(Math.max((series.last ?? 0) - (series.first ?? 0), 86400));
  const xDomain = $derived.by(() => {
    const first = series.first;
    if (first == null) return [0, 1];
    const last = series.last ?? first;
    if (last <= first) return [first - 43200, first + 43200];
    return [first, last];
  });

  const axis = $derived.by(() => {
    const raw = Math.max(0, series.total);
    if (raw <= 0) return { max: 60, values: [0, 60] };
    const unit = raw >= 3600 ? 3600 : 60;
    const total = raw / unit;
    const rough = total / 4;
    const magnitude = 10 ** Math.floor(Math.log10(rough));
    const normalized = rough / magnitude;
    const step = (normalized <= 1 ? 1 : normalized <= 2 ? 2 : normalized <= 5 ? 5 : 10) * magnitude;
    let max = Math.ceil(total / step) * step;
    if (max <= total) max += step;
    const values = [];
    for (let value = 0; value <= max + step / 1000; value += step) values.push(value * unit);
    return { max: max * unit, values };
  });

  const xTicks = $derived.by(() => {
    if (series.first == null) return [];
    const count = 4;
    const out = [];
    for (let index = 0; index < count; index += 1) {
      out.push(series.first + (span * index) / (count - 1));
    }
    return out;
  });

  function axisText(value) {
    if (value <= 0) return '0';
    const minutes = Math.round(value / 60);
    if (minutes >= 60 && minutes % 60 === 0) return `${minutes / 60}h`;
    return compactDuration(minutes);
  }

  function markLabel(at) {
    if (span >= 730 * 86400) return shortDate(at).slice(0, 4);
    if (span >= 60 * 86400) return shortDate(at).slice(0, 7);
    return monthDay(at);
  }

  const tickAlign = (value, index, values) =>
    index === 0 ? 'left' : index === values.length - 1 ? 'right' : 'center';

  const x = (d) => d.at;
  const y = (d) => d.total;
  const endLabel = $derived(compactDuration(Math.round(series.total / 60)));
  const endLabelY = (d) => (d.isLast ? d.total + axis.max * 0.07 : null);
  const endDotY = (d) => (d.isLast ? d.total : null);

  const peak = $derived.by(() => {
    let best = null;
    for (const point of series.points) {
      if (!best || point.step > best.step) best = point;
    }
    return best;
  });

  const longest = $derived(peak?.step ?? 0);
  const metricText = $derived(aliveMode ? t('settings.timeAlive') : t('settings.timeForeground'));

  const tooltipTemplate = (d) => {
    if (!d) return '';
    const step = compactDuration(Math.round(d.step / 60));
    const total = compactDuration(Math.round(d.total / 60));
    return `<div class="playlog-tip"><span class="playlog-tip-date">${shortDate(d.at)}</span><b class="playlog-tip-step">${step}</b><span class="playlog-tip-total">${t('detail.playLogCumulative')} ${total}</span></div>`;
  };

  export function scrollTo() {
    root?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
</script>

{#snippet metricBadge()}
  <span class="detail-playlog-metric">{metricText}</span>
{/snippet}

{#if log?.count}
  <div class="detail-playlog-anchor" bind:this={root}>
    <DetailSection
      title={t('detail.playLog')}
      count={log.count}
      action={metricBadge}
      class="detail-enter-playlog"
    >
      <div class="detail-playlog-meta">
        <span
          >{t('detail.playLogFirst')}
          <b>{shortDate(log.first_at)}</b></span
        >
        <span
          >{t('detail.playLogLast')}
          <b>{shortDate(log.last_at)}</b></span
        >
        <span
          >{t('detail.playLogAverage')}
          <b>{durationText(t, average)}</b></span
        >
        <span
          >{t('detail.playLogLongest')}
          <b>{durationText(t, longest)}</b></span
        >
      </div>

      <div class="detail-playlog-chart">
        <VisXYContainer
          data={points}
          height={292}
          margin={{ top: 18, right: 36, bottom: 28, left: 56 }}
          xDomain={xDomain}
          yDomain={[0, axis.max]}
          ariaLabel={t('detail.playLog')}
        >
          <VisArea {x} {y} color="var(--chart-fill)" curveType={STEP} />
          <VisLine {x} {y} color="var(--chart-main)" lineWidth={2} curveType={STEP} />
          <VisAxis
            type="x"
            tickValues={xTicks}
            tickFormat={markLabel}
            tickTextAlign={tickAlign}
            gridLine={false}
            tickLine={false}
            domainLine={false}
          />
          <VisAxis
            type="y"
            tickValues={axis.values}
            tickFormat={axisText}
            tickLine={false}
            domainLine={false}
          />
          <VisScatter {x} y={endDotY} size={6} color="var(--accent)" />
          <VisXYLabels
            {x}
            y={endLabelY}
            label={endLabel}
            color="var(--fg-secondary)"
            backgroundColor="transparent"
            clustering={false}
          />
          <VisCrosshair {x} {y} color="var(--accent)" template={tooltipTemplate} />
          <VisTooltip />
        </VisXYContainer>
      </div>
    </DetailSection>
  </div>
{/if}
