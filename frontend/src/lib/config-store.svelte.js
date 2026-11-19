import { api } from './api.js';
import { CONFIG_CHANGED, notifyConfigChanged } from './config-events.js';
import { CHARACTER_SCOPES, DEFAULT_CHARACTER_SCOPE } from './generated/character-roles.js';
import { normalizeStartupPage } from './startup-pages.js';
import { toast } from './toast.svelte.js';

const SPOILER_LEVELS = ['hide_all', 'hide_major', 'show_all'];

function emptyConfig() {
  return {
    time_metric: 'foreground',
    theme: 'light',
    accent: 'default',
    dashboard_layout: [],
    dashboard_include_uninstalled: true,
    hide_r18_tags: false,
    spoiler_level: 'hide_all',
    excluded_tags: [],
    character_scope: DEFAULT_CHARACTER_SCOPE,
    vndb_sync_mode: 'download',
    vndb_sync_wishlist: false,
    vndb_sync_rating: true,
    has_vndb_token: false,
    magpie_path: '',
    le_path: '',
    quit_behavior: 'quit',
    minimize_on_launch: true,
    startup_page: 'home',
    backup_retention_days: null,
  };
}

function normalize(raw) {
  const fallback = emptyConfig();
  return {
    time_metric: raw.time_metric === 'alive' ? 'alive' : fallback.time_metric,
    theme: typeof raw.theme === 'string' ? raw.theme : fallback.theme,
    accent: typeof raw.accent === 'string' ? raw.accent : fallback.accent,
    dashboard_layout: Array.isArray(raw.dashboard_layout) ? raw.dashboard_layout : [],
    dashboard_include_uninstalled: raw.dashboard_include_uninstalled !== false,
    hide_r18_tags: raw.hide_r18_tags === true,
    spoiler_level: SPOILER_LEVELS.includes(raw.spoiler_level)
      ? raw.spoiler_level
      : fallback.spoiler_level,
    excluded_tags: Array.isArray(raw.excluded_tags) ? raw.excluded_tags : [],
    character_scope: CHARACTER_SCOPES.some((scope) => scope.id === raw.character_scope)
      ? raw.character_scope
      : DEFAULT_CHARACTER_SCOPE,
    vndb_sync_mode:
      raw.vndb_sync_mode === 'upload' || raw.vndb_sync_mode === 'both'
        ? raw.vndb_sync_mode
        : fallback.vndb_sync_mode,
    vndb_sync_wishlist: raw.vndb_sync_wishlist === true,
    vndb_sync_rating: raw.vndb_sync_rating !== false,
    has_vndb_token: raw.has_vndb_token === true,
    magpie_path: raw.magpie_path ?? '',
    le_path: raw.le_path ?? '',
    quit_behavior: raw.quit_behavior === 'minimize_to_tray' ? 'minimize_to_tray' : 'quit',
    minimize_on_launch: raw.minimize_on_launch !== false,
    startup_page: normalizeStartupPage(raw.startup_page),
    backup_retention_days: raw.backup_retention_days ?? null,
  };
}

let config = $state(emptyConfig());

export function readConfig() {
  return config;
}

export function getHideR18Tags() {
  return config.hide_r18_tags;
}

export function getSpoilerLevel() {
  return config.spoiler_level;
}

export function getExcludedTags() {
  return config.excluded_tags;
}

export function getTimeMetric() {
  return config.time_metric;
}

let inflight = null;

export function loadConfig() {
  inflight ??= api
    .getConfig()
    .then((raw) => {
      Object.assign(config, normalize(raw ?? {}));
    })
    .catch(() => {})
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

export async function saveConfig(patch) {
  const previous = { ...config };
  Object.assign(config, patch);
  try {
    await api.updateConfig(patch);
    notifyConfigChanged();
    return true;
  } catch (err) {
    Object.assign(config, previous);
    toast(err.message, 'error');
    return false;
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener(CONFIG_CHANGED, loadConfig);
}
