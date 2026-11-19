<script>
  import { onMount } from 'svelte';
  import SelectDd from '../../components/SelectDd.svelte';
  import { api } from '../../lib/api.js';
  import { readConfig, saveConfig } from '../../lib/config-store.svelte.js';
  import { confirm } from '../../lib/confirm.svelte.js';
  import { formatBytes } from '../../lib/format.js';
  import { notifyLibraryChanged } from '../../lib/library-events.js';
  import { toast } from '../../lib/toast.svelte.js';
  import { t } from '../../lib/i18n.js';

  const RETENTION_OPTIONS = [
    { value: 'off', label: t('settings.backup.retentionOff') },
    { value: '7', label: t('settings.backup.retentionDays', { days: 7 }) },
    { value: '30', label: t('settings.backup.retentionDays', { days: 30 }) },
    { value: '90', label: t('settings.backup.retentionDays', { days: 90 }) },
    { value: '180', label: t('settings.backup.retentionDays', { days: 180 }) },
  ];

  const config = readConfig();

  let backups = $state([]);
  let backupBusy = $state(false);
  let retentionOpen = $state(false);

  const retention = $derived(
    config.backup_retention_days == null ? 'off' : String(config.backup_retention_days),
  );

  onMount(() => {
    loadBackups();
  });

  function onRetentionChange(raw) {
    saveConfig({ backup_retention_days: raw === 'off' ? null : Number(raw) });
  }

  function formatBackupTime(createdAt) {
    return new Date(createdAt * 1000).toLocaleString();
  }

  async function loadBackups() {
    try {
      backups = await api.listBackups();
    } catch (err) {
      toast(err.message, 'error');
    }
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
</script>

<section class="settings-section" id="settings-backup">
  <h2 class="settings-section__title">{t('settings.tab.backup')}</h2>

  <div class="settings-card">
    <div class="form-group">
      <div class="form-label">{t('settings.backup.retention')}</div>
      <SelectDd
        id="backup-retention"
        options={RETENTION_OPTIONS}
        value={retention}
        open={retentionOpen}
        onopenchange={(v) => (retentionOpen = v)}
        onchange={onRetentionChange}
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
                  {t('settings.backup.size')} {formatBytes(meta.size_bytes)} · {t(
                    'settings.backup.covers',
                    { count: meta.cover_count },
                  )}
                </div>
              </div>
              <div class="backup-actions">
                <button type="button" class="btn btn-ghost" onclick={() => onRestoreBackup(meta)}>
                  {t('settings.backup.restore')}
                </button>
                <button type="button" class="btn btn-ghost" onclick={() => onDeleteBackup(meta)}>
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
</section>
