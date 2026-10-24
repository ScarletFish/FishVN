<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import { getTheme, toggleTheme, getAccent, setAccent, viewTransition, ACCENTS, getFontScale, setFontScale } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';
  import { api } from '../lib/api.js';
  import { DASHBOARD_MODULES, moveModule, setModuleEnabled, reorderModule } from '../lib/dashboard-layout.js';
  import { notifyConfigChanged } from '../lib/config-events.js';
  import { getHideR18Tags, getSpoilerLevel, getExcludedTags } from '../lib/config-store.svelte.js';
  import { loadLibraryTags } from '../lib/library-tags.svelte.js';
  import { tagKeyLabel } from '../lib/tag-label.svelte.js';
  import TagPicker from './game-form/TagPicker.svelte';
  import SyncResultDialog from './SyncResultDialog.svelte';

  let { open = $bindable(false) } = $props();

  let tab = $state('appearance');
  let dark = $state(getTheme() === 'dark');
  let accent = $state(getAccent());
  let fontScale = $state(getFontScale());
  let timeMetric = $state('foreground');
  let hideR18 = $state(false);
  let spoilerLevel = $state('hide_all');
  let excludedTags = $state([]);
  let poolOpen = $state(false);
  let layout = $state([]);

  let vndbSyncMode = $state('download');
  let vndbImportWishlist = $state(false);
  let hasVndbToken = $state(false);
  let vndbTokenInput = $state('');
  let vndbTesting = $state(false);
  let vndbTestResult = $state(null);
  let vndbSyncing = $state(false);
  let syncPlan = $state(null);
  let resultOpen = $state(false);

  let tokenStatus = $derived.by(() => {
    if (vndbTestResult === true) return { kind: 'ok', text: t('settings.vndb.testOk') };
    if (vndbTestResult === false) return { kind: 'invalid', text: t('settings.vndb.testInvalid') };
    if (vndbTokenInput.trim()) return null;
    if (hasVndbToken) return { kind: 'saved', text: t('settings.vndb.tokenSet') };
    return { kind: 'unsaved', text: t('settings.vndb.tokenUnset') };
  });

  $effect(() => {
    void vndbTokenInput;
    vndbTestResult = null;
  });

  const SPOILER_LEVELS = ['hide_all', 'hide_major', 'show_all'];
  const VNDB_MODES = ['download', 'upload', 'both'];

  $effect(() => {
    if (!open) return;
    poolOpen = false;
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
        vndbSyncMode = config.vndb_sync_mode === 'upload' || config.vndb_sync_mode === 'both'
          ? config.vndb_sync_mode
          : 'download';
        vndbImportWishlist = !!config.vndb_sync_import_wishlist;
        hasVndbToken = !!config.has_vndb_token;
        vndbTokenInput = '';
        vndbTestResult = null;
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
    saveExcludedTags([...excludedTags, key]);
  }

  function removeExcludedTag(key) {
    saveExcludedTags(excludedTags.filter((item) => item !== key));
  }

  function toggleExcludedTag(key) {
    if (excludedTags.includes(key)) removeExcludedTag(key);
    else addExcludedTag(key);
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

  function onVndbModeChange(e) {
    const value = VNDB_MODES.includes(e.target.value) ? e.target.value : 'download';
    vndbSyncMode = value;
    api
      .updateConfig({ vndb_sync_mode: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onVndbImportWishlistChange(e) {
    const value = e.target.checked;
    vndbImportWishlist = value;
    api
      .updateConfig({ vndb_sync_import_wishlist: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  async function onVndbTokenTest() {
    const token = vndbTokenInput.trim();
    if (!token && !hasVndbToken) {
      vndbTestResult = null;
      toast(t('settings.vndb.tokenMissing'), 'warn');
      return;
    }
    vndbTesting = true;
    vndbTestResult = null;
    try {
      const ok = await api.testVndbToken(token);
      vndbTestResult = ok ? true : false;
    } catch (err) {
      vndbTestResult = 'error';
      toast(err.message, 'error');
    } finally {
      vndbTesting = false;
    }
  }

  async function onVndbTokenSave() {
    const token = vndbTokenInput.trim();
    if (!token) {
      toast(t('settings.vndb.tokenUnset'), 'warn');
      return;
    }
    try {
      await api.updateConfig({ vndb_token: token });
      notifyConfigChanged();
      hasVndbToken = true;
      vndbTokenInput = '';
      vndbTestResult = null;
      toast(t('settings.vndb.tokenSet'), 'success');
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  async function onVndbDisconnect() {
    try {
      await api.updateConfig({ vndb_token: '' });
      notifyConfigChanged();
      hasVndbToken = false;
      vndbTokenInput = '';
      vndbTestResult = null;
      toast(t('settings.vndb.tokenUnset'), 'success');
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  async function onVndbSync() {
    if (!hasVndbToken) {
      toast(t('settings.vndb.syncNeedToken'), 'warn');
      return;
    }
    vndbSyncing = true;
    try {
      const plan = await api.syncVndb();
      const changed =
        plan.creates.length > 0 ||
        plan.updates.length > 0 ||
        plan.aligns.length > 0 ||
        plan.conflicts.length > 0 ||
        plan.skipped > 0 ||
        plan.warnings.length > 0;
      if (changed) {
        syncPlan = plan;
        resultOpen = true;
      } else {
        toast(t('settings.vndb.syncNothing'), 'success');
      }
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      vndbSyncing = false;
    }
  }

  async function onPlanApply(decision) {
    try {
      const report = await api.applySyncPlan(decision);
      resultOpen = false;
      syncPlan = null;
      notifyConfigChanged();
      toast(t('settings.vndb.applied'), 'success');
      if (decision.backfill && report?.created_ids?.length) {
        try {
          await api.startBackfill(report.created_ids);
        } catch {}
      }
    } catch (err) {
      toast(err.message, 'error');
    }
  }
</script>

<Modal
  bind:open
  title={t('settings.title')}
  variant="modal--settings"
  bind:asideOpen={poolOpen}
  asideTitle={t('settings.excludedTags')}
>
  {#snippet aside()}
    <TagPicker
      selected={excludedTags}
      includeHidden={false}
      emptyText={t('settings.excludedPoolEmpty')}
      ontoggle={toggleExcludedTag}
      onclose={() => (poolOpen = false)}
    />
  {/snippet}

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
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'vndb'}
      aria-selected={tab === 'vndb'}
      onclick={() => (tab = 'vndb')}
    >
      {t('settings.vndb.tab')}
    </button>
  </div>

  <div class="modal-panels" use:panelScrollbars>
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
        <div class="form-label-row">
          <div class="form-label">{t('settings.excludedTags')}</div>
          <button
            type="button"
            class="btn tag-field__browse"
            onclick={() => (poolOpen = true)}
          >
            {t('settings.excludedBrowse')}
          </button>
        </div>
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
      </div>
    </div>

    <div class="modal-panel" class:active={tab === 'vndb'}>
      <div class="form-group">
        <div class="form-label">{t('settings.vndb.mode')}</div>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.vndb.mode')}>
          <label class="seg-radio-item" data-tooltip={t('settings.vndb.downloadDesc')}>
            <input
              type="radio"
              name="settings-vndb-mode"
              value="download"
              checked={vndbSyncMode === 'download'}
              onchange={onVndbModeChange}
            />
            {t('settings.vndb.download')}
          </label>
          <label class="seg-radio-item" data-tooltip={t('settings.vndb.uploadDesc')}>
            <input
              type="radio"
              name="settings-vndb-mode"
              value="upload"
              checked={vndbSyncMode === 'upload'}
              onchange={onVndbModeChange}
            />
            {t('settings.vndb.upload')}
          </label>
          <label class="seg-radio-item" data-tooltip={t('settings.vndb.bothDesc')}>
            <input
              type="radio"
              name="settings-vndb-mode"
              value="both"
              checked={vndbSyncMode === 'both'}
              onchange={onVndbModeChange}
            />
            {t('settings.vndb.both')}
          </label>
        </div>
        <p class="form-hint">{t('settings.vndb.modeHint')}</p>
      </div>

      <div class="form-group">
        <div class="form-row">
          <label class="toggle-switch" for="settings-import-wishlist">
            <input
              id="settings-import-wishlist"
              type="checkbox"
              checked={vndbImportWishlist}
              onchange={onVndbImportWishlistChange}
            />
            <span class="toggle-slider"></span>
          </label>
          <span class="form-row-label">{t('settings.vndb.importWishlist')}</span>
        </div>
        <p class="form-hint">{t('settings.vndb.importWishlistHint')}</p>
      </div>

      <div class="form-group">
        <div class="form-label">{t('settings.vndb.token')}</div>
        <div class="vndb-token-field">
          <input
            class="vndb-token-input"
            type="password"
            placeholder={t('settings.vndb.tokenPlaceholder')}
            bind:value={vndbTokenInput}
            autocomplete="off"
            spellcheck="false"
          />
          <button
            type="button"
            class="vndb-token-field__test"
            disabled={vndbTesting || (!vndbTokenInput.trim() && !hasVndbToken)}
            onclick={onVndbTokenTest}
          >
            {vndbTesting ? t('settings.vndb.testing') : t('settings.vndb.test')}
          </button>
        </div>
        <p class="form-hint">{t('settings.vndb.tokenHint')}</p>
        {#if tokenStatus}
          <p
            class="form-hint"
            class:vndb-test-ok={tokenStatus.kind === 'ok'}
            class:vndb-test-invalid={tokenStatus.kind === 'invalid'}
          >
            <i
              class="status-dot"
              class:status-dot--ok={tokenStatus.kind === 'ok' || tokenStatus.kind === 'saved'}
              class:status-dot--invalid={tokenStatus.kind === 'invalid'}
              aria-hidden="true"
            ></i>{tokenStatus.text}
          </p>
        {/if}
        <div class="form-actions">
          <button
            type="button"
            class="btn btn-primary"
            disabled={!vndbTokenInput.trim()}
            onclick={onVndbTokenSave}
          >
            {t('settings.vndb.save')}
          </button>
          <button type="button" class="btn btn-ghost" disabled={!hasVndbToken} onclick={onVndbDisconnect}>
            {t('settings.vndb.disconnect')}
          </button>
        </div>
      </div>

      <div class="form-group">
        <div class="form-label">{t('settings.vndb.syncNow')}</div>
        <p class="form-hint">{t('settings.vndb.syncHint')}</p>
        <div class="form-actions">
          <button
            type="button"
            class="btn btn-primary"
            disabled={!hasVndbToken || vndbSyncing}
            onclick={onVndbSync}
          >
            {vndbSyncing ? t('settings.vndb.syncing') : t('settings.vndb.syncNow')}
          </button>
        </div>
      </div>
    </div>
  </div>

  <SyncResultDialog
    bind:open={resultOpen}
    plan={syncPlan}
    onapply={onPlanApply}
    oncancel={() => (resultOpen = false)}
  />

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => (open = false)}>
      {t('common.close')}
    </button>
  </div>
</Modal>
