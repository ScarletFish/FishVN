import { invoke } from '@tauri-apps/api/core';

// 开发通道，后端见 src-tauri/src/devhttp.rs，用 ?dev-http 打开。
// 打同源的 /dev-http 走 vite 代理，改成 4546 直连会被 CORS 预检挡掉
const DEV_HTTP = import.meta.env.DEV && location.search.includes('dev-http');
const DEV_HTTP_BASE = '/dev-http';

// invoke 会把 gameId 这类驼峰参数转成 snake_case，裸 HTTP 没有这层转换
function toSnake(args) {
  return Object.fromEntries(
    Object.entries(args ?? {}).map(([k, v]) => [
      k.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`),
      v,
    ]),
  );
}

async function overHttp(command, args) {
  const res = await fetch(`${DEV_HTTP_BASE}/${command}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(toSnake(args)),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

// 与 Rust 侧通信。命令层只转发，错误是 AppError 序列化来的字符串。
// invoke 会用原值 reject，这里包成 Error 让调用方统一读 .message
async function call(command, args) {
  try {
    if (DEV_HTTP) return await overHttp(command, args);
    return await invoke(command, args);
  } catch (e) {
    throw new Error(e);
  }
}

export const api = {
  listGames: () => call('list_games'),
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
  searchVndb: (keyword) => call('search_vndb', { keyword }),
  fetchVnMetadata: (vndbId) => call('fetch_vn_metadata', { vndbId }),
  getGameMetadata: (id) => call('get_game_metadata', { id }),
  updateGameMetadata: (id, patch) => call('update_game_metadata', { id, patch }),
};