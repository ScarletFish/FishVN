<script>
  import { tick } from 'svelte';
  import Icon from '../../components/Icon.svelte';
  import {
    ACCENTS,
    THEMES,
    getAccent,
    getFontScale,
    getThemeMode,
    setAccent,
    setFontScale,
    setTheme,
    viewTransition,
  } from '../../lib/theme.js';
  import { getShowCardTitle, setShowCardTitle } from '../../lib/card-prefs.js';
  import { getReduceMotion, setReduceMotion } from '../../lib/motion.js';
  import { readConfig, saveConfig } from '../../lib/config-store.svelte.js';
  import {
    DASHBOARD_MODULES,
    moveModule,
    reorderModule,
    setModuleEnabled,
  } from '../../lib/dashboard-layout.js';
  import { t } from '../../lib/i18n.js';

  const config = readConfig();

  let themeMode = $state(getThemeMode());
  let accent = $state(getAccent());
  let fontScale = $state(getFontScale());
  let showCardTitle = $state(getShowCardTitle());
  let reduceMotion = $state(getReduceMotion());
  let dragId = $state(null);
  let dragOverId = $state(null);

  function onThemeChange(e) {
    const next = THEMES.includes(e.target.value) ? e.target.value : 'light';
    if (next === themeMode) return;
    themeMode = next;
    viewTransition(async () => {
      setTheme(next);
      await tick();
    });
    saveConfig({ theme: next });
  }

  function onAccentPick(name) {
    if (name === accent) return;
    viewTransition(async () => {
      accent = setAccent(name);
      await tick();
    });
    saveConfig({ accent: name });
  }

  function onShowCardTitleChange(e) {
    showCardTitle = setShowCardTitle(e.target.checked);
  }

  function onReduceMotionChange(e) {
    reduceMotion = setReduceMotion(e.target.checked);
  }

  function saveLayout(next) {
    saveConfig({ dashboard_layout: next });
  }

  function onHandleDown(e, id) {
    e.preventDefault();
    dragId = id;
    window.addEventListener('pointermove', onDragMove);
    window.addEventListener('pointerup', onDragEnd, { once: true });
  }

  function onDragMove(e) {
    if (!dragId) return;
    const row = document.elementFromPoint(e.clientX, e.clientY)?.closest('.layout-row');
    dragOverId = row?.dataset.id ?? null;
  }

  function onDragEnd() {
    window.removeEventListener('pointermove', onDragMove);
    const target = dragOverId;
    if (dragId && target && target !== dragId) {
      saveLayout(reorderModule(config.dashboard_layout, dragId, target));
    }
    dragId = null;
    dragOverId = null;
  }

  $effect(() => () => {
    window.removeEventListener('pointermove', onDragMove);
  });

  function onIncludeUninstalledChange(e) {
    saveConfig({ dashboard_include_uninstalled: e.target.checked });
  }

  function moduleLabel(id) {
    const known = DASHBOARD_MODULES.find((m) => m.id === id);
    return known ? t(known.label) : id;
  }
</script>

