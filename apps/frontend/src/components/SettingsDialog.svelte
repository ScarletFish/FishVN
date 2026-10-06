<script>
  import Modal from './Modal.svelte';
  import { getTheme, toggleTheme } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false) } = $props();

  let dark = $state(getTheme() === 'dark');

  $effect(() => {
    if (!open) return;
    dark = getTheme() === 'dark';
  });

  function onToggle() {
    dark = toggleTheme() === 'dark';
  }
</script>

<Modal bind:open title={t('settings.title')} variant="modal--settings">
  <div class="modal-panels">
    <div class="modal-panel active">
      <div class="form-group">
        <label for="settings-theme">{t('settings.theme')}</label>
        <div class="form-row">
          <label class="toggle-switch" for="settings-theme">
            <input id="settings-theme" type="checkbox" checked={dark} onchange={onToggle} />
            <span class="toggle-slider"></span>
          </label>
          <span class="form-row-label">{t('settings.themeHint')}</span>
        </div>
      </div>
    </div>
  </div>

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => (open = false)}>
      {t('common.close')}
    </button>
  </div>
</Modal>
