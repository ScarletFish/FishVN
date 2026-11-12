<script>
  import Icon from '../Icon.svelte';
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
        <Icon name="calendar" />
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
        <Icon name="calendar" />
      </DatePicker.Trigger>
    </div>
    <CalendarPopover bind:placeholder={toPlaceholder} />
  </DatePicker.Root>
</div>
