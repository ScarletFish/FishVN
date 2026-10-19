import { invoke } from '@tauri-apps/api/core';

const MAX_LENGTH = 2000;

export function report(level, message) {
  if (!('__TAURI_INTERNALS__' in window)) return;
  const text = message.length > MAX_LENGTH ? `${message.slice(0, MAX_LENGTH)}…` : message;
  invoke('log_frontend', { level, message: text }).catch(() => {});
}
