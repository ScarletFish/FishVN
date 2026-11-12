<script>
  import Icon from './Icon.svelte';
  import Modal from './Modal.svelte';
  import { t } from '../lib/i18n.js';
  import { api } from '../lib/api.js';
  import { subscribe } from '../lib/events.js';
  import { toast } from '../lib/toast.svelte.js';
  import { pickExecutable } from '../lib/pick-path.js';
  import { panelScrollbars } from '../lib/scrollbars.js';

  let { open = $bindable(false), path = '', ondone } = $props();

  let scanning = $state(false);
  let progress = $state(null);
  let candidates = $state([]);
  let picked = $state(new Set());
  let entries = $state({});
  let creating = $state(false);
  let result = $state(null);
  let cancelled = $state(false);
  let off = null;

  $effect(() => {
    if (open && path) {
      start();
      return () => {
        off?.();
        off = null;
      };
    }
  });

  async function start() {
    scanning = true;
    progress = null;
    candidates = [];
    picked = new Set();
    entries = {};
    result = null;
    cancelled = false;
    off = await subscribe('scan', (event) => {
      if (!event.error) progress = event;
    });
    try {
      const found = await api.scanDir(path);
      candidates = found;
      picked = new Set(found.filter((c) => !c.registered).map((c) => c.dir_path));
      entries = Object.fromEntries(found.map((c) => [c.dir_path, c.entry_path ?? '']));
    } catch (e) {
      if (!cancelled) toast(e.message, 'error');
      open = false;
    } finally {
      scanning = false;
      off?.();
      off = null;
    }
  }

  function close() {
    if (creating) return false;
    if (scanning) {
      cancelled = true;
      api.cancelTask('scan');
    }
    open = false;
  }

  function toggle(dirPath) {
    const next = new Set(picked);
    if (next.has(dirPath)) next.delete(dirPath);
    else next.add(dirPath);
    picked = next;
  }

  function toggleAll(event) {
    picked = event.currentTarget.checked
      ? new Set(candidates.filter((c) => !c.registered).map((c) => c.dir_path))
      : new Set();
  }

  const selectable = $derived(candidates.filter((c) => !c.registered));
  const pickedCount = $derived(candidates.filter((c) => picked.has(c.dir_path)).length);
  const allPicked = $derived(selectable.length > 0 && pickedCount === selectable.length);
  const percent = $derived(
    progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0
  );

  function entryStem(rel) {
    const file = rel.split('/').pop() ?? rel;
    const dot = file.lastIndexOf('.');
    return dot > 0 ? file.slice(0, dot) : file;
  }

  function ambiguous(c) {
    if (c.exes.length < 2) return false;
    const want = c.name.toLowerCase();
    const rootHit = c.exes.some(
      (rel) => !rel.includes('/') && entryStem(rel).toLowerCase() === want
    );
    return !rootHit;
  }

  function setEntry(dirPath, value) {
    entries = { ...entries, [dirPath]: value };
  }

  function relativeTo(dir, file) {
    const trimmed = dir.replace(/[\\/]+$/, '');
    const lower = trimmed.toLowerCase();
    if (file.toLowerCase().startsWith(`${lower}\\`)) {
      return file.slice(trimmed.length + 1).replace(/\\/g, '/');
    }
    if (file.toLowerCase().startsWith(`${lower}/`)) {
      return file.slice(trimmed.length + 1);
    }
    return null;
  }

  async function browseEntry(c) {
    const picked = await pickExecutable(t('mylist.pickExecutable'), c.dir_path);
    if (!picked) return;
    const rel = relativeTo(c.dir_path, picked);
    if (rel == null) {
      toast(t('scan.entryOutside'), 'error');
      return;
    }
    setEntry(c.dir_path, rel);
  }

  async function confirm() {
    creating = true;
    let added = 0;
    let skipped = 0;
    for (const c of candidates) {
      if (!picked.has(c.dir_path)) continue;
      try {
        const game = await api.createGame({ name: c.name });
        await api.saveInstallations(game.id, [
          { id: null, dir_path: c.dir_path, entry_path: entries[c.dir_path] || null },
        ]);
        added += 1;
      } catch {
        skipped += 1;
      }
    }
    creating = false;
    result = { added, skipped };
    ondone?.();
  }
</script>

<Modal {open} title={t('scan.title')} variant="modal--merge" onclose={close}>
  {#if scanning}
    <div class="scan-progress">
      <p>{t('scan.scanning')}</p>
      <div class="scan-progress__track">
        <div class="scan-progress__bar" style={`width: ${percent}%`}></div>
      </div>
      <button type="button" class="btn" onclick={close}>{t('common.cancel')}</button>
    </div>
  {:else if result}
    <div class="scan-progress">
      <p>{t('scan.result', result)}</p>
    </div>
  {:else if candidates.length === 0}
    <div class="vn-merge__empty">{t('scan.empty')}</div>
  {:else}
    <div class="scan-head">
      <label class="scan-head__all">
        <input
          type="checkbox"
          class="checkbox"
          checked={allPicked}
          onchange={toggleAll}
          disabled={selectable.length === 0}
        />
        {t('scan.selectAll')}
      </label>
      <span class="scan-head__count">{t('scan.count', { count: candidates.length })}</span>
    </div>
    <div class="scan-rows-scroll" use:panelScrollbars>
      <ul class="scan-rows">
        {#each candidates as c, index (c.dir_path)}
          <li class="scan-row" class:is-registered={c.registered}>
            <label>
              <input
                type="checkbox"
                class="checkbox"
                checked={picked.has(c.dir_path)}
                disabled={c.registered}
                onchange={() => toggle(c.dir_path)}
              />
              <span class="scan-row__name">{c.name}</span>
              <input
                type="text"
                class="scan-row__entry"
                list={`scan-exes-${index}`}
                value={entries[c.dir_path] ?? ''}
                disabled={c.registered}
                aria-label={t('scan.entryOf', { name: c.name })}
                onchange={(e) => setEntry(c.dir_path, e.currentTarget.value.trim())}
              />
              <datalist id={`scan-exes-${index}`}>
                {#each c.exes as exe (exe)}
                  <option value={exe}></option>
                {/each}
              </datalist>
              {#if ambiguous(c)}
                <span
                  class="scan-row__hint"
                  data-tooltip={t('scan.entryHint')}
                >{c.exes.length}</span>
              {/if}
              <button
                type="button"
                class="scan-row__browse"
                aria-label={t('common.browse')}
                data-tooltip={t('common.browse')}
                disabled={c.registered}
                onclick={() => browseEntry(c)}
              >
                <Icon name="folder" size="lg" />
              </button>
              {#if c.registered}
                <span class="scan-row__badge">{t('scan.registered')}</span>
              {/if}
            </label>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="modal-actions">
    {#if result}
      <button type="button" class="btn btn-primary" onclick={() => (open = false)}>
        {t('common.close')}
      </button>
    {:else if !scanning}
      <button type="button" class="btn" onclick={close}>{t('common.cancel')}</button>
      <button
        type="button"
        class="btn btn-primary"
        onclick={confirm}
        disabled={pickedCount === 0 || creating}
      >
        {creating ? t('scan.creating') : t('scan.confirm', { count: pickedCount })}
      </button>
    {/if}
  </div>
</Modal>
