<script>
  import { api } from '../lib/api.js';
  import { t } from '../lib/i18n.js';
  import { toast } from '../lib/toast.svelte.js';
  import { durationInput, parseDuration } from '../lib/playtime.js';

  let { gameId, disabled = false } = $props();

  let seconds = $state(0);
  let draft = $state('');
  let busy = $state(false);
  let playing = $state(false);

  const readout = $derived.by(() => {
    const typed = parseDuration(draft);
    return typed == null ? '' : `${typed} ${t('common.unitSecond')}`;
  });

  async function load() {
    const [state, active] = await Promise.all([api.getGameState(gameId), api.listActiveSessions()]);
    playing = active.some((session) => session.game_id === gameId);
    return state.play_seconds;
  }

  $effect(() => {
    let live = true;
    load()
      .then((value) => {
        if (!live) return;
        seconds = value;
        draft = durationInput(value);
      })
      .catch((err) => {
        if (live) toast(err.message, 'error');
      });
    return () => {
      live = false;
    };
  });

  async function save() {
    if (disabled || busy) return;
    const wanted = parseDuration(draft);
    if (wanted == null || wanted === seconds) {
      draft = durationInput(seconds);
      return;
    }

    const submitted = draft;
    busy = true;
    try {
      const state = await api.updateGameState(gameId, { play_seconds: wanted });
      seconds = state.play_seconds;
      if (draft === submitted) draft = durationInput(seconds);
    } catch (err) {
      toast(err.message, 'error');
      draft = durationInput(seconds);
    } finally {
      busy = false;
    }
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
      disabled={disabled || busy}
      oninput={(e) => (draft = e.currentTarget.value)}
      onblur={save}
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
