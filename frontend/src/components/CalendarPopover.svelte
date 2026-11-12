<script>
  import Icon from './Icon.svelte';
  import { DatePicker, Select } from 'bits-ui';
  import { t } from '../lib/i18n.js';

  let { placeholder = $bindable() } = $props();

  const YEAR_OPTIONS = Array.from({ length: 2030 - 1980 + 1 }, (_, i) => {
    const y = String(1980 + i);
    return { value: y, label: y };
  });
  const MONTH_OPTIONS = Array.from({ length: 12 }, (_, i) => {
    const m = String(i + 1);
    return { value: m, label: `${m}月` };
  });

  const setCalYear = (v) => {
    placeholder = placeholder.set({ year: Number(v) });
  };
  const setCalMonth = (v) => {
    placeholder = placeholder.set({ month: Number(v) });
  };
</script>

<DatePicker.Portal to="#modal-root">
  <DatePicker.Content class="cal-pop" sideOffset={6}>
    <DatePicker.Calendar>
      {#snippet children({ months, weekdays })}
        <div class="cal-pop__head">
          <DatePicker.PrevButton class="cal-pop__nav" aria-label={t('cal.prevMonth')}>
            <Icon name="caret-left" />
          </DatePicker.PrevButton>
          <div class="cal-pop__selects">
            <Select.Root type="single" bind:value={() => String(placeholder.year), setCalYear}>
              <Select.Trigger aria-label={t('cal.selectYear')}>
                {#snippet child({ props })}
                  <button {...props} type="button" class="cal-pop__select">
                    {placeholder.year}
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
                        class={Number(opt.value) === placeholder.year
                          ? 'cal-pop__opt is-selected'
                          : 'cal-pop__opt'}
                      >
                        <span>{opt.label}</span>
                        <Icon name="check" size="sm" />
                      </Select.Item>
                    {/each}
                  </Select.Viewport>
                </Select.Content>
              </Select.Portal>
            </Select.Root>
            <Select.Root type="single" bind:value={() => String(placeholder.month), setCalMonth}>
              <Select.Trigger aria-label={t('cal.selectMonth')}>
                {#snippet child({ props })}
                  <button {...props} type="button" class="cal-pop__select">
                    {placeholder.month}月
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
                        class={Number(opt.value) === placeholder.month
                          ? 'cal-pop__opt is-selected'
                          : 'cal-pop__opt'}
                      >
                        <span>{opt.label}</span>
                        <Icon name="check" size="sm" />
                      </Select.Item>
                    {/each}
                  </Select.Viewport>
                </Select.Content>
              </Select.Portal>
            </Select.Root>
          </div>
          <DatePicker.NextButton class="cal-pop__nav" aria-label={t('cal.nextMonth')}>
            <Icon name="caret-right" />
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
