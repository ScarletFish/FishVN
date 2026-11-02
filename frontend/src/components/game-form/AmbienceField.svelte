<script>
  import { t } from '../../lib/i18n.js';
  import { coverSrc } from '../../lib/cover-src.js';
  import { pickImage } from '../../lib/pick-path.js';
  import { BANNER_ASPECT, BANNER_FOCAL } from '../../lib/cover-focal.js';
  import CoverFocalDialog from './CoverFocalDialog.svelte';

  let {
    path = $bindable(''),
    mode = $bindable('soft'),
    plate = $bindable(false),
    focalX = $bindable(50),
    focalY = $bindable(22),
    coverPath = '',
    title = '',
    alias = '',
    saving = false,
  } = $props();

  let coverUrl = $state(null);
  let customUrl = $state(null);
  let broken = $state(false);
  let pickError = $state('');
  let focalOpen = $state(false);

  const custom = $derived(Boolean(path.trim()));
  const plateOn = $derived(mode === 'clear' && plate);

  function load(source, assign) {
    const next = (source ?? '').trim();
    if (!next) {
      assign(null);
      return;
    }
    let live = true;
    coverSrc(next)
      .then((url) => {
        if (live) assign(url);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }

  $effect(() => load(coverPath, (url) => (coverUrl = url)));
  $effect(() => load(path, (url) => (customUrl = url)));

  const previewUrl = $derived(custom ? customUrl : coverUrl);
  const previewOk = $derived(Boolean(previewUrl) && !broken);

  $effect(() => {
    if (previewUrl) broken = false;
  });

  async function chooseImage() {
    pickError = '';
    try {
      const picked = await pickImage(t('gameForm.field.ambience'));
      if (!picked) return false;
      path = picked;
      focalX = BANNER_FOCAL.x;
      focalY = BANNER_FOCAL.y;
      return true;
    } catch (err) {
      pickError = err.message;
      return false;
    }
  }

  async function chooseSource(next, input) {
    if (next === 'follow') {
      path = '';
      return;
    }
    if (path.trim()) return;
    if (!(await chooseImage())) input.checked = false;
  }
</script>

<div class="form-group">
  <span class="form-label">{t('gameForm.field.ambience')}</span>

  <div class="amb-field">
    <button
      type="button"
      class="amb-preview"
      data-mode={mode}
      class:is-plate={plateOn}
      disabled={saving || !previewOk}
      onclick={() => (focalOpen = true)}
    >
      <div class="amb-preview__bg">
        {#if previewOk}
          <img
            src={previewUrl}
            alt=""
            style:object-position="{focalX}% {focalY}%"
            onerror={() => (broken = true)}
          />
        {/if}
      </div>
      <div class="amb-preview__text">
        <span class="amb-preview__title" class:is-plate={plateOn}>
          {title.trim() || t('gameForm.ambience.sample')}
        </span>
        <span class="amb-preview__alias" class:is-plate={plateOn}>
          {alias.trim() || t('gameForm.ambience.sampleAlias')}
        </span>
      </div>
      <span class="media-badge">{t('gameForm.action.adjustFocal')}</span>
    </button>

    <div class="amb-field__controls">
      <div class="amb-field__row">
        <span class="amb-field__label">{t('gameForm.ambience.source')}</span>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('gameForm.ambience.source')}>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="game-form-ambience-source"
              checked={!custom}
              onchange={() => chooseSource('follow')}
              disabled={saving}
            />
            {t('gameForm.ambience.follow')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="game-form-ambience-source"
              checked={custom}
              onchange={(e) => chooseSource('custom', e.currentTarget)}
              disabled={saving}
            />
            {t('gameForm.ambience.custom')}
          </label>
        </div>
        {#if custom}
          <button type="button" class="btn" onclick={chooseImage} disabled={saving}>
            {t('gameForm.action.replaceAmbience')}
          </button>
        {/if}
      </div>

      {#if pickError}
        <p class="form-hint form-hint--error">{pickError}</p>
      {/if}

      <div class="amb-field__row">
        <span class="amb-field__label">{t('gameForm.ambience.mode')}</span>
        <div class="seg-radio-group" role="radiogroup" aria-label={t('gameForm.ambience.mode')}>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="game-form-ambience-mode"
              checked={mode === 'soft'}
              onchange={() => (mode = 'soft')}
              disabled={saving}
            />
            {t('gameForm.ambience.soft')}
          </label>
          <label class="seg-radio-item">
            <input
              type="radio"
              name="game-form-ambience-mode"
              checked={mode === 'clear'}
              onchange={() => (mode = 'clear')}
              disabled={saving}
            />
            {t('gameForm.ambience.clear')}
          </label>
        </div>
      </div>

      {#if mode === 'clear'}
        <div class="amb-field__row">
          <span class="amb-field__label">{t('gameForm.field.ambiencePlate')}</span>
          <label class="toggle-switch">
            <input type="checkbox" bind:checked={plate} disabled={saving} />
            <span class="toggle-slider" aria-hidden="true"></span>
          </label>
        </div>
      {/if}
    </div>
  </div>
</div>

<CoverFocalDialog
  bind:open={focalOpen}
  imagePath={custom ? path : coverPath}
  {focalX}
  {focalY}
  aspect={BANNER_ASPECT}
  aspectLabel="3.58:1"
  oncommit={(x, y) => {
    focalX = x;
    focalY = y;
  }}
  onclose={() => (focalOpen = false)}
/>
