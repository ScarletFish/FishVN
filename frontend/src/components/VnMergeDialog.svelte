<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { asText } from '../lib/vn-merge.js';

  let { open = $bindable(false), rows = [], onaccept, oncancel } = $props();

  let picked = $state(new Set());

  $effect(() => {
    if (open) picked = new Set(rows.map((r) => r.key));
  });

  function toggle(key) {
    const next = new Set(picked);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    picked = next;
  }
</script>

<Modal {open} title={t('gameForm.merge.title')} variant="modal--merge" onclose={oncancel}>
  {#if rows.length === 0}
    <div class="vn-merge__empty">{t('gameForm.merge.empty')}</div>
  {:else}
    <div class="vn-merge__head" aria-hidden="true">
      <span></span>
      <span>{t('gameForm.merge.current')}</span>
      <span>{t('gameForm.merge.incoming')}</span>
    </div>
    <ul class="vn-merge__rows scroll-thin">
      {#each rows as row (row.key)}
        {@const on = picked.has(row.key)}
        <li class="vn-merge__row">
          <span class="vn-merge__label">{t(`gameForm.field.${row.key}`)}</span>
          {#each [[row.current, !on], [row.incoming, on]] as [value, active], side}
            <span class="vn-merge__cell" class:is-active={active}>
              <span class="vn-merge__text">{asText(row.key, value)}</span>
              <button
                type="button"
                class="vn-merge__dot"
                aria-pressed={active}
                aria-label={t(`gameForm.merge.${side === 0 ? 'keep' : 'adopt'}`)}
                onclick={() => toggle(row.key)}
              ></button>
            </span>
          {/each}
        </li>
      {/each}
    </ul>
  {/if}

  <div class="modal-actions">
    <button type="button" class="btn" onclick={oncancel}>
      {t('common.cancel')}
    </button>
    <button type="button" class="btn btn-primary" onclick={() => onaccept?.(picked)}>
      {t('gameForm.merge.accept', { count: picked.size })}
    </button>
  </div>
</Modal>