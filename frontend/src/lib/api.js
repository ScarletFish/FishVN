import { invoke } from '@tauri-apps/api/core';
import { DEV_HTTP, DEV_HTTP_BASE } from './env.js';
import { ensureBackend } from './dev-backend.js';
import { report } from './frontend-log.js';

function toSnake(args) {
  return Object.fromEntries(
    Object.entries(args ?? {}).map(([k, v]) => [
      k.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`),
      v,
    ]),
  );
}

async function overHttp(command, args) {
  await ensureBackend();
  const res = await fetch(`${DEV_HTTP_BASE}/${command}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(toSnake(args)),
  });
  const text = await res.text();
  if (!res.ok) throw new Error(text);
  return JSON.parse(text);
}

async function call(command, args) {
  try {
    if (DEV_HTTP) return await overHttp(command, args);
    return await invoke(command, args);
  } catch (e) {
    report('warn', `IPC 失败 ${command}：${e}`);
    throw new Error(e);
  }
}

export const api = {
  listGames: () => call('list_games'),
  queryGames: (spec) => call('query_games', { spec }),
  queryCharacters: (spec) => call('query_characters', { spec }),
  listStaff: () => call('list_staff'),
  updateStaff: (patch) => call('update_staff', { patch }),
  mergeStaff: (merge) => call('merge_staff', { merge }),
  dashboardStats: () => call('dashboard_stats'),
  dashboardRecent: () => call('dashboard_recent'),
  dashboardTagCloud: () => call('dashboard_tag_cloud'),
  libraryTags: () => call('library_tags'),
  libraryTraits: () => call('library_traits'),
  dashboardMonthlyActivity: (months = 12) => call('dashboard_monthly_activity', { months }),
  dashboardPlaytimeTop: () => call('dashboard_playtime_top'),
  dashboardReleaseEras: () => call('dashboard_release_eras'),
  getGame: (id) => call('get_game', { id }),
  createGame: (input) => call('create_game', { input }),
  updateGame: (id, patch) => call('update_game', { id, patch }),
  deleteGame: (id) => call('delete_game', { id }),
  getGameState: (id) => call('get_game_state', { id }),
  updateGameState: (id, patch) => call('update_game_state', { id, patch }),
  listInstallations: (gameId) => call('list_installations', gameId ? { gameId } : {}),
  getInstallation: (id) => call('get_installation', { id }),
  createInstallation: (input) => call('create_installation', { input }),
  updateInstallation: (id, patch) => call('update_installation', { id, patch }),
  deleteInstallation: (id) => call('delete_installation', { id }),
  saveInstallations: (gameId, entries) => call('save_installations', { gameId, entries }),
  launchGame: (id) => call('launch_game', { id }),
  listActiveSessions: () => call('list_active_sessions'),
  gamePlayLog: (id) => call('game_play_log', { id }),
  searchVndb: (keyword) => call('search_vndb', { keyword }),
  fetchVnMetadata: (vndbId) => call('fetch_vn_metadata', { vndbId }),
  cancelTask: (task) => call('cancel_task', { task }),
  inspectPath: (path) => call('inspect_path', { path }),
  scanDir: (path) => call('scan_dir', { path }),
  pathExists: (path) => call('path_exists', { path }),
  getGameMetadata: (id) => call('get_game_metadata', { id }),
  updateGameMetadata: (id, patch) => call('update_game_metadata', { id, patch }),
  getConfig: () => call('get_config'),
  updateConfig: (patch) => call('update_config', { patch }),
  testVndbToken: (token) =>
    token ? call('test_vndb_token', { token }) : call('test_vndb_token'),
  syncVndb: () => call('sync_vndb'),
  applySyncPlan: (decision) => call('apply_sync_plan', { decision }),
  startBackfill: (gameIds) => call('start_backfill', { gameIds }),
  backfillSingle: (gameId) => call('backfill_single', { gameId }),
  backfillStatus: () => call('backfill_status'),
  backfillItem: (gameId) => call('backfill_item', { gameId }),
  eventsSince: (after) => call('events_since', { after }),
  pauseBackfill: () => call('pause_backfill'),
  resumeBackfill: () => call('resume_backfill'),
  tagDictionary: () => call('tag_dictionary'),
  listGameCharacters: (id) => call('list_game_characters', { id }),
  saveGameCharacters: (id, characters) => call('save_game_characters', { id, characters }),
  allowLocalFile: (path) => call('allow_local_file', { path }),
  thumbnail: (src) => call('thumbnail', { src }),
  createBackup: () => call('create_backup'),
  listBackups: () => call('list_backups'),
  restoreBackup: (id) => call('restore_backup', { id }),
  deleteBackup: (id) => call('delete_backup', { id }),
};
