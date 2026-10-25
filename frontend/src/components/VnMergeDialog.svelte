<script>
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { asText } from '../lib/vn-merge.js';
  import { staffDiff, staffGroupText, staffText } from '../lib/staff.js';
  import { charKey } from '../lib/char-merge.js';
  import { compactDuration } from '../lib/playtime.js';
  import { coverSrc } from '../lib/cover-src.js';
  import { panelScrollbars } from '../lib/scrollbars.js';

  let {
    open = $bindable(false),
    rows = [],
    charRows = [],
    coverFocalX = 50,
    coverFocalY = 50,
    onaccept,
    oncancel,
  } = $props();

  let picked = $state(new Set());
  let charPicked = $state(new Set());
  let thumbs = $state([]);
  let charThumbs = $state([]);

  $effect(() => {
    if (open) {
      picked = new Set(rows.map((r) => r.key));
      charPicked = new Set(charRows.map((r) => r.key));
    }
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

  $effect(() => {
    const paths = charRows.map((r) => [r.current?.image_path, r.incoming?.image_path]);
    let live = true;
    Promise.all(
      paths.map((pair) => Promise.all(pair.map((path) => coverSrc(path).catch(() => null))))
    ).then((all) => {
      if (live) charThumbs = all;
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

  function toggleChar(key) {
    const next = new Set(charPicked);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    charPicked = next;
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

  function charNameOf(row) {
    return row?.original || row?.name || '';
  }

  function charRoleLabel(role) {
    return role ? t(`gameForm.charRole.${role}`) : '';
  }
</script>

<Modal {open} title={t('gameForm.merge.title')} variant="modal--merge" onclose={oncancel}>
  {#if rows.length === 0 && charRows.length === 0}
    <div class="vn-merge__empty">{t('gameForm.merge.empty')}</div>
  {:else}
    <div class="vn-merge__head" aria-hidden="true">
      <span></span>
      <span>{t('gameForm.merge.current')}</span>
      <span>{t('gameForm.merge.incoming')}</span>
    </div>
    <div class="vn-merge__scroll" use:panelScrollbars>
      <ul class="vn-merge__rows">
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
                        style:object-position={side === 0 ? `${coverFocalX}% ${coverFocalY}%` : undefined}
                      />
                    {:else}
                      <span class="vn-merge__thumb-ph" aria-hidden="true">—</span>
                    {/if}
                  </span>
                {:else if diff?.labels.length}
                  <span class="vn-merge__body">
                    <span class="vn-merge__text scroll-thin">{staffGroupText(value, diff.labels)}</span>
                    {#if diff.rest > 0}
                      <span class="vn-merge__meta">
                        {t('gameForm.merge.staffRest', { count: diff.rest })}
                      </span>
                    {/if}
                  </span>
                {:else}
                  <span class="vn-merge__text scroll-thin">
                    {cellText(row.key, value) || t('gameForm.merge.emptyValue')}
                  </span>
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

        {#if charRows.length}
          <li class="vn-merge__section">
            {t('gameForm.merge.charSection', { count: charRows.length })}
          </li>
          {#each charRows as row, index (row.key)}
            {@const on = charPicked.has(row.key)}
            <li class="vn-merge__row vn-merge__row--char">
              <span class="vn-merge__label">{charKey(row.current) || charKey(row.incoming)}</span>
              {#each [[row.current, !on], [row.incoming, on]] as [value, active], side}
                <span class="vn-merge__cell" class:is-active={active}>
                  <span class="vn-merge__thumb">
                    {#if charThumbs[index]?.[side]}
                      <img
                        src={charThumbs[index][side]}
                        alt=""
                        loading="lazy"
                        decoding="async"
                      />
                    {:else}
                      <span class="vn-merge__thumb-ph" aria-hidden="true">—</span>
                    {/if}
                  </span>
                  <span class="vn-merge__text scroll-thin">
                    {charNameOf(value) || t('gameForm.merge.emptyValue')}
                    {#if value?.role}
                      <span class="vn-merge__meta">{charRoleLabel(value.role)}</span>
                    {/if}
                  </span>
                  <button
                    type="button"
                    class="vn-merge__dot"
                    aria-pressed={active}
                    aria-label={t(`gameForm.merge.${side === 0 ? 'keep' : 'adopt'}`)}
                    onclick={() => toggleChar(row.key)}
                  ></button>
                </span>
              {/each}
            </li>
          {/each}
        {/if}
      </ul>
    </div>
  {/if}

  <div class="modal-actions">
    <button type="button" class="btn" onclick={oncancel}>
      {t('common.cancel')}
    </button>
    <button
      type="button"
      class="btn btn-primary"
      onclick={() => onaccept?.(picked, charPicked)}
    >
      {t('gameForm.merge.accept', { count: picked.size + charPicked.size })}
    </button>
  </div>
</Modal>
