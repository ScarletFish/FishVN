import { t } from './i18n.js';

export const STATUS_KEYS = ['playing', 'wishlist', 'finished', 'stalled', 'dropped'];

export const DEFAULT_STATUS = 'wishlist';

export const HOME_STATUS_KEYS = ['playing', 'wishlist', 'finished'];

export const REPORT_STATUS_KEYS = ['playing', 'finished', 'stalled', 'dropped', 'wishlist'];

export function statusLabel(key) {
  return t(`status.${key}`);
}

export function statusOptions(keys = STATUS_KEYS) {
  return keys.map((value) => ({ value, label: statusLabel(value) }));
}
