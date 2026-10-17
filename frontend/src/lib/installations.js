import { newLocalId } from './characters.js';

const TRAILING_SEP = /[\\/]+$/;
const DRIVE_ONLY = /^[A-Za-z]:$/;
const LAUNCH_EXT = ['exe', 'bat', 'cmd', 'com', 'lnk'];
const SEPARATORS = /\//g;

function pathSeparator(dirPath) {
  return dirPath.includes('\\') && !dirPath.includes('/') ? '\\' : '/';
}

export function joinLaunchPath(dirPath, entryPath) {
  const dir = (dirPath ?? '').replace(TRAILING_SEP, '');
  const entry = (entryPath ?? '').trim();
  if (!dir) return entry;
  if (!entry) return dir;
  return `${dir}${pathSeparator(dir)}${entry}`;
}

export function splitLaunchPath(value) {
  const text = (value ?? '').trim().replace(TRAILING_SEP, '');
  const cut = Math.max(text.lastIndexOf('\\'), text.lastIndexOf('/'));
  const name = cut < 0 ? text : text.slice(cut + 1);
  if (!hasLaunchExtension(name)) return { dirPath: text, entryPath: '' };

  let dirPath = cut < 0 ? '' : text.slice(0, cut);
  if (DRIVE_ONLY.test(dirPath)) dirPath += '\\';
  return { dirPath, entryPath: name };
}

export function toInstallationDraft(row = {}) {
  const dirPath = row.dir_path ?? '';
  const entryPath = row.entry_path ?? '';
  return {
    key: newLocalId(),
    id: row.id ?? null,
    dirPath,
    entryPath,
    path: joinLaunchPath(dirPath, entryPath),
  };
}

export function newInstallation(path = '') {
  return { key: newLocalId(), id: null, dirPath: '', entryPath: '', path };
}

export function installPathInvalid(draft) {
  const text = draft.path.trim();
  if (!text) return true;
  if (isUnchanged(draft)) return false;
  return !splitLaunchPath(text).dirPath;
}

export function toInstallationPayload(draft) {
  if (installPathInvalid(draft)) return null;
  if (isUnchanged(draft)) {
    return {
      id: draft.id ?? null,
      dir_path: draft.dirPath,
      entry_path: draft.entryPath || null,
    };
  }
  const { dirPath, entryPath } = splitLaunchPath(draft.path.trim());
  return { id: draft.id ?? null, dir_path: dirPath, entry_path: entryPath || null };
}

export function defaultKeyOf(items, defaultId) {
  if (items.length === 0) return null;
  const hit = defaultId ? items.find((item) => item.id === defaultId) : null;
  return (hit ?? items[0]).key;
}

export function resolveDefaultId(saved, items, defaultKey) {
  const index = items.findIndex((item) => item.key === defaultKey);
  const chosen = index >= 0 ? saved[index] : saved[0];
  return chosen?.id ?? null;
}

export function hasInstallation(items, draft, ignoreKey = null) {
  const key = installationKey(draft);
  if (!key) return false;
  return items.some((item) => item.key !== ignoreKey && installationKey(item) === key);
}

function installationKey(draft) {
  const payload = toInstallationPayload(draft);
  if (!payload) return null;
  const dir = payload.dir_path.replace(SEPARATORS, '\\');
  return `${dir}\u0000${payload.entry_path ?? ''}`;
}

function isUnchanged(draft) {
  return draft.path.trim() === joinLaunchPath(draft.dirPath, draft.entryPath);
}

function hasLaunchExtension(name) {
  const dot = name.lastIndexOf('.');
  if (dot <= 0 || dot === name.length - 1) return false;
  return LAUNCH_EXT.includes(name.slice(dot + 1).toLowerCase());
}
