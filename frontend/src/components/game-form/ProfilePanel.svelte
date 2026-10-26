<script>
  import { t } from '../../lib/i18n.js';

  let {
    alias = $bindable(''),
    released = $bindable(''),
    rating = $bindable(''),
    votecount = $bindable(''),
    lengthMinutes = $bindable(''),
    developers = $bindable(''),
    ratingError = '',
    votecountError = '',
    lengthMinutesError = '',
    saving = false,
  } = $props();
</script>

<div class="panel-grid">
  <div class="form-group panel-grid__full">
    <label for="game-form-meta-alias">{t('gameForm.field.alias')}</label>
    <input
      id="game-form-meta-alias"
      type="text"
      bind:value={alias}
      placeholder={t('gameForm.placeholder.alias')}
      disabled={saving}
    />
  </div>

  <div class="form-group">
    <label for="game-form-meta-released">{t('gameForm.field.released')}</label>
    <input
      id="game-form-meta-released"
      type="text"
      bind:value={released}
      placeholder={t('gameForm.placeholder.released')}
      disabled={saving}
    />
  </div>

  <div class="form-group">
    <label
      for="game-form-meta-rating"
      data-tooltip={t('gameForm.hint.ratingScale')}
    >
      {t('gameForm.field.rating')}
    </label>
    <input
      id="game-form-meta-rating"
      type="text"
      inputmode="decimal"
      bind:value={rating}
      placeholder={t('gameForm.placeholder.rating')}
      disabled={saving}
    />
    <p class="form-hint">{t('gameForm.hint.ratingScale')}</p>
    {#if ratingError}
      <p class="form-hint form-hint--error">{ratingError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-votecount">{t('gameForm.field.votecount')}</label>
    <input
      id="game-form-meta-votecount"
      type="text"
      inputmode="numeric"
      bind:value={votecount}
      placeholder={t('gameForm.placeholder.votecount')}
      disabled={saving}
    />
    {#if votecountError}
      <p class="form-hint form-hint--error">{votecountError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-length">{t('gameForm.field.lengthMinutes')}</label>
    <input
      id="game-form-meta-length"
      type="text"
      inputmode="numeric"
      bind:value={lengthMinutes}
      placeholder={t('gameForm.placeholder.lengthMinutes')}
      disabled={saving}
    />
    {#if lengthMinutesError}
      <p class="form-hint form-hint--error">{lengthMinutesError}</p>
    {/if}
  </div>

  <div class="form-group">
    <label for="game-form-meta-developers">{t('gameForm.field.developers')}</label>
    <input
      id="game-form-meta-developers"
      type="text"
      bind:value={developers}
      placeholder={t('gameForm.placeholder.developers')}
      disabled={saving}
    />
  </div>
</div>
