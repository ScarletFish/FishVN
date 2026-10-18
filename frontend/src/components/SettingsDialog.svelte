<script>
  import Modal from './Modal.svelte';
  import { getTheme, toggleTheme } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';
  import { toast } from '../lib/toast.svelte.js';
  import { api } from '../lib/api.js';

  let { open = $bindable(false) } = $props();

  let tab = $state('appearance');
  let dark = $state(getTheme() === 'dark');
  let timeMetric = $state('foreground');

  $effect(() => {
    if (!open) return;
    dark = getTheme() === 'dark';
    api
      .getConfig()
      .then((config) => {
        timeMetric = config.time_metric === 'alive' ? 'alive' : 'foreground';
      })
      .catch(() => {});
  });

  function onToggle() {
    dark = toggleTheme() === 'dark';
  }

  function onMetricChange(e) {
    const value = e.target.value === 'alive' ? 'alive' : 'foreground';
    timeMetric = value;
    api
      .updateConfig({ time_metric: value })
      .catch((err) => toast(err.message, 'error'));
  }
</script>

<Modal bind:open title={t('settings.title')} variant="modal--settings">
  <div class="modal-tabs" role="tablist">
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'appearance'}
      aria-selected={tab === 'appearance'}
      onclick={() => (tab = 'appearance')}
    >
      {t('settings.tab.appearance')}
    </button>
  </div>

  <div class="modal-panels scroll-thin">
    <div class="modal-panel" class:active={tab === 'appearance'}>
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

      <div class="form-group">
        <div class="form-label">{t('settings.timeMetric')}</div>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.timeMetric')}>
          <label
            class="seg-radio-item"
            data-tooltip={t('settings.timeForegroundDesc')}
          >
            <input
              type="radio"
              name="settings-time-metric"
              value="foreground"
              checked={timeMetric === 'foreground'}
              onchange={onMetricChange}
            />
            {t('settings.timeForeground')}
          </label>
          <label
            class="seg-radio-item"
            data-tooltip={t('settings.timeAliveDesc')}
          >
            <input
              type="radio"
              name="settings-time-metric"
              value="alive"
              checked={timeMetric === 'alive'}
              onchange={onMetricChange}
            />
            {t('settings.timeAlive')}
          </label>
        </div>
        <p class="form-hint">{t('settings.timeMetricHint')}</p>
      </div>
    </div>
  </div>

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => (open = false)}>
      {t('common.close')}
    </button>
  </div>
</Modal>
