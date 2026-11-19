<script>
  import { onMount } from 'svelte';
  import Icon from '../../components/Icon.svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { check } from '@tauri-apps/plugin-updater';
  import { openExternal } from '../../lib/open-external.js';
  import { toast } from '../../lib/toast.svelte.js';
  import { t } from '../../lib/i18n.js';

  const GITHUB_REPO_URL = 'https://github.com/ScarletFish/FishVN';

  let appVersion = $state('');
  let updateState = $state('idle');
  let updateVersion = $state('');
  let updateProgress = $state(0);
  let pendingUpdate = null;

  onMount(() => {
    getVersion()
      .then((v) => (appVersion = v))
      .catch(() => {});
  });

  async function checkUpdate() {
    if (updateState === 'checking' || updateState === 'downloading') return;
    updateState = 'checking';
    updateProgress = 0;
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateVersion = update.version;
        updateState = 'available';
      } else {
        pendingUpdate = null;
        updateState = 'uptodate';
      }
    } catch (err) {
      pendingUpdate = null;
      updateState = 'error';
      toast(t('settings.about.checkFailed'), 'error');
    }
  }

  async function installUpdate() {
    if (!pendingUpdate || updateState !== 'available') return;
    updateState = 'downloading';
    updateProgress = 0;
    let downloaded = 0;
    let contentLength = 0;
    try {
      await pendingUpdate.downloadAndInstall((event) => {
        if (event.event === 'Started') contentLength = event.data.contentLength ?? 0;
        else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          updateProgress = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;
        }
      });
    } catch (err) {
      updateState = 'error';
      toast(t('settings.about.installFailed'), 'error');
    }
  }
</script>

<section class="settings-section" id="settings-about">
  <h2 class="settings-section__title">{t('settings.tab.about')}</h2>

  <div class="about-hero">
    <img src="/favicon.svg" alt="" width="112" height="112" />
    <div class="about-hero-name">{t('settings.about.brand')}</div>
    <div class="about-hero-version">{appVersion ? `v${appVersion}` : '—'}</div>
  </div>

  <div class="form-group form-action-row">
    <div>
      <div class="form-label">{t('settings.about.update')}</div>
      <p class="form-hint">{t('settings.about.updateHint')}</p>
    </div>
    <div class="about-actions">
      <button type="button" class="btn" onclick={() => openExternal(GITHUB_REPO_URL)}>
        {t('settings.about.repoOpen')}
        <Icon name="arrow-up-right" size="sm" />
      </button>
      {#if updateState === 'available'}
        <button type="button" class="btn btn-primary" onclick={installUpdate}>
          {t('settings.about.install', { version: updateVersion })}
        </button>
      {:else}
        <button
          type="button"
          class="btn btn-primary"
          disabled={updateState === 'checking' || updateState === 'downloading'}
          onclick={checkUpdate}
        >
          {#if updateState === 'checking' || updateState === 'downloading'}
            <span class="btn-spinner" aria-hidden="true"></span>
          {/if}
          {t('settings.about.check')}
        </button>
      {/if}
    </div>
  </div>

  {#if updateState !== 'idle'}
    <div class="form-group form-group--flat">
      <p class="form-hint">
        <i
          class="status-dot"
          class:status-dot--ok={updateState === 'uptodate'}
          class:status-dot--invalid={updateState === 'error'}
          aria-hidden="true"
        ></i>
        {#if updateState === 'checking'}
          {t('settings.about.checking')}
        {:else if updateState === 'uptodate'}
          {t('settings.about.uptodate')}
        {:else if updateState === 'available'}
          {t('settings.about.available', { version: updateVersion })}
        {:else if updateState === 'downloading'}
          {t('settings.about.downloading', { percent: updateProgress })}
        {:else if updateState === 'error'}
          {t('settings.about.failed')}
        {/if}
      </p>
    </div>
  {/if}
</section>

<style>
  .about-actions {
    display: flex;
    flex-shrink: 0;
    gap: var(--space-2);
  }

  .about-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4) 0 var(--space-6);
    margin-bottom: var(--space-4);
  }

  .about-hero img {
    width: var(--space-28);
    height: var(--space-28);
  }

  .about-hero-name {
    font-size: var(--text-lg);
    font-weight: var(--fw-medium);
    color: var(--fg-default);
  }

  .about-hero-version {
    font-size: var(--text-sm);
    color: var(--fg-muted);
  }
</style>
