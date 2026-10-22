<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import { getTheme, toggleTheme, getAccent, setAccent, viewTransition, ACCENTS, getFontScale, setFontScale } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';
  import { toast } from '../lib/toast.svelte.js';
  import { api } from '../lib/api.js';
  import { DASHBOARD_MODULES, moveModule, setModuleEnabled, reorderModule } from '../lib/dashboard-layout.js';
  import { notifyConfigChanged } from '../lib/config-events.js';
  import { getHideR18Tags, getSpoilerLevel, getExcludedTags } from '../lib/config-store.svelte.js';
  import { getLibraryTags, loadLibraryTags } from '../lib/library-tags.svelte.js';
  import { tagKeyLabel, tagLabel } from '../lib/tag-label.svelte.js';

  let { open = $bindable(false) } = $props();

  let tab = $state('appearance');
  let dark = $state(getTheme() === 'dark');
  let accent = $state(getAccent());
  let fontScale = $state(getFontScale());
  let timeMetric = $state('foreground');
  let hideR18 = $state(false);
  let spoilerLevel = $state('hide_all');
  let excludedTags = $state([]);
  let excludeQuery = $state('');
  let layout = $state([]);

  const SPOILER_LEVELS = ['hide_all', 'hide_major', 'show_all'];

  const excludedMatches = $derived.by(() => {
    const query = excludeQuery.trim().toLowerCase();
    const pool = new Set(excludedTags);
    return getLibraryTags()
      .filter((item) => !(item.id ? pool.has(item.id) : pool.has(item.tag)))
      .filter((item) => {
        if (!query) return true;
        const label = tagKeyLabel(item.id ?? item.tag);
        return label.toLowerCase().includes(query) || item.tag.toLowerCase().includes(query);
      });
  });

  $effect(() => {
    if (!open) return;
    dark = getTheme() === 'dark';
    accent = getAccent();
    fontScale = getFontScale();
    hideR18 = getHideR18Tags();
    spoilerLevel = getSpoilerLevel();
    excludedTags = getExcludedTags();
    loadLibraryTags();
    api
      .getConfig()
      .then((config) => {
        timeMetric = config.time_metric === 'alive' ? 'alive' : 'foreground';
        spoilerLevel = SPOILER_LEVELS.includes(config.spoiler_level) ? config.spoiler_level : 'hide_all';
        if (Array.isArray(config.excluded_tags)) excludedTags = config.excluded_tags;
        layout = config.dashboard_layout ?? [];
      })
      .catch(() => {});
  });

  async function saveExcludedTags(next) {
    excludedTags = next;
    try {
      await api.updateConfig({ excluded_tags: next });
      notifyConfigChanged();
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  function addExcludedTag(key) {
    if (excludedTags.includes(key)) return;
    excludeQuery = '';
    saveExcludedTags([...excludedTags, key]);
  }

  function removeExcludedTag(key) {
    saveExcludedTags(excludedTags.filter((item) => item !== key));
  }

  async function saveLayout(next) {
    layout = next;
    try {
      await api.updateConfig({ dashboard_layout: next });
      notifyConfigChanged();
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  let dragId = $state(null);
  let dragOverId = $state(null);

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
      saveLayout(reorderModule(layout, dragId, target));
    }
    dragId = null;
    dragOverId = null;
  }

  $effect(() => () => {
    window.removeEventListener('pointermove', onDragMove);
  });

  function moduleLabel(id) {
    const known = DASHBOARD_MODULES.find((m) => m.id === id);
    return known ? t(known.label) : id;
  }

  function onToggle() {
    const next = !dark;
    viewTransition(async () => {
      dark = toggleTheme() === 'dark';
      await tick();
    });
    api
      .updateConfig({ theme: next ? 'dark' : 'light' })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onAccentPick(name) {
    if (name === accent) return;
    viewTransition(async () => {
      accent = setAccent(name);
      await tick();
    });
    api
      .updateConfig({ accent: name })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onMetricChange(e) {
    const value = e.target.value === 'alive' ? 'alive' : 'foreground';
    timeMetric = value;
    api
      .updateConfig({ time_metric: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onHideR18Change(e) {
    const value = e.target.checked;
    hideR18 = value;
    api
      .updateConfig({ hide_r18_tags: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onSpoilerLevelChange(e) {
    const value = SPOILER_LEVELS.includes(e.target.value) ? e.target.value : 'hide_all';
    spoilerLevel = value;
    api
      .updateConfig({ spoiler_level: value })
      .then(notifyConfigChanged)
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
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'content'}
      aria-selected={tab === 'content'}
      onclick={() => (tab = 'content')}
    >
      {t('settings.tab.content')}
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

      <div class="form-group">
        <div class="form-label">{t('settings.dashboardLayout')}</div>
        <div class="layout-rows">
          {#each layout as module (module.id)}
            <div
              class="layout-row"
              class:dragging={dragId === module.id}
              class:drag-over={dragOverId === module.id}
              data-id={module.id}
            >
              <span
                class="layout-row-handle"
                role="button"
                tabindex="-1"
                aria-label={t('settings.layoutMoveUp')}
                onpointerdown={(e) => onHandleDown(e, module.id)}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><circle cx="9" cy="5" r="1.5"/><circle cx="15" cy="5" r="1.5"/><circle cx="9" cy="12" r="1.5"/><circle cx="15" cy="12" r="1.5"/><circle cx="9" cy="19" r="1.5"/><circle cx="15" cy="19" r="1.5"/></svg>
              </span>
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  checked={module.enabled}
                  aria-label={moduleLabel(module.id)}
                  onchange={(e) => saveLayout(setModuleEnabled(layout, module.id, e.target.checked))}
                />
                <span class="toggle-slider"></span>
              </label>
              <span class="layout-row-label">{moduleLabel(module.id)}</span>
              <span class="layout-row-actions">
                <button
                  type="button"
                  class="btn btn-ghost layout-move-btn"
                  data-tooltip={t('settings.layoutMoveUp')}
                  disabled={module.id === layout[0]?.id}
                  aria-label={t('settings.layoutMoveUp')}
                  onclick={() => saveLayout(moveModule(layout, module.id, -1))}
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="18 15 12 9 6 15"></polyline></svg>
                </button>
                <button
                  type="button"
                  class="btn btn-ghost layout-move-btn"
                  data-tooltip={t('settings.layoutMoveDown')}
                  disabled={module.id === layout[layout.length - 1]?.id}
                  aria-label={t('settings.layoutMoveDown')}
                  onclick={() => saveLayout(moveModule(layout, module.id, 1))}
                >
                  <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                </button>
              </span>
            </div>
          {/each}
          {#if !layout.some((m) => DASHBOARD_MODULES.some((known) => known.id === m.id))}
            <p class="form-hint">{t('settings.layoutEmpty')}</p>
          {/if}
        </div>
      </div>
    </div>

    <div class="modal-panel" class:active={tab === 'content'}>
      <div class="form-group">
        <div class="form-row">
          <label class="toggle-switch" for="settings-hide-r18">
            <input id="settings-hide-r18" type="checkbox" checked={hideR18} onchange={onHideR18Change} />
            <span class="toggle-slider"></span>
          </label>
          <span class="form-row-label">{t('settings.hideR18')}</span>
        </div>
        <p class="form-hint">{t('settings.hideR18Hint')}</p>
      </div>

      <div class="form-group">
        <div class="form-label">{t('settings.spoilerLevel')}</div>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.spoilerLevel')}>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="settings-spoiler-level"
              value="hide_all"
              checked={spoilerLevel === 'hide_all'}
              onchange={onSpoilerLevelChange}
            />
            {t('settings.spoilerHideAll')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="settings-spoiler-level"
              value="hide_major"
              checked={spoilerLevel === 'hide_major'}
              onchange={onSpoilerLevelChange}
            />
            {t('settings.spoilerHideMajor')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="settings-spoiler-level"
              value="show_all"
              checked={spoilerLevel === 'show_all'}
              onchange={onSpoilerLevelChange}
            />
            {t('settings.spoilerShowAll')}
          </label>
        </div>
        <p class="form-hint">{t('settings.spoilerLevelHint')}</p>
      </div>

      <div class="form-group">
        <div class="form-label">{t('settings.excludedTags')}</div>
        <p class="form-hint">{t('settings.excludedTagsHint')}</p>
        {#if excludedTags.length > 0}
          <div class="excluded-chips">
            {#each excludedTags as key (key)}
              <span class="excluded-chip">
                {tagKeyLabel(key)}
                <button
                  type="button"
                  class="excluded-chip-remove"
                  aria-label={t('settings.excludedRemove')}
                  onclick={() => removeExcludedTag(key)}
                >
                  ×
                </button>
              </span>
            {/each}
          </div>
        {:else}
          <p class="form-hint">{t('settings.excludedEmpty')}</p>
        {/if}
        <div class="excluded-add">
          <input
            type="text"
            bind:value={excludeQuery}
            placeholder={t('settings.excludedSearch')}
            autocomplete="off"
          />
          {#if excludedMatches.length > 0}
            <ul class="excluded-matches">
              {#each excludedMatches as item (item.id ?? item.tag)}
                <li>
                  <button type="button" onclick={() => addExcludedTag(item.id ?? item.tag)}>
                    <span>{tagLabel(item)}</span>
                    <span class="excluded-match-count">×{item.count}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {:else if excludeQuery.trim()}
            <p class="form-hint">{t('settings.excludedNoMatch')}</p>
          {/if}
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