<section class="settings-section" id="settings-appearance">
  <h2 class="settings-section__title">{t('settings.tab.appearance')}</h2>

  <div class="settings-card">
    <div class="form-group">
      <div class="form-label">{t('settings.theme')}</div>
      <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.theme')}>
        {#each THEMES as mode (mode)}
          <label class="seg-radio-item">
            <input
              type="radio"
              name="settings-theme"
              value={mode}
              checked={themeMode === mode}
              onchange={onThemeChange}
            />
            {t('settings.theme_' + mode)}
          </label>
        {/each}
      </div>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.accent')}</div>
      <div class="accent-swatches" role="radiogroup" aria-label={t('settings.accent')}>
        {#each ACCENTS as name (name)}
          <button
            type="button"
            role="radio"
            aria-checked={accent === name}
            aria-label={t('settings.accent_' + name)}
            class="accent-swatch"
            class:active={accent === name}
            data-accent={name}
            data-tooltip={t('settings.accent_' + name)}
            onclick={() => onAccentPick(name)}
          ></button>
        {/each}
      </div>
    </div>

    <div class="form-group">
      <div class="form-label-row">
        <div class="form-label">{t('settings.fontSize')}</div>
        <span class="form-value">{Math.round(fontScale * 100)}%</span>
      </div>
      <input
        class="font-scale-slider"
        type="range"
        min="75"
        max="125"
        step="5"
        value={Math.round(fontScale * 100)}
        aria-label={t('settings.fontSize')}
        oninput={(e) => (fontScale = setFontScale(Number(e.currentTarget.value) / 100))}
      />
      <p class="form-hint">{t('settings.fontSizeHint')}</p>
    </div>

    <div class="form-group">
      <div class="form-row">
        <span class="form-row-label">{t('settings.cardTitle')}</span>
        <label class="toggle-switch" for="settings-card-title">
          <input
            id="settings-card-title"
            type="checkbox"
            aria-label={t('settings.cardTitle')}
            checked={showCardTitle}
            onchange={onShowCardTitleChange}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="form-group">
      <div class="form-row">
        <span class="form-row-label">{t('settings.reduceMotion')}</span>
        <label class="toggle-switch" for="settings-reduce-motion">
          <input
            id="settings-reduce-motion"
            type="checkbox"
            aria-label={t('settings.reduceMotion')}
            checked={reduceMotion}
            onchange={onReduceMotionChange}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <p class="form-hint">{t('settings.reduceMotionHint')}</p>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.dashboardLayout')}</div>
      <div class="layout-rows">
        {#each config.dashboard_layout as module (module.id)}
          <div
            class="layout-row"
            class:dragging={dragId === module.id}
            class:drag-over={dragOverId === module.id}
            data-id={module.id}
          >
            <div class="layout-row-line">
              <span
                class="layout-row-handle"
                role="button"
                tabindex="-1"
                aria-label={t('settings.layoutMoveUp')}
                onpointerdown={(e) => onHandleDown(e, module.id)}
              >
                <Icon name="dots-six" />
              </span>
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  checked={module.enabled}
                  aria-label={moduleLabel(module.id)}
                  onchange={(e) =>
                    saveLayout(setModuleEnabled(config.dashboard_layout, module.id, e.target.checked))}
                />
                <span class="toggle-slider"></span>
              </label>
              <span class="layout-row-label">{moduleLabel(module.id)}</span>
              <span class="layout-row-actions">
                <button
                  type="button"
                  class="btn btn-ghost layout-move-btn"
                  data-tooltip={t('settings.layoutMoveUp')}
                  disabled={module.id === config.dashboard_layout[0]?.id}
                  aria-label={t('settings.layoutMoveUp')}
                  onclick={() => saveLayout(moveModule(config.dashboard_layout, module.id, -1))}
                >
                  <Icon name="caret-up" />
                </button>
                <button
                  type="button"
                  class="btn btn-ghost layout-move-btn"
                  data-tooltip={t('settings.layoutMoveDown')}
                  disabled={module.id ===
                    config.dashboard_layout[config.dashboard_layout.length - 1]?.id}
                  aria-label={t('settings.layoutMoveDown')}
                  onclick={() => saveLayout(moveModule(config.dashboard_layout, module.id, 1))}
                >
                  <Icon name="caret-down" />
                </button>
              </span>
            </div>
            {#if module.id === 'local_library'}
              <div class="layout-subrow" class:disabled={!module.enabled}>
                <label class="layout-subrow-line">
                  <span class="layout-subrow-label">
                    {t('settings.dashboardIncludeUninstalled')}
                  </span>
                  <span class="toggle-switch">
                    <input
                      type="checkbox"
                      checked={config.dashboard_include_uninstalled}
                      disabled={!module.enabled}
                      aria-label={t('settings.dashboardIncludeUninstalled')}
                      onchange={onIncludeUninstalledChange}
                    />
                    <span class="toggle-slider"></span>
                  </span>
                </label>
              </div>
            {/if}
          </div>
        {/each}
        {#if !config.dashboard_layout.some((m) => DASHBOARD_MODULES.some((known) => known.id === m.id))}
          <p class="form-hint">{t('settings.layoutEmpty')}</p>
        {/if}
      </div>
    </div>
  </div>
</section>
