<script>
  import SyncResultDialog from '../../components/SyncResultDialog.svelte';
  import { api } from '../../lib/api.js';
  import { readConfig, saveConfig } from '../../lib/config-store.svelte.js';
  import { confirm } from '../../lib/confirm.svelte.js';
  import { notifyBackfillRefresh } from '../../lib/task-events.js';
  import { notifyLibraryChanged } from '../../lib/library-events.js';
  import { notifyConfigChanged } from '../../lib/config-events.js';
  import { openExternal } from '../../lib/open-external.js';
  import { toast } from '../../lib/toast.svelte.js';
  import { t } from '../../lib/i18n.js';

  const VNDB_TOKENS_URL = 'https://vndb.org/u/tokens';

  let { onnavigate } = $props();

  const config = readConfig();
  let tokenSaved = $state(false);
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

  const hasToken = $derived(tokenSaved || config.has_vndb_token);

  const tokenStatus = $derived.by(() => {
    if (vndbTesting) return { kind: 'checking', text: t('settings.vndb.testing') };
    if (vndbTokenInput.trim()) return { kind: 'pending', text: t('settings.vndb.tokenPending') };
    if (vndbTestResult === true) return { kind: 'ok', text: t('settings.vndb.testOk') };
    if (vndbTestResult === false) return { kind: 'invalid', text: t('settings.vndb.testInvalid') };
    if (hasToken) return { kind: 'saved', text: t('settings.vndb.tokenSet') };
    return { kind: 'unset', text: t('settings.vndb.tokenUnset') };
  });

  const tokenAction = $derived.by(() => {
    if (vndbTokenInput.trim()) return { label: t('settings.vndb.save'), disabled: vndbTesting };
    return { label: t('settings.vndb.verify'), disabled: vndbTesting || !hasToken };
  });

  function onVndbModeChange(e) {
    saveConfig({ vndb_sync_mode: e.target.value });
  }

  function onVndbSyncWishlistChange(e) {
    saveConfig({ vndb_sync_wishlist: e.target.checked });
  }

  function onVndbSyncRatingChange(e) {
    saveConfig({ vndb_sync_rating: e.target.checked });
  }

  async function onVndbTokenSubmit() {
    const typed = vndbTokenInput.trim();
    if (typed) {
      const saved = await saveConfig({ vndb_token: typed });
      if (!saved) return;
      tokenSaved = true;
      vndbTokenInput = '';
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
    const saved = await saveConfig({ vndb_token: '' });
    if (!saved) return;
    tokenSaved = false;
    vndbTokenInput = '';
    vndbTestResult = null;
  }

  async function onVndbSync() {
    if (!hasToken) {
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

  async function onPlanApplyCancel() {
    await api.cancelTask('vndb-sync-push');
  }

  async function onPlanApply(decision) {
    try {
      const report = await api.applySyncPlan(decision);
      resultOpen = false;
      syncPlan = null;
      if (report?.cancelled) {
        notifyLibraryChanged();
        toast(t('settings.vndb.applyCancelled', { count: report?.pushed ?? 0 }), 'info');
        return;
      }
      notifyConfigChanged();
      notifyLibraryChanged();
      toast(t('settings.vndb.applied'), 'success');
      const created = report?.created_ids ?? [];
      if (created.length === 0) return;
      const ok = await confirm('backfillNow', { count: created.length });
      if (!ok) return;
      await api.startBackfill(created);
      notifyBackfillRefresh();
      onnavigate?.('backfill');
    } catch (err) {
      toast(err.message, 'error');
    }
  }
</script>

<section class="settings-section" id="settings-vndb">
  <h2 class="settings-section__title">{t('settings.vndb.tab')}</h2>

  <div class="settings-card">
    <div class="form-group">
      <div class="form-label">{t('settings.vndb.mode')}</div>
      <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.vndb.mode')}>
        <label class="seg-radio-item" data-tooltip={t('settings.vndb.downloadDesc')}>
          <input
            type="radio"
            name="settings-vndb-mode"
            value="download"
            checked={config.vndb_sync_mode === 'download'}
            onchange={onVndbModeChange}
          />
          {t('settings.vndb.download')}
        </label>
        <label class="seg-radio-item" data-tooltip={t('settings.vndb.uploadDesc')}>
          <input
            type="radio"
            name="settings-vndb-mode"
            value="upload"
            checked={config.vndb_sync_mode === 'upload'}
            onchange={onVndbModeChange}
          />
          {t('settings.vndb.upload')}
        </label>
        <label class="seg-radio-item" data-tooltip={t('settings.vndb.bothDesc')}>
          <input
            type="radio"
            name="settings-vndb-mode"
            value="both"
            checked={config.vndb_sync_mode === 'both'}
            onchange={onVndbModeChange}
          />
          {t('settings.vndb.both')}
        </label>
      </div>
      <p class="form-hint">{t('settings.vndb.modeHint')}</p>
    </div>

    <div class="form-group form-group--flat">
      <div class="form-row">
        <span class="form-row-label">{t('settings.vndb.syncWishlist')}</span>
        <label class="toggle-switch" for="settings-sync-wishlist">
          <input
            id="settings-sync-wishlist"
            type="checkbox"
            aria-label={t('settings.vndb.syncWishlist')}
            checked={config.vndb_sync_wishlist}
            onchange={onVndbSyncWishlistChange}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="form-group form-group--flat">
      <div class="form-row">
        <span class="form-row-label">{t('settings.vndb.syncRating')}</span>
        <label class="toggle-switch" for="settings-sync-rating">
          <input
            id="settings-sync-rating"
            type="checkbox"
            aria-label={t('settings.vndb.syncRating')}
            checked={config.vndb_sync_rating}
            onchange={onVndbSyncRatingChange}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
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
          }}>{t('settings.vndb.tokenHintLink')}</a
        >{' '}{t('settings.vndb.tokenHintSuffix')}
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
        {#if hasToken}
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
        disabled={!hasToken || vndbSyncing}
        onclick={onVndbSync}
      >
        {#if vndbSyncing}<span class="btn-spinner" aria-hidden="true"></span>{/if}
        {vndbSyncing ? t('settings.vndb.syncing') : t('settings.vndb.syncNow')}
      </button>
    </div>
  </div>
</section>

<SyncResultDialog
  bind:open={resultOpen}
  plan={syncPlan}
  loading={vndbSyncing}
  error={syncError}
  empty={syncEmpty}
  onapply={onPlanApply}
  onapplycancel={onPlanApplyCancel}
  oncancel={onSyncCancel}
  onretry={onVndbSync}
/>
