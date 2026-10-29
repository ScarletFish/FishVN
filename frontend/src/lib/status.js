import { t } from './i18n.js';

/**
 * 作品状态的唯一定义处。
 * 列表顺序即界面展示顺序（表单下拉、列表分组、筛选条保持一致）。
 * Rust 侧对应 src-tauri/src/types.rs 的 Status 枚举，新增状态两处都要改。
 */
export const STATUS_KEYS = ['playing', 'wishlist', 'finished', 'stalled', 'dropped'];

/** 新作品的默认状态，与建表默认值一致 */
export const DEFAULT_STATUS = 'wishlist';

/** 首页只列「在玩 / 想玩 / 已通关」 */
export const HOME_STATUS_KEYS = ['playing', 'wishlist', 'finished'];

/** VNDB 同步结果按「已玩优先、想玩垫底」分组报告 */
export const REPORT_STATUS_KEYS = ['playing', 'finished', 'stalled', 'dropped', 'wishlist'];

export function statusLabel(key) {
  return t(`status.${key}`);
}

export function statusOptions(keys = STATUS_KEYS) {
  return keys.map((value) => ({ value, label: statusLabel(value) }));
}
