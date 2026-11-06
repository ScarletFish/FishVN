
import { clampFocal } from './cover-focal.js';

export function toNumber(raw) {
  const text = String(raw ?? '').trim();
  if (!text) return null;
  const value = Number(text);
  return Number.isFinite(value) ? value : null;
}

export function toInteger(raw) {
  const value = toNumber(raw);
  return value === null ? null : Math.trunc(value);
}

export function cleanInteger(raw, maxIntDigits = 6) {
  return String(raw ?? '')
    .replace(/\D/g, '')
    .slice(0, maxIntDigits);
}

export function normalizeInteger(raw, { min = 0, max = Number.MAX_SAFE_INTEGER, maxIntDigits = 6 } = {}) {
  const digits = cleanInteger(raw, maxIntDigits);
  if (!digits) return '';
  return String(Math.min(max, Math.max(min, Number(digits))));
}

export function normalizeRating(raw) {
  const cleaned = cleanRating(raw);
  if (!cleaned || cleaned === '.') return '';
  return String(clampRating(Number(cleaned)));
}

export function cleanRating(raw) {
  let cleaned = String(raw ?? '').replace(/[^\d.]/g, '');
  const dot = cleaned.indexOf('.');
  if (dot !== -1) {
    cleaned = cleaned.slice(0, dot + 1) + cleaned.slice(dot + 1).replace(/\./g, '');
  }
  if (cleaned.includes('.')) {
    const [int, dec] = cleaned.split('.');
    cleaned = `${int}.${dec.slice(0, 1)}`;
  }
  const [int, dec] = cleaned.split('.');
  return dec === undefined ? int.slice(0, 3) : `${int.slice(0, 3)}.${dec}`;
}

export function clampRating(value) {
  return Math.round(Math.min(100, Math.max(0, value)) * 10) / 10;
}

export function isValidBirthday(raw) {
  const text = String(raw ?? '').trim();
  if (!text) return true;
  const full = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(text);
  const partial = /^(\d{1,2})-(\d{1,2})$/.exec(text);
  const parts = full ? [full[1], full[2], full[3]] : partial ? [null, partial[1], partial[2]] : null;
  if (!parts) return false;
  const month = Number(parts[1]);
  const day = Number(parts[2]);
  if (month < 1 || month > 12 || day < 1 || day > 31) return false;
  if (!full) return true;
  const probe = new Date(Date.UTC(Number(parts[0]), month - 1, day));
  return probe.getUTCMonth() === month - 1 && probe.getUTCDate() === day;
}

export function ratingRejected(raw) {
  const score = toNumber(raw);
  return Boolean(String(raw ?? '').trim()) && (score === null || score < 0 || score > 100);
}

export function votecountRejected(raw) {
  const votes = toNumber(raw);
  return Boolean(String(raw ?? '').trim()) && (votes === null || votes < 0);
}

export function lengthMinutesRejected(raw) {
  const minutes = toNumber(raw);
  return Boolean(String(raw ?? '').trim()) && (minutes === null || minutes < 0);
}

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
    alias: (fields.alias ?? '').trim() || null,
    released: (fields.released ?? '').trim() || null,
    rating: toNumber(fields.rating),
    votecount: toInteger(fields.votecount),
    length_minutes: toInteger(fields.lengthMinutes),
    description: (fields.description ?? '').trim() || null,
    image_path: (fields.coverPath ?? '').trim() || null,
    cover_focal_x: clampFocal(fields.focusX),
    cover_focal_y: clampFocal(fields.focusY),
    ambience_path: (fields.ambiencePath ?? '').trim() || null,
    ambience_mode: fields.ambienceMode,
    ambience_plate: fields.ambiencePlate,
    ambience_focal_x: clampFocal(fields.ambienceFocalX),
    ambience_focal_y: clampFocal(fields.ambienceFocalY),
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