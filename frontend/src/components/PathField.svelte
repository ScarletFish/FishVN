<script>
  import { t } from '../lib/i18n.js';

  let {
    value = $bindable(''),
    picker,
    pickerTitle = '',
    onPick = null,
    placeholder = '',
    ariaLabel = '',
    browseLabel = t('common.browse'),
    error = '',
    disabled = false,
    id = undefined,
  } = $props();

  let pickError = $state('');

  const hint = $derived(error || pickError);

  async function browse() {
    if (disabled) return;
    pickError = '';
    try {
      const picked = await picker(pickerTitle);
      if (!picked) return;
      if (onPick && onPick(picked) === false) return;
      value = picked;
    } catch (err) {
      pickError = err.message;
    }
  }
</script>

<div class="path-field">
  <div class="path-field__row">
    <input
      {id}
      type="text"
      bind:value
      placeholder={placeholder}
      aria-label={ariaLabel || undefined}
      spellcheck="false"
      autocomplete="off"
      {disabled}
    />
    <button type="button" class="btn path-field__browse" onclick={browse} {disabled}>
      {browseLabel}
    </button>
  </div>
  {#if hint}
    <p class="form-hint form-hint--error">{hint}</p>
  {/if}
</div>
