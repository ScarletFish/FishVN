import { openUrl } from '@tauri-apps/plugin-opener';

export function openExternal(url) {
  openUrl(url).catch(() => {});
}
