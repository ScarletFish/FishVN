<script>
  import Icon from '../Icon.svelte';
  import { t } from '../../lib/i18n.js';

  let { title, filled = false, enabled = true, ontoggle, onclear, children } = $props();
</script>

<article class="cond-card" class:is-filled={filled} class:is-off={!enabled}>
  <header class="cond-card__head">
    <h3 class="cond-card__title">{title}</h3>
    <span class="cond-card__spacer"></span>
    <button
      type="button"
      class="cond-card__clear"
      data-tooltip={t('query.clear')}
      aria-label={t('query.clear')}
      onclick={() => onclear?.()}
    >
      <Icon name="x" size="sm" />
    </button>
    <label class="toggle-switch toggle-switch--sm cond-card__switch">
      <input
        type="checkbox"
        checked={enabled}
        aria-label={t('query.enable')}
        onchange={() => ontoggle?.()}
      />
      <span class="toggle-slider"></span>
    </label>
  </header>
  {@render children()}
</article>
