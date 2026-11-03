<script>
  import { DatePicker } from 'bits-ui';
  import { parseDate, today, getLocalTimeZone } from '@internationalized/date';
  import { t } from '../../lib/i18n.js';
  import CalendarPopover from '../CalendarPopover.svelte';

  let { from = $bindable(''), to = $bindable(''), disabled = false } = $props();

  const CAL_MIN = parseDate('1980-01-01');
  const CAL_MAX = parseDate('2030-12-31');

  const fromValue = $derived(from ? parseDate(from) : undefined);
  const toValue = $derived(to ? parseDate(to) : undefined);

  let fromPlaceholder = $state(today(getLocalTimeZone()));
  let toPlaceholder = $state(today(getLocalTimeZone()));
  $effect(() => {
    if (fromValue) fromPlaceholder = fromValue;
  });
  $effect(() => {
    if (toValue) toPlaceholder = toValue;
  });
</script>

<div class="date-range" class:is-disabled={disabled}>
  <DatePicker.Root
    granularity="day"
    locale="zh-CN"
    minValue={CAL_MIN}
    maxValue={CAL_MAX}
    calendarLabel={t('query.dim.released')}
    {disabled}
    bind:value={() => fromValue, (v) => (from = v ? v.toString() : '')}
    bind:placeholder={fromPlaceholder}
  >
    <div class="date-field" class:is-empty={!fromValue} class:is-disabled={disabled}>
      <DatePicker.Input class="date-field__input">
        {#snippet children({ segments })}
          {#each segments as seg, i (i)}
            {#if seg.part === 'literal'}
              <span class="date-field__literal">{seg.value}</span>
            {:else}
              <DatePicker.Segment part={seg.part} class="date-field__seg">
                {seg.value}
              </DatePicker.Segment>
            {/if}
          {/each}
        {/snippet}
      </DatePicker.Input>
      <DatePicker.Trigger class="date-field__cal-btn" aria-label={t('query.from')}>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M7 3v3M17 3v3M4 8h16M5 5h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </DatePicker.Trigger>
    </div>
    <CalendarPopover bind:placeholder={fromPlaceholder} />
  </DatePicker.Root>

  <span class="date-range__sep">{t('query.to')}</span>

  <DatePicker.Root
    granularity="day"
    locale="zh-CN"
    minValue={CAL_MIN}
    maxValue={CAL_MAX}
    calendarLabel={t('query.dim.released')}
    {disabled}
    bind:value={() => toValue, (v) => (to = v ? v.toString() : '')}
    bind:placeholder={toPlaceholder}
  >
    <div class="date-field" class:is-empty={!toValue} class:is-disabled={disabled}>
      <DatePicker.Input class="date-field__input">
        {#snippet children({ segments })}
          {#each segments as seg, i (i)}
            {#if seg.part === 'literal'}
              <span class="date-field__literal">{seg.value}</span>
            {:else}
              <DatePicker.Segment part={seg.part} class="date-field__seg">
                {seg.value}
              </DatePicker.Segment>
            {/if}
          {/each}
        {/snippet}
      </DatePicker.Input>
      <DatePicker.Trigger class="date-field__cal-btn" aria-label={t('query.to')}>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M7 3v3M17 3v3M4 8h16M5 5h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </DatePicker.Trigger>
    </div>
    <CalendarPopover bind:placeholder={toPlaceholder} />
  </DatePicker.Root>
</div>
