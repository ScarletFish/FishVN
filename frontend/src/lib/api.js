import { invoke } from '@tauri-apps/api/core';

// 与 Rust 侧通信。命令层只转发，错误是 AppError 序列化来的字符串。
// invoke 会用原值 reject，这里包成 Error 让调用方统一读 .message
async function call(command, args) {
  try {
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
};