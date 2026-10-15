/** 角色粒度的 VNDB 同步比对。与作品级同口径：本地优先，冲突逐个取舍。 */

const asText = (v) => (v === null || v === undefined ? '' : String(v).trim());

const isBlank = (v) => asText(v) === '';

/** 后端 trimToNull 会把空串变 null，两边都要归一成同一个空值 */
const sameText = (a, b) => asText(a) === asText(b);

/** 「80」与 80 判等；空值只与空值判等，0 是真实取值 */
function sameNumber(a, b) {
  if (isBlank(a) || isBlank(b)) return isBlank(a) && isBlank(b);
  return Number(asText(a)) === Number(asText(b));
}

/** 集合语义：顺序不同按相等处理 */
function sameList(a, b, toKey = asText) {
  const x = (a ?? []).map(toKey).filter(Boolean).sort();
  const y = (b ?? []).map(toKey).filter(Boolean).sort();
  return x.length === y.length && x.every((s, i) => s === y[i]);
}

const vaKey = (v) => `${asText(v.staff_id)}|${asText(v.original)}|${asText(v.note)}`;

const traitKey = (t) => (typeof t === 'string' ? t : asText(t?.name ?? t?.id));

/** 比对的字段表，只收内容字段：sort_order 是位置，id 是存储身份 */
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

/** 配对键优先取原名：罗马字名重名会让两个不同角色被判成同一个 */
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

/** kept 含「同名且内容一致」的本地角色：它们既不是新增也不需要取舍 */
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

/**
 * 合并结果。adopted 命中的冲突取 VNDB 那份，其余一律保留本地——包括本地独有的角色，
 * 用户手建的角色在同步后原样留着。
 */
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

  // 稳定排序，同档内保持各自来源的原有顺序
  return merged
    .map((c, i) => ({ c, i }))
    .sort((a, b) => tier(a.c) - tier(b.c) || a.i - b.i)
    .map(({ c }, i) => ({ ...c, sort_order: i }));
}