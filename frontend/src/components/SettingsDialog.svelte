<script>
  import { tick } from 'svelte';
  import Modal from './Modal.svelte';
  import PathField from './PathField.svelte';
  import {
    getThemeMode,
    setTheme,
    getAccent,
    setAccent,
    viewTransition,
    ACCENTS,
    THEMES,
    getFontScale,
    setFontScale,
  } from '../lib/theme.js';
  import { t } from '../lib/i18n.js';
  import { panelScrollbars } from '../lib/scrollbars.js';
  import { toast } from '../lib/toast.svelte.js';
  import { api } from '../lib/api.js';
  import { pickExecutable } from '../lib/pick-path.js';
  import { DASHBOARD_MODULES, moveModule, setModuleEnabled, reorderModule } from '../lib/dashboard-layout.js';
  import { notifyConfigChanged } from '../lib/config-events.js';
  import { getHideR18Tags, getSpoilerLevel, getExcludedTags } from '../lib/config-store.svelte.js';
  import { loadLibraryTags } from '../lib/library-tags.svelte.js';
  import { tagKeyLabel } from '../lib/tag-label.svelte.js';
  import TagPicker from './game-form/TagPicker.svelte';
  import SyncResultDialog from './SyncResultDialog.svelte';
  import SelectDd from './SelectDd.svelte';
  import { confirm } from '../lib/confirm.svelte.js';
  import { notifyBackfillRefresh } from '../lib/task-events.js';
  import { notifyLibraryChanged } from '../lib/library-events.js';
  import { openExternal } from '../lib/open-external.js';

  let { open = $bindable(false) } = $props();

  let tab = $state('appearance');
  let themeMode = $state(getThemeMode());
  let accent = $state(getAccent());
  let fontScale = $state(getFontScale());
  let timeMetric = $state('foreground');
  let hideR18 = $state(false);
  let spoilerLevel = $state('hide_all');
  let excludedTags = $state([]);
  let poolOpen = $state(false);
  let layout = $state([]);
  let includeUninstalled = $state(true);

  let magpiePath = $state('');
  let lePath = $state('');
  let magpiePathError = $state('');
  let lePathError = $state('');

  let vndbSyncMode = $state('download');
  let vndbSyncWishlist = $state(false);
  let vndbSyncRating = $state(true);
  let hasVndbToken = $state(false);
  let vndbTokenInput = $state('');
  let vndbTesting = $state(false);
  let vndbTestResult = $state(null);
  let vndbSyncing = $state(false);
  let syncPlan = $state(null);
  let syncError = $state(null);
  let syncEmpty = $state(false);
  let syncSeq = 0;
  let syncActive = 0;
  let resultOpen = $state(false);
  let quitBehavior = $state('quit');

  let backupRetentionMode = $state('off');
  let backupRetentionOpen = $state(false);
  let backups = $state([]);
  let backupBusy = $state(false);

  const RETENTION_OPTIONS = [
    { value: 'off', label: t('settings.backup.retentionOff') },
    { value: '7', label: t('settings.backup.retentionDays', { days: 7 }) },
    { value: '30', label: t('settings.backup.retentionDays', { days: 30 }) },
    { value: '90', label: t('settings.backup.retentionDays', { days: 90 }) },
    { value: '180', label: t('settings.backup.retentionDays', { days: 180 }) },
  ];

  let tokenStatus = $derived.by(() => {
    if (vndbTesting) return { kind: 'checking', text: t('settings.vndb.testing') };
    if (vndbTokenInput.trim()) return { kind: 'pending', text: t('settings.vndb.tokenPending') };
    if (vndbTestResult === true) return { kind: 'ok', text: t('settings.vndb.testOk') };
    if (vndbTestResult === false) return { kind: 'invalid', text: t('settings.vndb.testInvalid') };
    if (hasVndbToken) return { kind: 'saved', text: t('settings.vndb.tokenSet') };
    return { kind: 'unset', text: t('settings.vndb.tokenUnset') };
  });

  const tokenAction = $derived.by(() => {
    if (vndbTokenInput.trim()) return { label: t('settings.vndb.save'), disabled: vndbTesting };
    return { label: t('settings.vndb.verify'), disabled: vndbTesting || !hasVndbToken };
  });

  const SPOILER_LEVELS = ['hide_all', 'hide_major', 'show_all'];
  const VNDB_MODES = ['download', 'upload', 'both'];
  const VNDB_TOKENS_URL = 'https://vndb.org/u/tokens';

  $effect(() => {
    if (!open) return;
    poolOpen = false;
    themeMode = getThemeMode();
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
        includeUninstalled = config.dashboard_include_uninstalled !== false;
        vndbSyncMode = config.vndb_sync_mode === 'upload' || config.vndb_sync_mode === 'both'
          ? config.vndb_sync_mode
          : 'download';
        vndbSyncWishlist = !!config.vndb_sync_wishlist;
        vndbSyncRating = config.vndb_sync_rating !== false;
        hasVndbToken = !!config.has_vndb_token;
        vndbTokenInput = '';
        vndbTestResult = null;
        magpiePath = config.magpie_path ?? '';
        lePath = config.le_path ?? '';
        magpiePathError = '';
        lePathError = '';
        quitBehavior = config.quit_behavior === 'minimize_to_tray' ? 'minimize_to_tray' : 'quit';
        backupRetentionMode = (config.backup_retention_days ?? null) == null
          ? 'off'
          : String(config.backup_retention_days);
        loadBackups();
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

  async function saveToolPath(field, raw) {
    const path = (raw ?? '').trim();
    const isMagpie = field === 'magpie';
    const key = isMagpie ? 'magpie_path' : 'le_path';
    if (isMagpie) magpiePathError = '';
    else lePathError = '';
    try {
      if (path && !(await api.pathExists(path))) {
        if (isMagpie) magpiePathError = t('settings.tools.pathMissing');
        else lePathError = t('settings.tools.pathMissing');
        return;
      }
      await api.updateConfig({ [key]: path || null });
      notifyConfigChanged();
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  function onIncludeUninstalledChange(e) {
    const value = e.target.checked;
    includeUninstalled = value;
    api
      .updateConfig({ dashboard_include_uninstalled: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
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

  function selectTab(next) {
    tab = next;
    poolOpen = false;
  }

  function onThemeChange(e) {
    const next = THEMES.includes(e.target.value) ? e.target.value : 'light';
    if (next === themeMode) return;
    themeMode = next;
    viewTransition(async () => {
      setTheme(next);
      await tick();
    });
    api
      .updateConfig({ theme: next })
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

  function onQuitBehaviorChange(e) {
    const value = e.target.value === 'minimize_to_tray' ? 'minimize_to_tray' : 'quit';
    quitBehavior = value;
    api
      .updateConfig({ quit_behavior: value })
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

  function onVndbSyncWishlistChange(e) {
    const value = e.target.checked;
    vndbSyncWishlist = value;
    api
      .updateConfig({ vndb_sync_wishlist: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  function onVndbSyncRatingChange(e) {
    const value = e.target.checked;
    vndbSyncRating = value;
    api
      .updateConfig({ vndb_sync_rating: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  async function onVndbTokenSubmit() {
    const typed = vndbTokenInput.trim();
    if (typed) {
      try {
        await api.updateConfig({ vndb_token: typed });
        notifyConfigChanged();
        hasVndbToken = true;
        vndbTokenInput = '';
      } catch (err) {
        toast(err.message, 'error');
        return;
      }
    }
    await checkVndbToken();
  }

  async function checkVndbToken() {
    vndbTesting = true;
    vndbTestResult = null;
    try {
      const ok = await api.testVndbToken('');
      vndbTestResult = ok ? true : false;
    } catch (err) {
      vndbTestResult = 'error';
      toast(err.message, 'error');
    } finally {
      vndbTesting = false;
    }
  }

  async function onVndbDisconnect() {
    try {
      await api.updateConfig({ vndb_token: '' });
      notifyConfigChanged();
      hasVndbToken = false;
      vndbTokenInput = '';
      vndbTestResult = null;
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  function formatBackupTime(createdAt) {
    return new Date(createdAt * 1000).toLocaleString();
  }

  function formatBackupSize(bytes) {
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  }

  async function loadBackups() {
    try {
      backups = await api.listBackups();
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  function onBackupRetentionChange(raw) {
    const value = raw === 'off' || raw == null ? null : Number(raw);
    api
      .updateConfig({ backup_retention_days: value })
      .then(notifyConfigChanged)
      .catch((err) => toast(err.message, 'error'));
  }

  async function onCreateBackup() {
    if (backupBusy) return;
    backupBusy = true;
    try {
      await api.createBackup();
      await loadBackups();
      toast(t('settings.backup.created'), 'success');
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      backupBusy = false;
    }
  }

  async function onRestoreBackup(meta) {
    const ok = await confirm('restoreBackup', { time: formatBackupTime(meta.created_at) });
    if (!ok) return;
    try {
      await api.restoreBackup(meta.id);
      await loadBackups();
      notifyLibraryChanged();
      toast(t('settings.backup.restored'), 'success');
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  async function onDeleteBackup(meta) {
    const ok = await confirm('deleteBackup', { time: formatBackupTime(meta.created_at) });
    if (!ok) return;
    try {
      await api.deleteBackup(meta.id);
      await loadBackups();
      toast(t('settings.backup.deleted'), 'success');
    } catch (err) {
      toast(err.message, 'error');
    }
  }

  async function onVndbSync() {
    if (!hasVndbToken) {
      toast(t('settings.vndb.syncNeedToken'), 'warn');
      return;
    }
    const seq = ++syncSeq;
    syncActive = seq;
    vndbSyncing = true;
    syncPlan = null;
    syncError = null;
    syncEmpty = false;
    resultOpen = true;
    try {
      const plan = await api.syncVndb();
      if (seq !== syncActive) return;
      const changed =
        plan.creates.length > 0 ||
        plan.updates.length > 0 ||
        plan.aligns.length > 0 ||
        plan.conflicts.length > 0 ||
        plan.skipped > 0 ||
        plan.warnings.length > 0;
      if (changed) {
        syncPlan = plan;
      } else {
        syncEmpty = true;
      }
    } catch (err) {
      if (seq !== syncActive) return;
      syncError = err.message;
    } finally {
      if (seq === syncSeq) vndbSyncing = false;
    }
  }

  function onSyncCancel() {
    syncActive = 0;
    resultOpen = false;
    vndbSyncing = false;
  }

  async function onPlanApply(decision) {
    try {
      const report = await api.applySyncPlan(decision);
      resultOpen = false;
      syncPlan = null;
      notifyConfigChanged();
      notifyLibraryChanged();
      toast(t('settings.vndb.applied'), 'success');
      const created = report?.created_ids ?? [];
      if (created.length === 0) return;
      const ok = await confirm('backfillNow', { count: created.length });
      if (!ok) return;
      await api.startBackfill(created);
      notifyBackfillRefresh();
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
      onclick={() => selectTab('appearance')}
    >
      {t('settings.tab.appearance')}
    </button>
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'content'}
      aria-selected={tab === 'content'}
      onclick={() => selectTab('content')}
    >
      {t('settings.tab.content')}
    </button>
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'tools'}
      aria-selected={tab === 'tools'}
      onclick={() => selectTab('tools')}
    >
      {t('settings.tab.tools')}
    </button>
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'vndb'}
      aria-selected={tab === 'vndb'}
      onclick={() => selectTab('vndb')}
    >
      {t('settings.vndb.tab')}
    </button>
    <button
      type="button"
      role="tab"
      class="modal-tab"
      class:active={tab === 'backup'}
      aria-selected={tab === 'backup'}
      onclick={() => selectTab('backup')}
    >
      {t('settings.tab.backup')}
    </button>
  </div>

  <div class="modal-panels" use:panelScrollbars>
    <div class="modal-panel" class:active={tab === 'appearance'}>
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
              <div class="layout-row-line">
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
              {#if module.id === 'local_library'}
                <div class="layout-subrow" class:disabled={!module.enabled}>
                  <label class="layout-subrow-line">
                    <span class="toggle-switch">
                      <input
                        type="checkbox"
                        checked={includeUninstalled}
                        disabled={!module.enabled}
                        aria-label={t('settings.dashboardIncludeUninstalled')}
                        onchange={onIncludeUninstalledChange}
                      />
                      <span class="toggle-slider"></span>
                    </span>
                    <span class="layout-subrow-label">{t('settings.dashboardIncludeUninstalled')}</span>
                  </label>
                  <p class="form-hint">{t('settings.dashboardIncludeUninstalledHint')}</p>
                </div>
              {/if}
            </div>
          {/each}
          {#if !layout.some((m) => DASHBOARD_MODULES.some((known) => known.id === m.id))}
            <p class="form-hint">{t('settings.layoutEmpty')}</p>
          {/if}
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
              checked={quitBehavior === 'quit'}
              onchange={onQuitBehaviorChange}
            />
            {t('settings.quitQuit')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="quit-behavior"
              value="minimize_to_tray"
              checked={quitBehavior === 'minimize_to_tray'}
              onchange={onQuitBehaviorChange}
            />
            {t('settings.quitTray')}
          </label>
        </div>
        <p class="form-hint">{t('settings.quitBehaviorHint')}</p>
      </div>
    </div>

    <div class="modal-panel" class:active={tab === 'content'}>
      <div class="form-group">
        <div class="form-row">
          <label class="toggle-switch" for="settings-hide-r18">
            <input
              id="settings-hide-r18"
              type="checkbox"
              aria-label={t('settings.hideR18')}
              checked={hideR18}
              onchange={onHideR18Change}
            />
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

    <div class="modal-panel" class:active={tab === 'tools'}>
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

      <div class="form-group form-group--flat">
        <div class="form-row">
          <label class="toggle-switch" for="settings-sync-wishlist">
            <input
              id="settings-sync-wishlist"
              type="checkbox"
              aria-label={t('settings.vndb.syncWishlist')}
              checked={vndbSyncWishlist}
              onchange={onVndbSyncWishlistChange}
            />
            <span class="toggle-slider"></span>
          </label>
          <span class="form-row-label">{t('settings.vndb.syncWishlist')}</span>
        </div>
        <p class="form-hint">{t('settings.vndb.syncWishlistHint')}</p>
      </div>

      <div class="form-group form-group--flat">
        <div class="form-row">
          <label class="toggle-switch" for="settings-sync-rating">
            <input
              id="settings-sync-rating"
              type="checkbox"
              aria-label={t('settings.vndb.syncRating')}
              checked={vndbSyncRating}
              onchange={onVndbSyncRatingChange}
            />
            <span class="toggle-slider"></span>
          </label>
          <span class="form-row-label">{t('settings.vndb.syncRating')}</span>
        </div>
        <p class="form-hint">{t('settings.vndb.syncRatingHint')}</p>
      </div>

      <div class="form-group">
        <div class="form-label">{t('settings.vndb.token')}</div>
        <div class="vndb-token-field">
          <input
            class="vndb-token-input"
            type="password"
            placeholder={t('settings.vndb.tokenPlaceholder')}
            bind:value={vndbTokenInput}
            oninput={() => (vndbTestResult = null)}
            onkeydown={(e) => {
              if (e.key === 'Enter' && !tokenAction.disabled) onVndbTokenSubmit();
            }}
            autocomplete="off"
            spellcheck="false"
          />
          <button
            type="button"
            class="vndb-token-field__action"
            disabled={tokenAction.disabled}
            onclick={onVndbTokenSubmit}
          >
            {tokenAction.label}
          </button>
        </div>
        <p class="form-hint">
          {t('settings.vndb.tokenHintPrefix')}{' '}<a
            class="form-hint-link"
            href={VNDB_TOKENS_URL}
            onclick={(e) => {
              e.preventDefault();
              openExternal(VNDB_TOKENS_URL);
            }}>{t('settings.vndb.tokenHintLink')}</a>{' '}{t('settings.vndb.tokenHintSuffix')}
        </p>
        <div class="vndb-token-status">
          <p
            class="vndb-token-status__text"
            class:vndb-test-ok={tokenStatus.kind === 'ok'}
            class:vndb-test-invalid={tokenStatus.kind === 'invalid'}
          >
            <i
              class="status-dot"
              class:status-dot--ok={tokenStatus.kind === 'ok'}
              class:status-dot--invalid={tokenStatus.kind === 'invalid'}
              aria-hidden="true"
            ></i>{tokenStatus.text}
          </p>
          {#if hasVndbToken}
            <button type="button" class="vndb-token-clear" onclick={onVndbDisconnect}>
              {t('settings.vndb.disconnect')}
            </button>
          {/if}
        </div>
      </div>

      <div class="form-group form-action-row">
        <div>
          <div class="form-label">{t('settings.vndb.syncSection')}</div>
          <p class="form-hint">{t('settings.vndb.syncHint')}</p>
        </div>
        <button
          type="button"
          class="btn btn-primary"
          disabled={!hasVndbToken || vndbSyncing}
          onclick={onVndbSync}
        >
          {#if vndbSyncing}<span class="btn-spinner" aria-hidden="true"></span>{/if}
          {vndbSyncing ? t('settings.vndb.syncing') : t('settings.vndb.syncNow')}
        </button>
      </div>
    </div>

    <div class="modal-panel" class:active={tab === 'backup'}>
      <div class="form-group">
        <div class="form-label">{t('settings.backup.retention')}</div>
        <SelectDd
          id="backup-retention"
          options={RETENTION_OPTIONS}
          value={backupRetentionMode}
          open={backupRetentionOpen}
          onopenchange={(v) => (backupRetentionOpen = v)}
          onchange={(v) => {
            backupRetentionMode = v;
            onBackupRetentionChange(v);
          }}
          ariaLabel={t('settings.backup.retention')}
        />
        <p class="form-hint">{t('settings.backup.retentionHint')}</p>
      </div>

      <div class="form-group form-group--flat">
        <div class="form-row">
          <div class="form-row-text">
            <div class="form-row-title">{t('settings.backup.createTitle')}</div>
            <div class="form-row-desc">{t('settings.backup.createDesc')}</div>
          </div>
          <button
            type="button"
            class="btn btn-primary"
            disabled={backupBusy}
            onclick={onCreateBackup}
          >
            {#if backupBusy}<span class="btn-spinner" aria-hidden="true"></span>{/if}
            {backupBusy ? t('settings.backup.creating') : t('settings.backup.create')}
          </button>
        </div>
      </div>

      <div class="form-group">
        {#if backups.length > 0}
          <div class="backup-list">
            {#each backups as meta (meta.id)}
              <div class="backup-row">
                <div class="backup-meta">
                  <div class="backup-time">{formatBackupTime(meta.created_at)}</div>
                  <div class="backup-sub">
                    {t('settings.backup.size')} {formatBackupSize(meta.size_bytes)} · {t('settings.backup.covers', { count: meta.cover_count })}
                  </div>
                </div>
                <div class="backup-actions">
                  <button
                    type="button"
                    class="btn btn-ghost"
                    onclick={() => onRestoreBackup(meta)}
                  >
                    {t('settings.backup.restore')}
                  </button>
                  <button
                    type="button"
                    class="btn btn-ghost"
                    onclick={() => onDeleteBackup(meta)}
                  >
                    {t('settings.backup.delete')}
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="form-hint">{t('settings.backup.listEmpty')}</p>
        {/if}
      </div>
    </div>
  </div>

  <SyncResultDialog
    bind:open={resultOpen}
    plan={syncPlan}
    loading={vndbSyncing}
    error={syncError}
    empty={syncEmpty}
    onapply={onPlanApply}
    oncancel={onSyncCancel}
    onretry={onVndbSync}
  />

  <div class="modal-actions">
    <button type="button" class="btn" onclick={() => (open = false)}>
      {t('common.close')}
    </button>
  </div>
</Modal>
