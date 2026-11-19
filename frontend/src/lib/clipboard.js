import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager';

export async function copyText(text) {
  try {
    await writeText(text);
  } catch {
    await navigator.clipboard.writeText(text);
  }
}

export async function readClip() {
  try {
    return await readText();
  } catch {
    return await navigator.clipboard.readText();
  }
}
