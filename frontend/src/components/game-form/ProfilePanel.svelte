<script>
  import { DatePicker } from 'bits-ui';
  import { parseDate, today, getLocalTimeZone } from '@internationalized/date';
  import { t } from '../../lib/i18n.js';
  import CalendarPopover from '../CalendarPopover.svelte';
  import StaffPanel from './StaffPanel.svelte';
  import {
    cleanInteger,
    cleanRating,
    normalizeInteger,
    normalizeRating,
  } from '../../lib/form-metadata.js';

  let {
    alias = $bindable(''),
    released = $bindable(''),
    rating = $bindable(''),
    votecount = $bindable(''),
    lengthMinutes = $bindable(''),
    developers = $bindable(''),
    staff = $bindable([]),
    staffRoleOpen = $bindable(null),
    ratingError = '',
    votecountError = '',
    lengthMinutesError = '',
    ontouch = () => {},
    saving = false,
  } = $props();

  function onNumberInput(e, field, clean) {
    const cleaned = clean(e.currentTarget.value);
    e.currentTarget.value = cleaned;
    if (field === 'rating') rating = cleaned;
    else if (field === 'votecount') votecount = cleaned;
    else lengthMinutes = cleaned;
  }

  function parseReleasedDate(raw) {
    const text = String(raw ?? '').trim();
    if (!/^\d{4}-\d{2}-\d{2}$/.test(text)) return undefined;
    try {
      return parseDate(text);
    } catch {
      return undefined;
    }
  }

  const releasedDate = $derived(parseReleasedDate(released));

  function setReleasedDate(v) {
    released = v ? v.toString() : '';
    ontouch('released');
  }

  const CAL_MIN = parseDate('1980-01-01');
  const CAL_MAX = parseDate('2030-12-31');

  let calPlaceholder = $state(today(getLocalTimeZone()));
  $effect(() => {
    if (releasedDate) calPlaceholder = releasedDate;
  });
</script>

<div class="panel-grid">
  <div class="form-group panel-grid__full">
    <label for="game-form-meta-alias">{t('gameForm.field.alias')}</label>
    <input
      id="game-form-meta-alias"
      type="text"
      bind:value={alias}
      placeholder={t('gameForm.placeholder.alias')}
      disabled={saving}
    />
  </div>

  <div class="form-group">
    <label for="game-form-meta-released">{t('gameForm.field.released')}</label>
    <DatePicker.Root
      granularity="day"
      locale="zh-CN"
      minValue={CAL_MIN}
      maxValue={CAL_MAX}
      calendarLabel={t('gameForm.field.released')}
      disabled={saving}
      bind:value={() => releasedDate, setReleasedDate}
      bind:placeholder={calPlaceholder}
    >
      <div
        class="date-field"
        class:is-empty={!releasedDate}
        class:is-disabled={saving}
      >
        <DatePicker.Input id="game-form-meta-released" class="date-field__input">
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
        <DatePicker.Trigger class="date-field__cal-btn" aria-label={t('gameForm.field.released')}>
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
      <CalendarPopover bind:placeholder={calPlaceholder} />
    </DatePicker.Root>
  </div>

  <div class="form-group">
    <label
      for="game-form-meta-rating"
      data-tooltip={t('gameForm.hint.ratingScale')}
    >
      {t('gameForm.field.rating')}
    </label>
    <input
      id="game-form-meta-rating"
      type="text"
      inputmode="decimal"
      value={rating}
      placeholder={t('gameForm.placeholder.rating')}
      disabled={saving}
      oninput={(e) => onNumberInput(e, 'rating', cleanRating)}
      onblur={() => {
        rating = normalizeRating(rating);
        ontouch('rating');
      }}
    />
    <p class="form-hint">{t('gameForm.hint.ratingScale')}</p>
    {#if ratingError}
      <p class="form-hint form-hint--error">{ratingError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-votecount">{t('gameForm.field.votecount')}</label>
    <input
      id="game-form-meta-votecount"
      type="text"
      inputmode="numeric"
      value={votecount}
      placeholder={t('gameForm.placeholder.votecount')}
      disabled={saving}
      oninput={(e) => onNumberInput(e, 'votecount', (raw) => cleanInteger(raw, 7))}
      onblur={() => {
        votecount = normalizeInteger(votecount, { max: 9999999, maxIntDigits: 7 });
        ontouch('votecount');
      }}
    />
    {#if votecountError}
      <p class="form-hint form-hint--error">{votecountError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-length">{t('gameForm.field.lengthMinutes')}</label>
    <input
      id="game-form-meta-length"
      type="text"
      inputmode="numeric"
      value={lengthMinutes}
      placeholder={t('gameForm.placeholder.lengthMinutes')}
      disabled={saving}
      oninput={(e) => onNumberInput(e, 'lengthMinutes', (raw) => cleanInteger(raw, 5))}
      onblur={() => {
        lengthMinutes = normalizeInteger(lengthMinutes, { max: 99999, maxIntDigits: 5 });
        ontouch('lengthMinutes');
      }}
    />
    {#if lengthMinutesError}
      <p class="form-hint form-hint--error">{lengthMinutesError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-developers">{t('gameForm.field.developers')}</label>
    <input
      id="game-form-meta-developers"
      type="text"
      bind:value={developers}
      placeholder={t('gameForm.placeholder.developers')}
      disabled={saving}
    />
  </div>

  <StaffPanel bind:staff bind:staffRoleOpen {saving} />
</div>
