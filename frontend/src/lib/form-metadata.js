
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

export function buildMetadataPatch(fields) {
  return {
    alias: fields.alias.trim() || null,
    released: fields.released.trim() || null,
    rating: toNumber(fields.rating),
    votecount: toInteger(fields.votecount),
    description: fields.description.trim() || null,
    image_path: fields.coverPath.trim() || null,
    tags: fields.tags,
    developers: fields.developers
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
  };
}