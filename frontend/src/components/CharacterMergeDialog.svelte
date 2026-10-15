<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { charKey } from '../lib/char-merge.js';
  import { coverSrc } from '../lib/cover-src.js';

  let { open = $bindable(false), rows = [], onaccept, oncancel } = $props();

  let picked = $state(new Set());
  let thumbs = $state([]);

  $effect(() => {
    if (open) picked = new Set(rows.map((row) => row.key));
  });

  // 立绘走封面那条 coverSrc 管线，两侧一起预取；面板关掉时结果作废
  $effect(() => {
    const paths = rows.map((row) => [row.current?.image_path, row.incoming?.image_path]);
    let live = true;
    Promise.all(
      paths.map((pair) =>
        Promise.all(pair.map((path) => coverSrc(path).catch(() => null)))
      )
    ).then((all) => {
      if (live) thumbs = all;
    });
    return () => {
      live = false;
    };
  });

  function toggle(key) {
    const next = new Set(picked);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    picked = next;
  }

  function nameOf(row) {
    return row?.original || row?.name || '';
  }

  function roleLabel(role) {
    return role ? t(`gameForm.charRole.${role}`) : '';
  }
</script>

<Modal {open} title={t('gameForm.merge.charTitle')} variant="modal--merge" onclose={oncancel}>
  {#if rows.length === 0}
    <div class="vn-merge__empty">{t('gameForm.merge.charEmpty')}</div>
  {:else}
    <div class="vn-merge__head" aria-hidden="true">
      <span>{t('gameForm.field.charName')}</span>
      <span>{t('gameForm.merge.current')}</span>
      <span>{t('gameForm.merge.incoming')}</span>
    </div>
    <ul class="vn-merge__rows scroll-thin">
      {#each rows as row, index (row.key)}
        {@const on = picked.has(row.key)}
        <li class="vn-merge__row vn-merge__row--char">
          <span class="vn-merge__label">{charKey(row.current) || charKey(row.incoming)}</span>
          {#each [[row.current, !on], [row.incoming, on]] as [value, active], side}
            <span class="vn-merge__cell" class:is-active={active}>
              <span class="vn-merge__thumb">
                {#if thumbs[index]?.[side]}
                  <img
                    src={thumbs[index][side]}
                    alt=""
                    loading="lazy"
                    decoding="async"
                  />
                {:else}
                  <span class="vn-merge__thumb-ph" aria-hidden="true">—</span>
                {/if}
              </span>
              <span class="vn-merge__text">
                {nameOf(value)}
                {#if value?.role}
                  <span class="vn-merge__meta">{roleLabel(value.role)}</span>
                {/if}
              </span>
              <button
                type="button"
                class="vn-merge__dot"
                aria-pressed={active}
                aria-label={t(`gameForm.merge.${side === 0 ? 'keep' : 'adopt'}`)}
                onclick={() => toggle(row.key)}
              ></button>
            </span>
          {/each}
        </li>
      {/each}
    </ul>
  {/if}

  <div class="modal-actions">
    <button type="button" class="btn" onclick={oncancel}>
      {t('common.cancel')}
    </button>
    <button type="button" class="btn btn-primary" onclick={() => onaccept?.(picked)}>
      {t('gameForm.merge.accept', { count: picked.size })}
    </button>
  </div>
</Modal>