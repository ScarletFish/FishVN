<script>
  import { portal } from '../lib/portal.js';
  import { getTheme, toggleTheme } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';

  let { open = $bindable(false) } = $props();

  let dark = $state(getTheme() === 'dark');

  $effect(() => {
    const app = document.getElementById('app');
    if (!app) return;
    app.inert = open;
    return () => {
      app.inert = false;
    };
  });

  $effect(() => {
    if (!open) return;
    dark = getTheme() === 'dark';
  });

  function close() {
    open = false;
  }

  function onOverlayClick(e) {
    // 判定落点，弹窗内部的点击会冒泡上来，不能一律当成点遮罩
    if (e.target === e.currentTarget) close();
  }

  function onKeydown(e) {
    if (e.key === 'Escape' && open) close();
  }

  function onToggle() {
    dark = toggleTheme() === 'dark';
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="modal-overlay show" use:portal onclick={onOverlayClick} role="presentation">
    <div
      class="modal modal--settings"
      role="dialog"
      aria-modal="true"
      aria-label={t('settings.title')}
    >
      <div class="modal-header">
        <h2>{t('settings.title')}</h2>
      </div>

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
        <button type="button" class="btn" onclick={close}>
          {t('common.close')}
        </button>
      </div>
    </div>
  </div>
{/if}