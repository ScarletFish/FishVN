<script>
  import Icon from '../Icon.svelte';
  import { t } from '../../lib/i18n.js';

  let {
    value = $bindable(''),
    step = 1,
    min = 0,
    max = null,
    placeholder = '',
    ariaLabel = '',
  } = $props();

  function bump(delta) {
    const cur = parseFloat(value);
    let next = (Number.isFinite(cur) ? cur : 0) + delta;
    if (next < min) next = min;
    if (max !== null && next > max) next = max;
    const decimals = (String(step).split('.')[1] || '').length;
    value = String(Number(next.toFixed(decimals)));
  }
</script>

<div class="stepper">
  <input type="number" {placeholder} aria-label={ariaLabel} bind:value />
  <div class="stepper__btns">
    <button type="button" class="stepper__btn" aria-label={t('query.stepUp')} onclick={() => bump(step)}>
      <Icon name="caret-up" />
    </button>
    <button type="button" class="stepper__btn" aria-label={t('query.stepDown')} onclick={() => bump(-step)}>
      <Icon name="caret-down" />
    </button>
  </div>
</div>
