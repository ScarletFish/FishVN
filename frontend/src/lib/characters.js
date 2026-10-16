
import { toInteger } from './form-metadata.js';

export function newLocalId() {
  return `local-${crypto.randomUUID()}`;
}

function trimToNull(value) {
  const text = (value ?? '').trim();
  return text || null;
}

function numToText(value) {
  return value == null ? '' : String(value);
}

export function blankCharacter() {
  return {
    id: newLocalId(),
    source: null,
    name: '',
    original: '',
    gender: '',
    age: '',
    height: '',
    weight: '',
    role: '',
    imagePath: '',
    description: '',
    va: [],
  };
}

export function toDraft(row = {}) {
  return {
    id: row.id ?? newLocalId(),
    source: row.id ? row : null,
    name: row.name ?? '',
    original: row.original ?? '',
    gender: row.gender?.[0] ?? '',
    age: numToText(row.age),
    height: numToText(row.height),
    weight: numToText(row.weight),
    role: row.role ?? '',
    imagePath: row.image_path ?? '',
    description: row.description ?? '',
    va: (row.va ?? []).map((entry) => ({
      key: newLocalId(),
      staffId: entry.staff_id ?? '',
      name: entry.name ?? '',
      original: entry.original ?? '',
      note: entry.note ?? '',
    })),
  };
}

export function toPayload(draft) {
  return {
    ...(draft.source ?? {}),
    id: draft.id,
    name: trimToNull(draft.name),
    original: trimToNull(draft.original),
    gender: draft.gender ? [draft.gender] : null,
    age: toInteger(draft.age),
    height: toInteger(draft.height),
    weight: toInteger(draft.weight),
    role: trimToNull(draft.role),
    image_path: trimToNull(draft.imagePath),
    description: trimToNull(draft.description),
    va: draft.va.map((entry) => ({
      staff_id: entry.staffId || newLocalId(),
      name: trimToNull(entry.name),
      original: trimToNull(entry.original),
      note: entry.note ?? '',
    })),
  };
}

export function blankVa() {
  return { key: newLocalId(), staffId: newLocalId(), name: '', note: '' };
}

export function displayName(draft) {
  return (draft.original ?? '').trim() || (draft.name ?? '').trim();
}

function isUnnamed(draft) {
  return !displayName(draft);
}

export function namedOnly(list) {
  return list.filter((draft) => !isUnnamed(draft));
}

export function moveCharacters(list, index, delta) {
  const target = index + delta;
  if (target < 0 || target >= list.length) return list;
  const next = list.slice();
  [next[index], next[target]] = [next[target], next[index]];
  return next;
}
