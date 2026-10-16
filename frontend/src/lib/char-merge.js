
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

const FIELDS = [
  { key: 'name', same: sameText },
  { key: 'original', same: sameText },
  { key: 'description', same: sameText },
  { key: 'birthday', same: sameText },
  { key: 'blood_type', same: sameText },
  { key: 'cup', same: sameText },
  { key: 'role', same: sameText },
  { key: 'image_id', same: sameText },
  { key: 'image_path', same: sameText },
  { key: 'gender', same: (a, b) => sameList(a, b) },
  { key: 'aliases', same: (a, b) => sameList(a, b) },
  { key: 'traits', same: (a, b) => sameList(a, b, traitKey) },
  { key: 'va', same: (a, b) => sameList(a, b, vaKey) },
  { key: 'age', same: sameNumber },
  { key: 'height', same: sameNumber },
  { key: 'weight', same: sameNumber },
  { key: 'bust', same: sameNumber },
  { key: 'waist', same: sameNumber },
  { key: 'hips', same: sameNumber },
];

export function charKey(c) {
  return asText(c?.original || c?.name);
}

function indexByKey(list) {
  const map = new Map();
  for (const c of list ?? []) map.set(charKey(c), c);
  return map;
}

function sameCharacter(a, b) {
  return FIELDS.every((f) => f.same(a?.[f.key], b?.[f.key]));
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

const ROLE_TIERS = ['main', 'primary', 'side', 'appears'];

function tier(c) {
  const index = ROLE_TIERS.indexOf(asText(c?.role));
  return index === -1 ? ROLE_TIERS.length : index;
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
    .sort((a, b) => tier(a.c) - tier(b.c) || a.i - b.i)
    .map(({ c }, i) => ({ ...c, sort_order: i }));
}