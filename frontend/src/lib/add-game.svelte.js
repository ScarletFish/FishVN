import { t } from './i18n.js';
import { api } from './api.js';
import { toast } from './toast.svelte.js';
import { pickDirectory, pickExecutable } from './pick-path.js';
import { getFormTab } from './form-tab-pref.js';

const DEFAULT_TAB = 'basic';

export const addGame = $state({
  formOpen: false,
  formGameId: null,
  formTab: getFormTab(),
  prefill: null,
  scanOpen: false,
  scanPath: '',
});

export function openForm(id, { tab } = {}) {
  addGame.prefill = null;
  addGame.formGameId = id;
  if (tab) {
    addGame.formTab = tab;
  } else if (id == null) {
    addGame.formTab = DEFAULT_TAB;
  }
  addGame.formOpen = true;
}

export async function openPrefilled(path) {
  try {
    const info = await api.inspectPath(path);
    openForm(null);
    addGame.prefill = {
      name: info.name,
      installations: [{ dir_path: info.dir_path, entry_path: info.entry_path ?? '' }],
    };
  } catch (e) {
    toast(e.message, 'error');
  }
}

export async function addFromFile() {
  const picked = await pickExecutable(t('mylist.pickExecutable'));
  if (!picked) return;
  await openPrefilled(picked);
}

export async function addFromScan() {
  const dir = await pickDirectory(t('mylist.pickDirectory'));
  if (!dir) return;
  addGame.scanPath = dir;
  addGame.scanOpen = true;
}
