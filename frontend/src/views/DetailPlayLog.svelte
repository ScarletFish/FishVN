<script>
  import { onMount } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { compactDuration, durationText } from '../lib/playtime.js';
  import { averageSeconds, buildCumulativeSeries, monthDay, shortDate } from '../lib/play-log.js';
  import { getTimeMetric } from '../lib/config-store.svelte.js';
  import { LIBRARY_CHANGED, onLibraryItemUpdated } from '../lib/library-events.js';
  import DetailSection from '../components/DetailSection.svelte';

  let { gameId } = $props();

  let log = $state(null);
  let root = $state(null);
  let chartW = $state(0);
  let hoverPx = $state(null);

  const H = 246;
  const LEFT = 56;
  const RIGHT = 18;
  const TOP = 44;
  const BASE = 198;
  const LABEL_Y = 224;
  const TAIL_PX = 14;

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

  $effect(() => {
    if (!log?.sessions?.some((session) => session.ended_at == null)) return;
    let live = true;
    const timer = setInterval(() => load(() => live), 30000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  });

  onMount(() => {
    const unwatch = onLibraryItemUpdated((id) => {
      if (id === gameId) reload();
    });
    window.addEventListener(LIBRARY_CHANGED, reload);
    return () => {
      unwatch();
      window.removeEventListener(LIBRARY_CHANGED, reload);
    };
  });

  const aliveMode = $derived(getTimeMetric() === 'alive');
  const average = $derived(averageSeconds(log?.total_seconds, log?.count));
  const series = $derived(buildCumulativeSeries(log?.sessions ?? [], aliveMode ? 'alive' : 'foreground'));
  const W = $derived(chartW > 0 ? chartW : 900);
  const plotW = $derived(W - LEFT - RIGHT);
  const dataSpan = $derived(Math.max((series.last ?? 0) - (series.first ?? 0), 86400));
  const span = $derived(dataSpan * (1 + TAIL_PX / Math.max(1, plotW)));

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

  function x(at) {
    return LEFT + ((at - (series.first ?? 0)) / span) * (W - LEFT - RIGHT);
  }

  function y(value) {
    return BASE - (value / Math.max(1, axis.max)) * (BASE - TOP);
  }

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

  const paths = $derived.by(() => {
    const points = series.points;
    if (!points.length) return { line: '', area: '' };
    const startX = x(series.first).toFixed(1);
    const steps = [`M${startX},${y(0).toFixed(1)}`];
    let previous = 0;
    for (const point of points) {
      const px = x(point.at).toFixed(1);
      steps.push(`L${px},${y(previous).toFixed(1)}`);
      steps.push(`L${px},${y(point.total).toFixed(1)}`);
      previous = point.total;
    }
    const endX = (W - RIGHT).toFixed(1);
    steps.push(`L${endX},${y(previous).toFixed(1)}`);
    const line = steps.join(' ');
    return { line, area: `${line} L${endX},${BASE} L${startX},${BASE} Z` };
  });

  const marks = $derived.by(() => {
    if (!series.first) return [];
    const count = span < 3 * 86400 ? 1 : W < 560 ? 3 : 4;
    if (count === 1) {
      const at = series.first + span / 2;
      return [{ at, px: x(at), anchor: 'middle', label: markLabel(at) }];
    }
    const out = [];
    for (let index = 0; index < count; index += 1) {
      const at = series.first + (span * index) / (count - 1);
      out.push({
        at,
        px: x(at),
        anchor: index === 0 ? 'start' : index === count - 1 ? 'end' : 'middle',
        label: markLabel(at),
      });
    }
    return out;
  });

  const peak = $derived.by(() => {
    let best = null;
    for (const point of series.points) {
      if (!best || point.step > best.step) best = point;
    }
    return best;
  });

  const longest = $derived(peak?.step ?? 0);

  const metricText = $derived(
    aliveMode ? t('settings.timeAlive') : t('settings.timeForeground'),
  );

  const endLabel = $derived.by(() => {
    if (!series.total) return null;
    return {
      x: W - RIGHT,
      y: Math.max(34, y(series.total) - 10),
      text: compactDuration(Math.round(series.total / 60)),
    };
  });

  export function scrollTo() {
    root?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  function trackHover(event) {
    if (!series.points.length) return;
    const rect = event.currentTarget.getBoundingClientRect();
    if (!rect.width) return;
    const px = ((event.clientX - rect.left) / rect.width) * W;
    hoverPx = Math.min(Math.max(px, LEFT), W - RIGHT);
  }

  const hover = $derived.by(() => {
    if (hoverPx == null || !series.points.length) return null;
    const at = (series.first ?? 0) + ((hoverPx - LEFT) / Math.max(1, plotW)) * span;
    let index = -1;
    for (let cursor = 0; cursor < series.points.length; cursor += 1) {
      if (series.points[cursor].at <= at) index = cursor;
      else break;
    }
    if (index < 0) return null;
    const point = series.points[index];
    return {
      px: hoverPx,
      y: y(point.total),
      date: shortDate(point.at),
      step: compactDuration(Math.round(point.step / 60)),
      total: compactDuration(Math.round(point.total / 60)),
    };
  });
</script>

{#if log?.count}
  <div class="detail-playlog-anchor" bind:this={root}>
    <DetailSection title={t('detail.playLog')} count={log.count} class="detail-enter-playlog">
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
      <svg
        class="detail-playlog-chart"
        viewBox={`0 0 ${W} ${H}`}
        height={H}
        bind:clientWidth={chartW}
        role="img"
        aria-label={t('detail.playLog')}
        onpointermove={trackHover}
        onpointerleave={() => (hoverPx = null)}
      >
        <text class="detail-playlog-metric" x={W - RIGHT} y={13} text-anchor="end">
          {metricText}
        </text>

        {#each axis.values as tick}
          <line class="detail-playlog-grid" x1={LEFT} x2={W - RIGHT} y1={y(tick)} y2={y(tick)} />
          <text class="detail-playlog-axis" x={LEFT - 8} y={y(tick) + 4} text-anchor="end">
            {axisText(tick)}
          </text>
        {/each}

        <path class="detail-playlog-area" d={paths.area} />
        <path class="detail-playlog-line" d={paths.line} />

        {#if hover}
          <line class="detail-playlog-cursor" x1={hover.px} y1={TOP} x2={hover.px} y2={BASE} />
          <circle class="detail-playlog-cursor-dot" cx={hover.px} cy={hover.y} r="4" />
          <text class="detail-playlog-hover" x={LEFT} y={13}>{hover.date}<tspan class="detail-playlog-hover-step" dx="10">{hover.step}</tspan><tspan dx="10">{t('detail.playLogCumulative')} {hover.total}</tspan></text>
        {/if}

        {#each marks as mark, index (index)}
          <text class="detail-playlog-axis" x={mark.px} y={LABEL_Y} text-anchor={mark.anchor}>
            {mark.label}
          </text>
        {/each}

        {#if endLabel}
          <text class="detail-playlog-end" x={endLabel.x} y={endLabel.y} text-anchor="end">
            {endLabel.text}
          </text>
        {/if}

        <circle class="detail-playlog-dot" cx={W - RIGHT} cy={y(series.total)} r="3.5" />
      </svg>
    </DetailSection>
  </div>
{/if}
