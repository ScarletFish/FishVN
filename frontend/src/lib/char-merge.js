import { CHARACTER_MERGE_FIELDS } from './generated/character-fields.js';
import { roleTier } from './generated/character-roles.js';

const asText = (v) => (v === null || v === undefined ? '' : String(v).trim());

const isBlank = (v) => asText(v) === '';

const sameText = (a, b) => asText(a) === asText(b);

function sameNumber(a, b) {
  if (isBlank(a) || isBlank(b)) return isBlank(a) && isBlank(b);
  return Number(asText(a)) === Number(asText(b));
}

function sameList(a, b, toKey = asText) {
  const x = (a ?? []).map(toKey).filter(Boolean).sort();
  const y = (b ?? []).map(toKey).filter(Boolean).sort();
  return x.length === y.length && x.every((s, i) => s === y[i]);
}

const vaKey = (v) => `${asText(v.staff_id)}|${asText(v.original)}|${asText(v.note)}`;

const traitKey = (t) => (typeof t === 'string' ? t : asText(t?.name ?? t?.id));

const SAME_RULES = {
  text: sameText,
  number: sameNumber,
  list: (a, b) => sameList(a, b),
  listTrait: (a, b) => sameList(a, b, traitKey),
  listVa: (a, b) => sameList(a, b, vaKey),
};

export const MERGE_FIELDS = CHARACTER_MERGE_FIELDS.map((field) => ({
  key: field.key,
  same: SAME_RULES[field.same],
}));

export function charKey(c) {
  return asText(c?.original || c?.name);
}

function indexByKey(list) {
  const map = new Map();
  for (const c of list ?? []) map.set(charKey(c), c);
  return map;
}

function sameCharacter(a, b) {
  return MERGE_FIELDS.every((f) => f.same(a?.[f.key], b?.[f.key]));
}

export function planCharacters(local, incoming) {
  const localMap = indexByKey(local);
  const added = [];
  const conflicts = [];

  for (const c of incoming ?? []) {
    const key = charKey(c);
    const current = localMap.get(key);
    if (current === undefined) {
      added.push(c);
      continue;
    }
    if (!sameCharacter(current, c)) conflicts.push({ key, current, incoming: c });
  }

  const conflictKeys = new Set(conflicts.map((r) => r.key));
  const kept = (local ?? []).filter((c) => !conflictKeys.has(charKey(c)));
  return { added, conflicts, kept };
}

export function applyCharacters(local, incoming, adopted) {
  const incomingMap = indexByKey(incoming);
  const merged = [];

  for (const c of local ?? []) {
    const match = incomingMap.get(charKey(c));
    merged.push(match && adopted?.has(charKey(c)) ? { ...match } : { ...c });
  }

  const localKeys = new Set((local ?? []).map(charKey));
  for (const c of incoming ?? []) {
    if (!localKeys.has(charKey(c))) merged.push({ ...c });
  }

  return merged
    .map((c, i) => ({ c, i }))
    .sort((a, b) => roleTier(a.c.role) - roleTier(b.c.role) || a.i - b.i)
    .map(({ c }, i) => ({ ...c, sort_order: i }));
}