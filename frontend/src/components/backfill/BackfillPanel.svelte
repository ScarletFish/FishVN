<script>
  import { t } from '../../lib/i18n.js';
  import { api } from '../../lib/api.js';
  import { toast } from '../../lib/toast.svelte.js';
  import { notifyBackfillRefresh } from '../../lib/task-events.js';
  import { STATE_BADGE, STATE_LABEL } from '../../lib/backfill-labels.js';

  let { item, running = false, onretried, onclose } = $props();

  let busy = $state(false);

  const bound = $derived(item.hit_title ?? '');
  const year = $derived(item.hit_released ? item.hit_released.slice(0, 4) : '');
  const candidates = $derived(item.candidates ?? []);

  async function rebind(cand) {
    if (busy || running) return;
    busy = true;
    try {
      await api.updateGame(item.game_id, { vndb_id: cand.id });
      await api.startBackfill([item.game_id]);
      notifyBackfillRefresh();
      onretried?.();
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      busy = false;
    }
  }

  async function retry() {
    if (busy || running) return;
    busy = true;
    try {
      await api.startBackfill([item.game_id]);
      notifyBackfillRefresh();
      onretried?.();
    } catch (e) {
      toast(e.message, 'error');
    } finally {
      busy = false;
    }
  }

  function openVndb() {
    if (!item.vndb_id) return;
    const url = `https://vndb.org/${item.vndb_id}`;
    window.open(url, '_blank');
  }
</script>

<div class="bf-panel__head">
  <span class="bf-badge {STATE_BADGE[item.state]}">
    {t(STATE_LABEL[item.state])}
  </span>
  <button type="button" class="bf-panel__close" aria-label={t('common.close')} onclick={onclose}>
    ×
  </button>
</div>

<div class="bf-field">
  <span class="bf-field__label">{t('backfillPage.panelLocal')}</span>
  <span class="bf-field__value">{item.name}</span>
</div>

<div class="bf-field">
  <span class="bf-field__label">{t('backfillPage.panelBound')}</span>
  {#if bound || item.vndb_id}
    <span class="bf-field__value">
      {bound || item.vndb_id}
      {#if year}<span class="bf-field__muted">{year}</span>{/if}
    </span>
    {#if item.vndb_id}
      <button type="button" class="bf-link" onclick={openVndb}>
        {item.vndb_id} · {t('backfillPage.openVndb')}
      </button>
    {/if}
  {:else}
    <span class="bf-field__muted">—</span>
  {/if}
</div>

{#if item.state === 'ok'}
  <div class="bf-field">
    <span class="bf-field__label">{t('backfillPage.panelFields')}</span>
    {#if item.filled.length}
      <span class="bf-chips">
        {#each item.filled as f (f)}<span class="bf-chip">{f}</span>{/each}
      </span>
    {:else}
      <span class="bf-field__muted">{t('backfillPage.filledNone')}</span>
    {/if}
    {#if item.characters > 0}
      <span class="bf-field__muted">
        {t('backfillPage.characterCount', { count: item.characters })}
      </span>
    {/if}
  </div>
{:else if item.error}
  <div class="bf-field">
    <span class="bf-field__label">{t('backfillPage.panelError')}</span>
    <span class="bf-field__value bf-field__value--error">{item.error}</span>
  </div>
{/if}

{#if candidates.length}
  <div class="bf-field">
    <span class="bf-field__label">
      {t('backfillPage.panelCandidates')}
      <span class="bf-field__muted">{t('backfillPage.hits', { count: item.hit_count })}</span>
    </span>
    <ul class="bf-cands">
      {#each candidates as cand (cand.id)}
        <li>
          <button
            type="button"
            class="bf-cand"
            class:is-current={cand.id === item.vndb_id}
            disabled={busy || running}
            onclick={() => rebind(cand)}
          >
            <span class="bf-cand__id">{cand.id}</span>
            <span class="bf-cand__title">{cand.title}</span>
            {#if cand.released}<span class="bf-cand__year">{cand.released.slice(0, 4)}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<div class="bf-panel__actions">
  <button
    type="button"
    class="btn btn-ghost"
    disabled={busy || running || item.state === 'pending' || item.state === 'active'}
    onclick={retry}
  >
    {t('backfillPage.retryOne')}
  </button>
</div>
