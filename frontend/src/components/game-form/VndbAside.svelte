<script>
  import { t } from '../../lib/i18n.js';
  import { panelScrollbars } from '../../lib/scrollbars.js';

  let {
    search,
    pulling = false,
    syncProgress = null,
    syncNotice = '',
    syncCancelled = false,
    selectedId = null,
    onpick,
    oncancel,
    onclose,
  } = $props();

  const PHASE_KEYS = {
    meta: 'gameForm.search.phaseMeta',
    characters: 'gameForm.search.phaseCharacters',
    images: 'gameForm.search.phaseImages',
    cover: 'gameForm.search.phaseCover',
  };

  const pullText = $derived.by(() => {
    if (!syncProgress) return t('gameForm.search.pulling');
    const label = t(PHASE_KEYS[syncProgress.phase] ?? 'gameForm.search.pulling');
    if (syncProgress.total == null) return label;
    return `${label} ${syncProgress.done ?? 0}/${syncProgress.total}`;
  });

  const pullRatio = $derived(
    syncProgress?.total ? Math.min(1, (syncProgress.done ?? 0) / syncProgress.total) : null
  );
</script>

<div class="modal-panels" use:panelScrollbars>
  <div class="modal-panel active">
    {#if search.searching}
      <div class="vndb-progress" role="progressbar" aria-label={t('gameForm.search.loading')}>
        <span class="vndb-progress__bar"></span>
      </div>
    {:else if search.searchError}
      <p class="vndb-note vndb-note--error">{search.searchError}</p>
    {:else if search.hits.length === 0}
      <p class="vndb-note">{t('gameForm.search.empty')}</p>
    {:else}
      {#if pulling}
        <p class="vndb-note">{pullText}</p>
        <div
          class="vndb-progress"
          class:vndb-progress--determinate={pullRatio != null}
          role="progressbar"
          aria-label={pullText}
          aria-valuenow={pullRatio == null ? undefined : Math.round(pullRatio * 100)}
        >
          <span
            class="vndb-progress__bar"
            style={pullRatio == null ? undefined : `width: ${pullRatio * 100}%`}
          ></span>
        </div>
        {#if syncNotice}
          <p class="vndb-note vndb-note--error">{syncNotice}</p>
        {/if}
        <button type="button" class="btn vndb-pull-cancel" onclick={oncancel} disabled={syncCancelled}>
          {t('common.cancel')}
        </button>
      {/if}
      <ul class="vndb-hits" class:is-pulling={pulling}>
        {#each search.hits as hit (hit.id)}
          <li>
            <button
              type="button"
              class="vndb-hit"
              class:active={selectedId === hit.id}
              disabled={pulling}
              onclick={() => onpick(hit.id)}
            >
              <span class="vndb-hit__cover">
                {#if hit.image?.thumbnail}
                  <img src={hit.image.thumbnail} alt="" loading="lazy" decoding="async" />
                {:else}
                  <span class="vndb-hit__ph">{(hit.alttitle || hit.title || '?')[0]}</span>
                {/if}
              </span>
              <span class="vndb-hit__text">
                <span class="vndb-hit__title">{hit.alttitle || hit.title}</span>
                {#if hit.alttitle && hit.alttitle !== hit.title}
                  <span class="vndb-hit__alt">{hit.title}</span>
                {/if}
                <span class="vndb-hit__meta">
                  {[hit.released?.slice(0, 4), hit.olang].filter(Boolean).join(' · ')}
                </span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<div class="modal-actions">
  <button type="button" class="btn" onclick={onclose}>
    {t('gameForm.search.close')}
  </button>
</div>

<style>
  .vndb-note {
    padding: var(--space-4) 0;
    font-size: var(--text-base);
    color: var(--fg-muted);
  }

  .vndb-note--error {
    color: var(--error);
  }

  .vndb-progress {
    position: relative;
    overflow: hidden;
    height: var(--space-075);
    margin-top: var(--space-2);
    background: var(--bg-surface);
    border-radius: var(--radius-pill);
  }

  .vndb-progress__bar {
    position: absolute;
    inset-block: 0;
    width: 40%;
    background: var(--accent-gradient);
    border-radius: var(--radius-pill);
    animation: vndb-sweep 1.1s var(--ease-out) infinite;
  }

  .vndb-progress--determinate .vndb-progress__bar {
    animation: none;
    transition: width var(--duration-fast) var(--ease-out);
  }

  .vndb-pull-cancel {
    margin-top: var(--space-2);
  }

  @keyframes vndb-sweep {
    from {
      left: -40%;
    }
    to {
      left: 100%;
    }
  }

  .vndb-hits {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    list-style: none;
    transition: opacity var(--duration-fast) var(--ease-out);
  }

  .vndb-hits.is-pulling {
    opacity: 0.45;
  }

  .vndb-hit {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3);
    text-align: left;
    color: var(--fg-default);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

.vndb-hit:hover {
  background: var(--bg-hover);
  border-color: var(--border-strong);
}

.vndb-hit.active {
  background: var(--bg-selected);
  border-color: var(--accent-border);
}

  .vndb-hit__cover {
    flex-shrink: 0;
    width: var(--space-28);
    aspect-ratio: 3 / 4;
    overflow: hidden;
    background: var(--bg-raised);
    border-radius: var(--radius-sm);
  }

  .vndb-hit__cover img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .vndb-hit__ph {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    font-size: var(--text-xl);
    color: var(--fg-muted);
  }

  .vndb-hit__text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .vndb-hit__title {
    display: -webkit-box;
    overflow: hidden;
    font-size: var(--text-body);
    font-weight: var(--fw-medium);
    color: var(--fg-default);
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .vndb-hit__alt,
  .vndb-hit__meta {
    overflow: hidden;
    font-size: var(--text-base);
    color: var(--fg-muted);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
