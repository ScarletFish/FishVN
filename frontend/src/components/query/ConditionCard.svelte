<script>
  import { t } from '../../lib/i18n.js';

  let { title, filled = false, enabled = true, ontoggle, onclear, children } = $props();
</script>

<article class="cond-card" class:is-filled={filled} class:is-off={!enabled}>
  <header class="cond-card__head">
    <h3 class="cond-card__title">{title}</h3>
    <span class="cond-card__spacer"></span>
    {#if filled}
      <button
        type="button"
        class="cond-card__clear"
        data-tooltip={t('query.clear')}
        aria-label={t('query.clear')}
        onclick={() => onclear?.()}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    {/if}
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
