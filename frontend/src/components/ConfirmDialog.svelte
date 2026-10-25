<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false), action, params = {}, onresolve } = $props();

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
    </div>
  </div>

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => settle(false)}>
      {t(`confirm.${action}.cancel`)}
    </button>
    <button type="button" class="btn btn-primary" onclick={() => settle(true)}>
      {t(`confirm.${action}.confirm`)}
    </button>
  </div>
</Modal>
