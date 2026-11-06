import { t } from './i18n.js';

const STATUS = [
  { key: 'playing', home: true, report: 0 },
  { key: 'wishlist', home: true, report: 4, default: true },
  { key: 'finished', home: true, report: 1 },
  { key: 'stalled', home: false, report: 2 },
  { key: 'dropped', home: false, report: 3 },
];

export const DEFAULT_STATUS = STATUS.find((s) => s.default).key;

export const STATUS_KEYS = STATUS.map((s) => s.key);

export const HOME_STATUS_KEYS = STATUS.filter((s) => s.home).map((s) => s.key);

export const REPORT_STATUS_KEYS = [...STATUS]
  .sort((a, b) => a.report - b.report)
  .map((s) => s.key);

export function statusLabel(key) {
  return t(`status.${key}`);
}

export function statusOptions(keys = STATUS_KEYS) {
  return keys.map((value) => ({ value, label: statusLabel(value) }));
}
