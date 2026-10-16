import { open } from '@tauri-apps/plugin-dialog';
import { DEV_HTTP } from './env.js';

const IMAGE_EXT = ['png', 'jpg', 'jpeg', 'webp', 'bmp'];

function guard() {
  if (DEV_HTTP) throw new Error('浏览器模式选不了文件，请在应用窗口里操作');
}

export async function pickDirectory(title) {
  guard();
  const picked = await open({ directory: true, multiple: false, title });
  return picked ?? null;
}

export async function pickImage(title) {
  guard();
  const picked = await open({
    directory: false,
    multiple: false,
    title,
    filters: [{ name: '图片', extensions: IMAGE_EXT }],
  });
  return picked ?? null;
}
