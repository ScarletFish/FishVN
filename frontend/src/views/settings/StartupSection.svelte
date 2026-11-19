<script>
  import PathField from '../../components/PathField.svelte';
  import SelectDd from '../../components/SelectDd.svelte';
  import { readConfig, saveConfig } from '../../lib/config-store.svelte.js';
  import { pickExecutable } from '../../lib/pick-path.js';
  import { api } from '../../lib/api.js';
  import { STARTUP_PAGE_IDS } from '../../lib/startup-pages.js';
  import { t } from '../../lib/i18n.js';

  const config = readConfig();

  let magpiePath = $state(config.magpie_path);
  let lePath = $state(config.le_path);
  let magpiePathError = $state('');
  let lePathError = $state('');

  function onStartupPageChange(next) {
    saveConfig({ startup_page: next });
  }

  function onMinimizeOnLaunchChange(e) {
    saveConfig({ minimize_on_launch: e.target.checked });
  }

  function onQuitBehaviorChange(e) {
    saveConfig({ quit_behavior: e.target.value });
  }

  async function saveToolPath(field, raw) {
    const path = (raw ?? '').trim();
    const isMagpie = field === 'magpie';
    if (isMagpie) magpiePathError = '';
    else lePathError = '';
    if (path && !(await api.pathExists(path))) {
      if (isMagpie) magpiePathError = t('settings.tools.pathMissing');
      else lePathError = t('settings.tools.pathMissing');
      return;
    }
    saveConfig({ [isMagpie ? 'magpie_path' : 'le_path']: path });
  }
</script>

<section class="settings-section" id="settings-startup">
  <h2 class="settings-section__title">{t('settings.tab.startup')}</h2>

  <div class="settings-card">
    <div class="form-group">
      <div class="form-label">{t('settings.startupPage')}</div>
      <SelectDd
        id="settings-startup-page"
        options={STARTUP_PAGE_IDS.map((id) => ({ value: id, label: t(`nav.${id}`) }))}
        value={config.startup_page}
        onchange={onStartupPageChange}
        ariaLabel={t('settings.startupPage')}
      />
      <p class="form-hint">{t('settings.startupPageHint')}</p>
    </div>

    <div class="form-group">
      <div class="form-row">
        <span class="form-row-label">{t('settings.minimizeOnLaunch')}</span>
        <label class="toggle-switch" for="settings-minimize-on-launch">
          <input
            id="settings-minimize-on-launch"
            type="checkbox"
            aria-label={t('settings.minimizeOnLaunch')}
            checked={config.minimize_on_launch}
            onchange={onMinimizeOnLaunchChange}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.quitBehavior')}</div>
      <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.quitBehavior')}>
        <label class="seg-radio-item">
          <input
            type="radio"
            name="quit-behavior"
            value="quit"
            checked={config.quit_behavior === 'quit'}
            onchange={onQuitBehaviorChange}
          />
          {t('settings.quitQuit')}
        </label>
        <label class="seg-radio-item">
          <input
            type="radio"
            name="quit-behavior"
            value="minimize_to_tray"
            checked={config.quit_behavior === 'minimize_to_tray'}
            onchange={onQuitBehaviorChange}
          />
          {t('settings.quitTray')}
        </label>
      </div>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.tools.magpie')}</div>
      <PathField
        bind:value={magpiePath}
        picker={pickExecutable}
        pickerTitle={t('settings.tools.pickMagpie')}
        oncommit={(picked) => saveToolPath('magpie', picked)}
        placeholder={t('settings.tools.pathPlaceholder')}
        ariaLabel={t('settings.tools.magpie')}
        error={magpiePathError}
      />
      <p class="form-hint">{t('settings.tools.magpieHint')}</p>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.tools.le')}</div>
      <PathField
        bind:value={lePath}
        picker={pickExecutable}
        pickerTitle={t('settings.tools.pickLe')}
        oncommit={(picked) => saveToolPath('le', picked)}
        placeholder={t('settings.tools.pathPlaceholder')}
        ariaLabel={t('settings.tools.le')}
        error={lePathError}
      />
      <p class="form-hint">{t('settings.tools.leHint')}</p>
    </div>
  </div>
</section>
