<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false), action, params = {}, onresolve } = $props();

  let removeFiles = $state(false);

  $effect(() => {
    if (open) removeFiles = false;
  });

  function settle(result) {
    onresolve?.(result);
    open = false;
  }
</script>

<Modal
  bind:open
  title={t(`confirm.${action}.title`)}
  onclose={() => settle(false)}
>
  <div class="modal-panels modal-panels--static">
    <div class="modal-panel active">
      <p class="confirm-message">{t(`confirm.${action}.message`, params)}</p>
      {#if action === 'deleteGame'}
        <label class="confirm-option">
          <input type="checkbox" class="checkbox" bind:checked={removeFiles} />
          {t('confirm.deleteGame.removeFiles')}
        </label>
      {/if}
    </div>
  </div>

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => settle(false)}>
      {t(`confirm.${action}.cancel`)}
    </button>
    <button
      type="button"
      class="btn btn-primary"
      onclick={() => settle(action === 'deleteGame' ? { confirmed: true, removeFiles } : true)}
    >
      {t(`confirm.${action}.confirm`)}
    </button>
  </div>
</Modal>
