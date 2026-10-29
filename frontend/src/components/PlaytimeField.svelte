<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { toast } from '../lib/toast.svelte.js';
  import { durationInput, parseDuration } from '../lib/playtime.js';

  let { gameId = null, seconds = $bindable(0), disabled = false } = $props();

  let draft = $state(durationInput(seconds));
  let focused = $state(false);
  let playing = $state(false);

  const readout = $derived.by(() => {
    const typed = parseDuration(draft);
    return typed == null ? '' : `${typed} ${t('common.unitSecond')}`;
  });

  // 值从外面来（打开表单时异步载入），正在输入时不去动用户的手
  $effect(() => {
    const next = durationInput(seconds);
    if (!focused) draft = next;
  });

  $effect(() => {
    if (!gameId) return;
    let live = true;
    api
      .listActiveSessions()
      .then((active) => {
        if (live) playing = active.some((session) => session.game_id === gameId);
      })
      .catch((err) => {
        if (live) toast(err.message, 'error');
      });
    return () => {
      live = false;
    };
  });

  /** 只写进表单草稿，随「保存」一起提交；认不出来的输入还原成原值 */
  function commit() {
    if (disabled) return;
    const wanted = parseDuration(draft);
    if (wanted == null) {
      draft = durationInput(seconds);
      return;
    }
    seconds = wanted;
    draft = durationInput(seconds);
  }
</script>

<div class="form-group">
  <label for="game-form-playtime">{t('gameForm.playtime.label')}</label>

  <div class="playtime-input-row">
    <input
      id="game-form-playtime"
      class="playtime-input"
      type="text"
      value={draft}
      placeholder={t('gameForm.playtime.placeholder')}
      aria-label={t('gameForm.playtime.label')}
      {disabled}
      oninput={(e) => (draft = e.currentTarget.value)}
      onfocus={() => (focused = true)}
      onblur={() => {
        focused = false;
        commit();
      }}
      onkeydown={(e) => {
        if (e.key !== 'Enter') return;
        e.preventDefault();
        e.currentTarget.blur();
      }}
    />
    <span class="playtime-readout">{readout}</span>
  </div>

  <p class="form-hint">{t('gameForm.playtime.hint')}</p>

  {#if playing}
    <p class="playtime-live">
      <span class="pill pill--info">{t('status.playing')}</span>
      {t('gameForm.playtime.live')}
    </p>
  {/if}
</div>
