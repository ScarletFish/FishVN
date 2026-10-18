<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { asText } from '../lib/vn-merge.js';
  import { staffDiff, staffGroupText, staffText } from '../lib/staff.js';
  import { compactDuration } from '../lib/playtime.js';
  import { coverSrc } from '../lib/cover-src.js';

  let { open = $bindable(false), rows = [], onaccept, oncancel } = $props();

  let picked = $state(new Set());
  let thumbs = $state([]);

  $effect(() => {
    if (open) picked = new Set(rows.map((r) => r.key));
  });

  $effect(() => {
    const paths = rows.map((r) => (r.key === 'coverPath' ? [r.current, r.incoming] : []));
    let live = true;
    Promise.all(
      paths.map(async (pair) => {
        const out = [];
        for (const p of pair) {
          if (!p) {
            out.push(null);
            continue;
          }
          try {
            out.push(await coverSrc(p));
          } catch {
            out.push(null);
          }
        }
        return out;
      })
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

  function isCover(key) {
    return key === 'coverPath';
  }

  function cellText(key, value) {
    if (key === 'length_minutes' && value !== '' && value != null) {
      return `#${compactDuration(Number(value))}`;
    }
    if (key === 'staff') return staffText(value);
    return asText(key, value);
  }
</script>

<Modal {open} title={t('gameForm.merge.title')} variant="modal--merge" onclose={oncancel}>
  {#if rows.length === 0}
    <div class="vn-merge__empty">{t('gameForm.merge.empty')}</div>
  {:else}
    <div class="vn-merge__head" aria-hidden="true">
      <span></span>
      <span>{t('gameForm.merge.current')}</span>
      <span>{t('gameForm.merge.incoming')}</span>
    </div>
    <ul class="vn-merge__rows scroll-thin">
      {#each rows as row, index (row.key)}
        {@const on = picked.has(row.key)}
        {@const diff = row.key === 'staff' ? staffDiff(row.current, row.incoming) : null}
        <li class="vn-merge__row">
          <span class="vn-merge__label">{t(`gameForm.field.${row.key}`)}</span>
          {#each [[row.current, !on], [row.incoming, on]] as [value, active], side}
            <span class="vn-merge__cell" class:is-active={active}>
              {#if isCover(row.key)}
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
              {:else if diff?.labels.length}
                <span class="vn-merge__body">
                  <span class="vn-merge__text">{staffGroupText(value, diff.labels)}</span>
                  {#if diff.rest > 0}
                    <span class="vn-merge__meta">
                      {t('gameForm.merge.staffRest', { count: diff.rest })}
                    </span>
                  {/if}
                </span>
              {:else}
                <span class="vn-merge__text">{cellText(row.key, value)}</span>
              {/if}
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