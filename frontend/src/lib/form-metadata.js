
import { clampFocal } from './cover-focal.js';

export function toNumber(raw) {
  const text = raw.trim();
  if (!text) return null;
  const value = Number(text);
  return Number.isFinite(value) ? value : null;
}

export function toInteger(raw) {
  const value = toNumber(raw);
  return value === null ? null : Math.trunc(value);
}

export function cleanRating(raw) {
  let cleaned = raw.replace(/[^\d.]/g, '');
  const dot = cleaned.indexOf('.');
  if (dot !== -1) {
    cleaned = cleaned.slice(0, dot + 1) + cleaned.slice(dot + 1).replace(/\./g, '');
  }
  if (cleaned.includes('.')) {
    const [int, dec] = cleaned.split('.');
    cleaned = `${int}.${dec.slice(0, 1)}`;
  }
  return cleaned;
}

export function clampRating(value) {
  return Math.round(Math.min(100, Math.max(0, value)) * 10) / 10;
}

export function ratingRejected(raw) {
  const score = toNumber(raw);
  return Boolean(raw.trim()) && (score === null || score < 0 || score > 100);
}

export function votecountRejected(raw) {
  const votes = toNumber(raw);
  return Boolean(raw.trim()) && (votes === null || votes < 0);
}

export function lengthMinutesRejected(raw) {
  const minutes = toNumber(raw);
  return Boolean(raw.trim()) && (minutes === null || minutes < 0);
}

/**
 * 状态补丁：写入前的最后一次归一化。
 * 评分的钳制不能只靠输入框失焦——回车提交时焦点还在框里，blur 不会触发。
 * `playSeconds` 缺省表示「这次不动总时长」：进行中的游玩结算会自己往上加，
 * 用户没改过就别把表单打开时的旧值写回去。
 */
export function buildStatePatch({ status, userRating, notes, playSeconds }) {
  const patch = {
    status,
    user_rating: userRating === null ? null : clampRating(userRating),
    notes: (notes ?? '').trim() || null,
  };
  if (playSeconds !== undefined) patch.play_seconds = Math.max(0, Math.round(playSeconds));
  return patch;
}

export function buildMetadataPatch(fields) {
  return {
    alias: fields.alias.trim() || null,
    released: fields.released.trim() || null,
    rating: toNumber(fields.rating),
    votecount: toInteger(fields.votecount),
    length_minutes: toInteger(fields.lengthMinutes),
    description: fields.description.trim() || null,
    image_path: fields.coverPath.trim() || null,
    cover_focal_x: clampFocal(fields.focusX),
    cover_focal_y: clampFocal(fields.focusY),
    tags: fields.tags,
    developers: fields.developers
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
    staff: (fields.staff ?? [])
      .filter((entry) => (entry.name ?? '').trim() || (entry.original ?? '').trim())
      .map((entry) => ({
        role: entry.role,
        name: (entry.name ?? '').trim() || null,
        original: (entry.original ?? '').trim() || null,
        note: entry.note ?? null,
      })),
  };
}