<script>
  import { onMount } from 'svelte';
  import Modal from '../../components/Modal.svelte';
  import TagPicker from '../../components/game-form/TagPicker.svelte';
  import { CHARACTER_SCOPES } from '../../lib/generated/character-roles.js';
  import { readConfig, saveConfig } from '../../lib/config-store.svelte.js';
  import { loadLibraryTags, toAllTagItems } from '../../lib/library-tags.svelte.js';
  import { tagKeyLabel } from '../../lib/tag-label.svelte.js';
  import { t } from '../../lib/i18n.js';

  const TAG_GROUP_ORDER = [
    'custom',
    'theme',
    'character',
    'setting',
    'plot',
    'style',
    'other',
    'tech',
  ];

  const config = readConfig();
  let poolOpen = $state(false);

  onMount(() => {
    poolOpen = false;
    loadLibraryTags();
  });

  function onHideR18Change(e) {
    saveConfig({ hide_r18_tags: e.target.checked });
  }

  function onSpoilerLevelChange(e) {
    saveConfig({ spoiler_level: e.target.value });
  }

  function onMetricChange(e) {
    saveConfig({ time_metric: e.target.value });
  }

  function onCharacterScopeChange(e) {
    saveConfig({ character_scope: e.target.value });
  }

  function saveExcludedTags(next) {
    saveConfig({ excluded_tags: next });
  }

  function toggleExcludedTag(key) {
    const current = config.excluded_tags;
    saveExcludedTags(
      current.includes(key) ? current.filter((item) => item !== key) : [...current, key],
    );
  }

  function removeExcludedTag(key) {
    saveExcludedTags(config.excluded_tags.filter((item) => item !== key));
  }
</script>

<section class="settings-section" id="settings-content">
  <h2 class="settings-section__title">{t('settings.tab.content')}</h2>

  <div class="settings-card">
    <div class="form-group">
      <div class="form-row">
        <span class="form-row-label">{t('settings.hideR18')}</span>
        <label class="toggle-switch" for="settings-hide-r18">
          <input
            id="settings-hide-r18"
            type="checkbox"
            aria-label={t('settings.hideR18')}
            checked={config.hide_r18_tags}
            onchange={onHideR18Change}
          />
          <span class="toggle-slider"></span>
        </label>
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
            checked={config.spoiler_level === 'hide_all'}
            onchange={onSpoilerLevelChange}
          />
          {t('settings.spoilerHideAll')}
        </label>
        <label class="seg-radio-item">
          <input
            type="radio"
            name="settings-spoiler-level"
            value="hide_major"
            checked={config.spoiler_level === 'hide_major'}
            onchange={onSpoilerLevelChange}
          />
          {t('settings.spoilerHideMajor')}
        </label>
        <label class="seg-radio-item">
          <input
            type="radio"
            name="settings-spoiler-level"
            value="show_all"
            checked={config.spoiler_level === 'show_all'}
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
          onclick={() => {
            loadLibraryTags();
            poolOpen = true;
          }}
        >
          {t('settings.excludedBrowse')}
        </button>
      </div>
      <p class="form-hint">{t('settings.excludedTagsHint')}</p>
      {#if config.excluded_tags.length > 0}
        <div class="excluded-chips">
          {#each config.excluded_tags as key (key)}
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

    <div class="form-group">
      <div class="form-label">{t('settings.characterScope.label')}</div>
      <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.characterScope.label')}>
        {#each CHARACTER_SCOPES as scope (scope.id)}
          <label class="seg-radio-item">
            <input
              type="radio"
              name="settings-character-scope"
              value={scope.id}
              checked={config.character_scope === scope.id}
              onchange={onCharacterScopeChange}
            />
            {t(`settings.characterScope.${scope.id}`)}
          </label>
        {/each}
      </div>
      <p class="form-hint">{t('settings.characterScope.hint')}</p>
    </div>

    <div class="form-group">
      <div class="form-label">{t('settings.timeMetric')}</div>
      <div class="seg-radio-group" role="radiogroup" aria-label={t('settings.timeMetric')}>
        <label class="seg-radio-item" data-tooltip={t('settings.timeForegroundDesc')}>
          <input
            type="radio"
            name="settings-time-metric"
            value="foreground"
            checked={config.time_metric === 'foreground'}
            onchange={onMetricChange}
          />
          {t('settings.timeForeground')}
        </label>
        <label class="seg-radio-item" data-tooltip={t('settings.timeAliveDesc')}>
          <input
            type="radio"
            name="settings-time-metric"
            value="alive"
            checked={config.time_metric === 'alive'}
            onchange={onMetricChange}
          />
          {t('settings.timeAlive')}
        </label>
      </div>
      <p class="form-hint">{t('settings.timeMetricHint')}</p>
    </div>
  </div>
</section>

<Modal bind:open={poolOpen} title={t('settings.excludedTags')} variant="modal--picker">
  <TagPicker
    items={toAllTagItems()}
    selected={config.excluded_tags}
    groupOrder={TAG_GROUP_ORDER}
    hiddenGroups={['tech']}
    emptyText={t('settings.excludedPoolEmpty')}
    onToggle={toggleExcludedTag}
    onClose={() => (poolOpen = false)}
  />
</Modal>
