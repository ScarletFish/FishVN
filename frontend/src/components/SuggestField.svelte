<script>
  import Icon from './Icon.svelte';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { t } from '../lib/i18n.js';

  let {
    value = $bindable(''),
    options = [],
    id = undefined,
    placeholder = '',
    disabled = false,
    class: className = '',
    inputClass = '',
    ariaLabel = undefined,
    onchange = () => {},
  } = $props();

  let rootEl = $state(null);
  let inputEl = $state(null);
  let open = $state(false);
  let active = $state(-1);

  const filtered = $derived.by(() => {
    const q = String(value ?? '').trim().toLowerCase();
    return options.filter((opt) => !q || String(opt).toLowerCase().includes(q));
  });

  function show() {
    if (disabled || open) return;
    open = true;
    active = -1;
  }

  function hide() {
    open = false;
    active = -1;
  }

  function pick(option) {
    value = option;
    onchange(option);
    hide();
    inputEl?.focus();
  }

  function onDocPointerDown(e) {
    if (open && rootEl && !rootEl.contains(e.target)) hide();
  }

  function onKeydown(e) {
    if (!open) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        show();
      }
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      hide();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      active = Math.min(active + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      active = Math.max(active - 1, 0);
    } else if (e.key === 'Enter' && active >= 0 && filtered[active]) {
      e.preventDefault();
      pick(filtered[active]);
    }
  }
</script>

<svelte:window onpointerdown={onDocPointerDown} />

<div class="suggest-field {className}" bind:this={rootEl}>
  <div class="suggest-field__control">
    <input
      bind:this={inputEl}
      {id}
      type="text"
      class={inputClass}
      {value}
      {placeholder}
      {disabled}
      aria-label={ariaLabel}
      autocomplete="off"
      oninput={(e) => {
        value = e.currentTarget.value;
        onchange(value);
        if (open) active = -1;
      }}
      onkeydown={onKeydown}
    />
    <button
      type="button"
      class="suggest-field__toggle"
      aria-label={t('suggestField.toggle')}
      aria-expanded={open}
      tabindex="-1"
      {disabled}
      onclick={() => (open ? hide() : show())}
    >
      <Icon name="caret-down" />
    </button>
  </div>
  {#if open}
    <div class="suggest-field__menu" use:panelScrollbars>
      {#if filtered.length === 0}
        <p class="suggest-field__empty">{t('suggestField.empty')}</p>
      {:else}
        {#each filtered as option, i (option)}
          <button
            type="button"
            class="suggest-field__option"
            class:is-active={i === active}
            onpointerdown={(e) => e.preventDefault()}
            onclick={() => pick(option)}
          >
            {option}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .suggest-field {
    position: relative;
    width: 100%;
    min-width: 0;
  }

  .suggest-field__control {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    min-width: 0;
  }

  .suggest-field__control input {
    flex: 1;
    min-width: 0;
    height: var(--space-10);
    padding-right: calc(var(--space-6) + var(--space-2));
  }

  .suggest-field__toggle {
    position: absolute;
    right: var(--space-2);
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--space-6);
    height: 100%;
    padding: 0;
    color: var(--fg-default);
    background: none;
    border: none;
    cursor: pointer;
  }

  .suggest-field__toggle:disabled {
    cursor: default;
    opacity: 0.5;
  }

  .suggest-field__menu {
    position: absolute;
    top: calc(100% + var(--space-1));
    left: 0;
    z-index: var(--z-dropdown);
    display: flex;
    flex-direction: column;
    gap: var(--space-025);
    width: 100%;
    max-height: 15rem;
    padding: var(--space-1);
    overflow-y: auto;
    background: var(--bg-floating);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-raised);
    animation: suggest-in var(--duration-fast) var(--ease-out) both;
  }

  @keyframes suggest-in {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .suggest-field__option {
    padding: var(--space-250) var(--space-3);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    color: var(--fg-default);
    text-align: left;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .suggest-field__option:hover,
  .suggest-field__option.is-active {
    background: var(--bg-hover);
  }

  .suggest-field__empty {
    margin: 0;
    padding: var(--space-2) var(--space-3);
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }
</style>
