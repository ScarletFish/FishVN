<script>
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { pickExecutable } from '../lib/pick-path.js';
  import { report } from '../lib/frontend-log.js';
  import { toast } from '../lib/toast.svelte.js';
  import { formatBytes } from '../lib/format.js';
  import PathField from './PathField.svelte';
  import {
    draftSize,
    hasInstallation,
    installPathInvalid,
    newInstallation,
    splitLaunchPath,
  } from '../lib/installations.js';

  let {
    items = $bindable([]),
    defaultKey = $bindable(null),
    revealAll = false,
    disabled = false,
  } = $props();

  let picking = $state(false);
  let touched = $state({});

  function pathError(item) {
    if (!(revealAll || touched[item.key])) return '';
    return installPathInvalid(item) ? t('gameForm.install.pathInvalid') : '';
  }

  function markTouched(key) {
    if (touched[key]) return;
    touched = { ...touched, [key]: true };
  }

  function taken(path, ignoreKey = null) {
    return hasInstallation(items, newInstallation(path), ignoreKey);
  }

  const sizing = new Set();
  const sizeFailed = new Set();

  async function loadSize(item) {
    if (!item.dirPath) return;
    if (sizing.has(item.key) || sizeFailed.has(item.key)) return;
    sizing.add(item.key);
    try {
      const size = await api.dirSize(item.dirPath);
      const current = items.find((it) => it.key === item.key);
      if (!current || current.dirPath !== item.dirPath) return;
      items = items.map((it) => (it.key === item.key ? { ...it, sizeBytes: size } : it));
    } catch (err) {
      sizeFailed.add(item.key);
      report('warn', `统计目录大小失败 ${item.dirPath}：${err}`);
    } finally {
      sizing.delete(item.key);
    }
  }

  function applyPath(item, picked) {
    const { dirPath, entryPath } = splitLaunchPath((picked ?? '').trim());
    if (!dirPath) return;
    if (dirPath === item.dirPath && entryPath === item.entryPath) return;
    sizeFailed.delete(item.key);
    const next = { ...item, dirPath, entryPath, sizeBytes: null };
    items = items.map((it) => (it.key === item.key ? next : it));
    loadSize(next);
  }

  async function addItem() {
    if (disabled || picking) return;
    picking = true;
    try {
      const picked = await pickExecutable(t('gameForm.install.pickTitle'));
      if (!picked) return;
      if (taken(picked)) {
        toast(t('gameForm.install.duplicate'), 'error');
        return;
      }
      const added = newInstallation(picked);
      const { dirPath, entryPath } = splitLaunchPath(picked.trim());
      const next = { ...added, dirPath, entryPath };
      items = [...items, next];
      if (defaultKey == null) defaultKey = added.key;
      loadSize(next);
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      picking = false;
    }
  }

  function acceptPick(item, picked) {
    if (!taken(picked, item.key)) return true;
    toast(t('gameForm.install.duplicate'), 'error');
    return false;
  }

  function removeItem(item) {
    items = items.filter((entry) => entry.key !== item.key);
    if (defaultKey === item.key) defaultKey = items[0]?.key ?? null;
  }
</script>

<div class="form-group">
  <span class="form-label">{t('gameForm.install.label')}</span>

  <ul class="install-list">
    {#each items as item (item.key)}
      <li class="install-row">
        <label class="install-pick" data-tooltip={t('gameForm.install.pickDefault')}>
          <input
            type="radio"
            class="install-pick__input"
            name="install-default"
            checked={defaultKey === item.key}
            aria-label={t('gameForm.install.pickDefault')}
            {disabled}
            onchange={() => (defaultKey = item.key)}
          />
          <span class="install-pick__ring"></span>
        </label>

        <div class="install-main">
          <PathField
            bind:value={item.path}
            picker={pickExecutable}
            pickerTitle={t('gameForm.install.pickTitle')}
            onPick={(picked) => acceptPick(item, picked)}
            oncommit={(picked) => {
              markTouched(item.key);
              applyPath(item, picked);
            }}
            placeholder={t('gameForm.install.pathPlaceholder')}
            ariaLabel={t('gameForm.install.pathAria')}
            error={pathError(item)}
            suffix={draftSize(item) != null ? formatBytes(draftSize(item)) : ''}
            {disabled}
          />

          <div class="install-flags">
            <div class="install-flag-row">
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  checked={item.leLaunch}
                  onchange={(event) => (item.leLaunch = event.currentTarget.checked)}
                  {disabled}
                />
                <span class="toggle-slider"></span>
              </label>
              <span class="install-flag-label">{t('gameForm.install.leLaunch')}</span>
            </div>

            <div class="install-flag-row">
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  checked={item.magpie !== 'off'}
                  onchange={(event) =>
                    (item.magpie = event.currentTarget.checked ? 'full' : 'off')}
                  {disabled}
                />
                <span class="toggle-slider"></span>
              </label>
              <span class="install-flag-label">{t('gameForm.install.magpie')}</span>
              <div
                class="seg-radio-group install-magpie"
                role="radiogroup"
                aria-label={t('gameForm.install.magpie')}
              >
                <label class="seg-radio-item">
                  <input
                    type="radio"
                    name="install-magpie-{item.key}"
                    checked={item.magpie === 'full'}
                    onchange={() => (item.magpie = 'full')}
                    disabled={disabled || item.magpie === 'off'}
                  />
                  {t('gameForm.install.magpieFull')}
                </label>
                <label class="seg-radio-item">
                  <input
                    type="radio"
                    name="install-magpie-{item.key}"
                    checked={item.magpie === 'window'}
                    onchange={() => (item.magpie = 'window')}
                    disabled={disabled || item.magpie === 'off'}
                  />
                  {t('gameForm.install.magpieWindow')}
                </label>
              </div>
            </div>

            <input
              class="install-args"
              type="text"
              bind:value={item.launchArgs}
              placeholder={item.leLaunch
                ? t('gameForm.install.argsLeDisabled')
                : t('gameForm.install.argsPlaceholder')}
              aria-label={t('gameForm.install.argsAria')}
              disabled={disabled || item.leLaunch}
              spellcheck="false"
              autocomplete="off"
            />
          </div>
        </div>

        <button
          type="button"
          class="btn btn-danger install-remove"
          onclick={() => removeItem(item)}
          {disabled}
        >
          {t('gameForm.install.remove')}
        </button>
      </li>
    {/each}
  </ul>

  {#if items.length > 1}
    <p class="form-hint install-hint">{t('gameForm.install.hint')}</p>
  {/if}

  <button
    type="button"
    class="btn install-add"
    onclick={addItem}
    disabled={disabled || picking}
  >
    {t('gameForm.install.add')}
  </button>
</div>
