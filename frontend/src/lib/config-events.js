export const CONFIG_CHANGED = 'fishvn:config-changed';

export function notifyConfigChanged() {
  window.dispatchEvent(new CustomEvent(CONFIG_CHANGED));
}
