<script>
  import { DatePicker, Select } from 'bits-ui';
  import { parseDate, today, getLocalTimeZone } from '@internationalized/date';
  import { t } from '../../lib/i18n.js';
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
  const YEAR_OPTIONS = Array.from({ length: 2030 - 1980 + 1 }, (_, i) => {
    const y = String(1980 + i);
    return { value: y, label: y };
  });
  const MONTH_OPTIONS = Array.from({ length: 12 }, (_, i) => {
    const m = String(i + 1);
    return { value: m, label: `${m}月` };
  });

  let calPlaceholder = $state(releasedDate ?? today(getLocalTimeZone()));
  $effect(() => {
    if (releasedDate) calPlaceholder = releasedDate;
  });

  const setCalYear = (v) => {
    calPlaceholder = calPlaceholder.set({ year: Number(v) });
  };
  const setCalMonth = (v) => {
    calPlaceholder = calPlaceholder.set({ month: Number(v) });
  };
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
      <DatePicker.Portal to="#modal-root">
        <DatePicker.Content class="cal-pop" sideOffset={6}>
          <DatePicker.Calendar>
            {#snippet children({ months, weekdays })}
              <div class="cal-pop__head">
                <DatePicker.PrevButton
                  class="cal-pop__nav"
                  aria-label={t('gameForm.cal.prevMonth')}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path
                      d="m14 6-6 6 6 6"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </DatePicker.PrevButton>
                <div class="cal-pop__selects">
                  <Select.Root
                    type="single"
                    bind:value={() => String(calPlaceholder.year), setCalYear}
                  >
                    <Select.Trigger aria-label={t('gameForm.cal.selectYear')}>
                      {#snippet child({ props })}
                        <button {...props} type="button" class="cal-pop__select">
                          {calPlaceholder.year}
                        </button>
                      {/snippet}
                    </Select.Trigger>
                    <Select.Portal to="#modal-root">
                      <Select.Content class="cal-pop__menu" align="start" sideOffset={4}>
                        <Select.Viewport class="cal-pop__viewport">
                        {#each YEAR_OPTIONS as opt (opt.value)}
                          <Select.Item
                            value={opt.value}
                            label={opt.label}
                            class={Number(opt.value) === calPlaceholder.year
                              ? 'cal-pop__opt is-selected'
                              : 'cal-pop__opt'}
                          >
                            <span>{opt.label}</span>
                            <svg
                              class="cal-pop__opt-check"
                              viewBox="0 0 24 24"
                              fill="none"
                              stroke="currentColor"
                              stroke-width="2.5"
                            >
                              <polyline points="20 6 9 17 4 12" />
                            </svg>
                          </Select.Item>
                        {/each}
                        </Select.Viewport>
                      </Select.Content>
                    </Select.Portal>
                  </Select.Root>
                  <Select.Root
                    type="single"
                    bind:value={() => String(calPlaceholder.month), setCalMonth}
                  >
                    <Select.Trigger aria-label={t('gameForm.cal.selectMonth')}>
                      {#snippet child({ props })}
                        <button {...props} type="button" class="cal-pop__select">
                          {calPlaceholder.month}月
                        </button>
                      {/snippet}
                    </Select.Trigger>
                    <Select.Portal to="#modal-root">
                      <Select.Content class="cal-pop__menu" align="start" sideOffset={4}>
                        <Select.Viewport class="cal-pop__viewport">
                        {#each MONTH_OPTIONS as opt (opt.value)}
                          <Select.Item
                            value={opt.value}
                            label={opt.label}
                            class={Number(opt.value) === calPlaceholder.month
                              ? 'cal-pop__opt is-selected'
                              : 'cal-pop__opt'}
                          >
                            <span>{opt.label}</span>
                            <svg
                              class="cal-pop__opt-check"
                              viewBox="0 0 24 24"
                              fill="none"
                              stroke="currentColor"
                              stroke-width="2.5"
                            >
                              <polyline points="20 6 9 17 4 12" />
                            </svg>
                          </Select.Item>
                        {/each}
                        </Select.Viewport>
                      </Select.Content>
                    </Select.Portal>
                  </Select.Root>
                </div>
                <DatePicker.NextButton
                  class="cal-pop__nav"
                  aria-label={t('gameForm.cal.nextMonth')}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path
                      d="m10 6 6 6-6 6"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </DatePicker.NextButton>
              </div>
              <table class="cal-pop__grid">
                <thead>
                  <tr>
                    {#each weekdays as wd (wd)}
                      <th class="cal-pop__weekday" scope="col">{wd}</th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each months as month (month.value.toString())}
                    {#each month.weeks as week, wi (wi)}
                      <tr>
                        {#each week as day, di (day.toString())}
                          <td class="cal-pop__cell">
                            <DatePicker.Cell date={day} month={month.value}>
                              <DatePicker.Day class="cal-pop__day" />
                            </DatePicker.Cell>
                          </td>
                        {/each}
                      </tr>
                    {/each}
                  {/each}
                </tbody>
              </table>
            {/snippet}
          </DatePicker.Calendar>
        </DatePicker.Content>
      </DatePicker.Portal>
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
