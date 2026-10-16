<script>
  import { t } from '../lib/i18n.js';
  import { pickExecutable } from '../lib/pick-path.js';
  import { toast } from '../lib/toast.svelte.js';
  import PathField from './PathField.svelte';
  import {
    hasInstallation,
    installPathInvalid,
    newInstallation,
  } from '../lib/installations.js';

  let { items = $bindable([]), defaultKey = $bindable(null), disabled = false } = $props();

  let picking = $state(false);

  function taken(path, ignoreKey = null) {
    return hasInstallation(items, newInstallation(path), ignoreKey);
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
      items = [...items, added];
      if (defaultKey == null) defaultKey = added.key;
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

  {#if items.length === 0}
    <p class="install-empty">{t('gameForm.install.empty')}</p>
  {/if}

  <ul class="install-list">
    {#each items as item (item.key)}
      <li class="install-row">
        <PathField
          bind:value={item.path}
          picker={pickExecutable}
          pickerTitle={t('gameForm.install.pickTitle')}
          onPick={(picked) => acceptPick(item, picked)}
          placeholder={t('gameForm.install.pathPlaceholder')}
          ariaLabel={t('gameForm.install.pathAria')}
          error={installPathInvalid(item) ? t('gameForm.install.pathInvalid') : ''}
          {disabled}
        />

        <label class="install-default">
          <input
            type="radio"
            class="install-default__input"
            name="install-default"
            checked={defaultKey === item.key}
            {disabled}
            onchange={() => (defaultKey = item.key)}
          />
          <span class="install-default__face">
            {defaultKey === item.key
              ? t('gameForm.install.default')
              : t('gameForm.install.setDefault')}
          </span>
        </label>

        <button
          type="button"
          class="btn btn-ghost install-row__remove"
          onclick={() => removeItem(item)}
          {disabled}
        >
          {t('gameForm.install.remove')}
        </button>
      </li>
    {/each}
  </ul>

  <button
    type="button"
    class="btn install-add"
    onclick={addItem}
    disabled={disabled || picking}
  >
    {t('gameForm.install.add')}
  </button>
</div>
