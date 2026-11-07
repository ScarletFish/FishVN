import { api } from './api.js';
import { t } from './i18n.js';
import { toast } from './toast.svelte.js';
import { subscribe } from './events.js';
import { applyIncoming, conflictRows } from './vn-merge.js';
import { applyCharacters, planCharacters } from './char-merge.js';

export function createVnMergeFlow() {
  let pulling = $state(false);
  let progress = $state(null);
  let notice = $state('');
  let cancelled = $state(false);
  let rows = $state([]);
  let charRows = $state([]);
  let incoming = $state(null);
  let incomingCharacters = $state([]);
  let selectedId = $state(null);
  let previous = null;
  let previousCharacters = [];

  function clear() {
    rows = [];
    charRows = [];
    incoming = null;
    incomingCharacters = [];
    selectedId = null;
    previous = null;
    previousCharacters = [];
  }

  function reset() {
    pulling = false;
    progress = null;
    notice = '';
    cancelled = false;
    clear();
  }

  async function run(channel, task, filter = null) {
    if (pulling) return null;
    pulling = true;
    progress = null;
    notice = '';
    cancelled = false;
    let off = null;
    try {
      off = await subscribe(channel, (ev) => {
        if (filter && !filter(ev)) return;
        if (ev.error) notice = ev.error;
        else if (ev.phase) progress = ev;
      });
      return await task();
    } catch (e) {
      if (!cancelled) toast(e.message, 'error');
      return null;
    } finally {
      off?.();
      pulling = false;
      progress = null;
    }
  }

  async function pick(id, load) {
    clear();
    return run('vndb-sync', async () => {
      const [games, ctx] = await Promise.all([api.listGames(), load()]);
      const owner = games.find((g) => g.vndb_id === id && g.id !== ctx.ownId);
      if (owner) {
        toast(t('gameForm.error.vndbTaken', { name: owner.name }), 'error');
        return null;
      }
      const sync = await api.fetchVnMetadata(id);
      if (cancelled) return null;
      previous = ctx.current;
      previousCharacters = ctx.characters ?? [];
      incoming = sync;
      incomingCharacters = sync.characters ?? [];
      selectedId = id;
      rows = conflictRows(previous, incoming);
      charRows = planCharacters(previousCharacters, incomingCharacters).conflicts;
      if (!rows.length && !charRows.length) {
        toast(t('backfillPage.fixNone'));
        clear();
        return null;
      }
      return true;
    });
  }

  function accept(picked, charPicked) {
    if (!incoming || !previous) return null;
    const result = {
      fields: applyIncoming(previous, incoming, picked),
      characters: applyCharacters(previousCharacters, incomingCharacters, charPicked),
      id: incoming.id ?? selectedId,
      previous,
      incoming,
    };
    clear();
    return result;
  }

  async function cancel() {
    cancelled = true;
    await api.cancelTask('vndb-sync');
  }

  return {
    get pulling() {
      return pulling;
    },
    get progress() {
      return progress;
    },
    get notice() {
      return notice;
    },
    get cancelled() {
      return cancelled;
    },
    get rows() {
      return rows;
    },
    get charRows() {
      return charRows;
    },
    get selectedId() {
      return selectedId;
    },
    run,
    pick,
    accept,
    cancel,
    reset,
  };
}
