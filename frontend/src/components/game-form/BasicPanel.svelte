<script>
  import { t } from '../../lib/i18n.js';
  import { clampRating, cleanRating, toNumber } from '../../lib/form-metadata.js';
  import PlaytimeField from '../PlaytimeField.svelte';
  import SelectDd from '../SelectDd.svelte';

  let {
    name = $bindable(''),
    status = $bindable('wishlist'),
    statusOpen = $bindable(false),
    userRating = $bindable(null),
    userRatingDraft = $bindable(''),
    notes = $bindable(''),
    description = $bindable(''),
    error = '',
    saving = false,
    gameId = null,
    inputEl = $bindable(null),
  } = $props();

  const STATUS_KEYS = ['playing', 'wishlist', 'finished', 'dropped'];
  const STATUS_OPTIONS = STATUS_KEYS.map((key) => ({ value: key, label: t(`status.${key}`) }));

  function stepRating(delta) {
    const current = userRating ?? 0;
    const next = Math.round((current + delta) * 10) / 10;
    if (next <= 0) {
      userRating = delta < 0 ? null : 5;
    } else {
      userRating = Math.min(100, next);
    }
    userRatingDraft = userRating === null ? '' : String(userRating);
  }

  function onUserRatingInput(e) {
    const cleaned = cleanRating(e.target.value);
    e.target.value = cleaned;
    userRatingDraft = cleaned;
    userRating = cleaned === '' ? null : toNumber(cleaned);
  }

  function onUserRatingBlur(e) {
    if (userRating === null) {
      e.target.value = '';
      return;
    }
    userRating = clampRating(userRating);
    userRatingDraft = String(userRating);
    e.target.value = userRatingDraft;
  }
</script>

<div class="panel-grid">
  <div class="form-group panel-grid__full">
    <label for="game-form-name">{t('gameForm.field.name')}</label>
    <input
      id="game-form-name"
      type="text"
      bind:value={name}
      bind:this={inputEl}
      placeholder={t('gameForm.namePlaceholder')}
      disabled={saving}
    />
    {#if error}
      <p class="form-hint form-hint--error">{error}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-status">{t('gameForm.field.status')}</label>
    <SelectDd
      id="game-form-status"
      options={STATUS_OPTIONS}
      value={status}
      open={statusOpen}
      onopenchange={(v) => (statusOpen = v)}
      onchange={(v) => (status = v)}
      disabled={saving}
    />
  </div>

  <div class="form-group">
    <label for="game-form-rating">{t('gameForm.field.userRating')}</label>
    <div class="num-stepper">
      <button
        type="button"
        class="num-stepper-btn"
        onclick={() => stepRating(-5)}
        disabled={saving}
        aria-label={t('gameForm.action.ratingDown')}
      >−</button>
      <input
        id="game-form-rating"
        class="num-stepper-val"
        type="text"
        inputmode="decimal"
        value={userRatingDraft}
        placeholder={t('gameForm.placeholder.rating')}
        oninput={onUserRatingInput}
        onblur={onUserRatingBlur}
        disabled={saving}
        aria-label={t('gameForm.field.userRating')}
      />
      <button
        type="button"
        class="num-stepper-btn"
        onclick={() => stepRating(5)}
        disabled={saving}
        aria-label={t('gameForm.action.ratingUp')}
      >+</button>
    </div>
    <p class="form-hint">{t('gameForm.hint.userRating')}</p>
  </div>

  <div class="form-group panel-grid__full form-group--notes">
    <label for="game-form-notes">{t('gameForm.field.notes')}</label>
    <textarea
      id="game-form-notes"
      class="scroll-thin"
      bind:value={notes}
      placeholder={t('gameForm.placeholder.notes')}
      disabled={saving}
    ></textarea>
  </div>

  <div class="form-group panel-grid__full form-group--tall">
    <label for="game-form-description">{t('gameForm.field.description')}</label>
    <textarea
      id="game-form-description"
      class="scroll-thin"
      bind:value={description}
      placeholder={t('gameForm.placeholder.description')}
      disabled={saving}
    ></textarea>
  </div>

  {#if gameId}
    <div class="panel-grid__full">
      <PlaytimeField {gameId} disabled={saving} />
    </div>
  {/if}
</div>
