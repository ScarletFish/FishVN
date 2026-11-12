<script>
  import Icon from '../components/Icon.svelte';
  import { onMount, tick } from 'svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { toast } from '../lib/toast.svelte.js';
  import { LIBRARY_CHANGED, onLibraryItemUpdated } from '../lib/library-events.js';
  import DetailSection from '../components/DetailSection.svelte';

  let { gameId } = $props();

  let notes = $state('');
  let editing = $state(false);
  let draft = $state('');
  let saving = $state(false);
  let saved = $state(false);
  let open = $state(false);
  let clamped = $state(false);
  let bodyEl = $state(null);
  let areaEl = $state(null);
  let savedTimer = null;

  async function load() {
    try {
      const state = await api.getGameState(gameId);
      if (state) notes = state.notes ?? '';
    } catch {}
  }

  $effect(() => {
    notes = '';
    editing = false;
    open = false;
    load();
  });

  onMount(() => {
    const reload = () => {
      if (saving || editing) return;
      load();
    };
    const unwatch = onLibraryItemUpdated((id) => {
      if (id === gameId) reload();
    });
    window.addEventListener(LIBRARY_CHANGED, reload);
    return () => {
      unwatch();
      window.removeEventListener(LIBRARY_CHANGED, reload);
      if (savedTimer) clearTimeout(savedTimer);
    };
  });

  function measure(text) {
    const el = bodyEl;
    if (!el || editing || open || !text) return;
    clamped = el.scrollHeight > el.clientHeight + 2;
  }

  $effect(() => {
    measure(notes);
  });

  async function begin() {
    draft = notes;
    editing = true;
    await tick();
    areaEl?.focus();
  }

  async function commit() {
    if (!editing || saving) return;
    const next = draft.trim();
    if (next === notes.trim()) {
      editing = false;
      open = false;
      return;
    }
    saving = true;
    try {
      const state = await api.updateGameState(gameId, { notes: next || null });
      notes = state?.notes ?? '';
      editing = false;
      open = false;
      saved = true;
      if (savedTimer) clearTimeout(savedTimer);
      savedTimer = setTimeout(() => (saved = false), 1800);
    } catch (err) {
      toast(err.message, 'error');
    } finally {
      saving = false;
    }
  }

  function discard() {
    editing = false;
    open = false;
    draft = notes;
  }

  function onkeydown(event) {
    if (event.key === 'Escape') {
      event.preventDefault();
      discard();
      return;
    }
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      commit();
    }
  }
</script>

<DetailSection title={t('detail.notes')} class="detail-enter-notes">
  {#snippet action()}
    {#if saved}
      <span class="detail-notes-saved">{t('detail.notesSaved')}</span>
    {/if}
    {#if !editing}
      <button
        type="button"
        class="detail-notes-edit"
        data-tooltip={t('detail.notesEdit')}
        aria-label={t('detail.notesEdit')}
        onclick={begin}
      >
        <Icon name="pencil" />
      </button>
    {/if}
  {/snippet}

  {#if editing}
    <textarea
      class="detail-notes-input scroll-thin"
      bind:this={areaEl}
      bind:value={draft}
      rows="4"
      aria-label={t('detail.notes')}
      onblur={commit}
      onkeydown={onkeydown}
    ></textarea>
    <div class="detail-notes-actions">
      <button
        type="button"
        class="btn"
        onmousedown={(event) => event.preventDefault()}
        onclick={discard}
      >
        {t('common.cancel')}
      </button>
      <button
        type="button"
        class="btn btn-primary"
        onmousedown={(event) => event.preventDefault()}
        onclick={commit}
        disabled={saving}
      >
        {t('common.save')}
      </button>
    </div>
  {:else if notes}
    <div
      class="detail-notes-body scroll-thin"
      class:detail-notes-body--open={open}
      bind:this={bodyEl}
    >
      {notes}
    </div>
    {#if clamped || open}
      <button
        type="button"
        class="detail-notes-toggle"
        aria-expanded={open}
        onclick={() => (open = !open)}
      >
        {open ? t('detail.notesLess') : t('detail.notesMore')}
      </button>
    {/if}
  {:else}
    <button type="button" class="detail-notes-empty" onclick={begin}>
      {t('detail.notesEmpty')}
    </button>
  {/if}
</DetailSection>
