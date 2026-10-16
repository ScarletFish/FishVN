/** VNDB 元数据与表单字段的比对。面板的文案在组件侧按 key 查词条。 */

export function stripBbcode(text) {
  if (!text) return '';
  return text
    .replace(/\[(spoiler|raw)\][\s\S]*?\[\/\1\]/gi, '')
    .replace(/\[url[^\]]*\]([\s\S]*?)\[\/url\]/gi, '$1')
    .replace(/\[\/?[a-z]+\]/gi, '')
    .trim();
}

const devName = (d) => d.name || d.original || '';

/** 比对时两边形态不同：表单那边是逗号串，VNDB 那边是对象数组 */
function devList(value) {
  if (typeof value === 'string') return value.split(',');
  return value.map(devName);
}

/** 标签与开发商都是集合语义：顺序不同不算冲突 */
function sameNames(a, b, toList) {
  const x = toList(a).map((s) => String(s).trim()).filter(Boolean).sort();
  const y = toList(b).map((s) => String(s).trim()).filter(Boolean).sort();
  return x.length === y.length && x.every((s, i) => s === y[i]);
}

const asString = (v) => (v === null || v === undefined ? '' : String(v));

/** key 同时是表单字段名与 i18n 词条的尾段；read 取出的值就是该填进表单的形态 */
const FIELDS = [
  { key: 'name', read: (vn) => vn.title ?? '', empty: (v) => !v?.trim(), same: (a, b) => a === b },
  { key: 'alias', read: (vn) => vn.alttitle ?? '', empty: (v) => !v?.trim(), same: (a, b) => a === b },
  { key: 'released', read: (vn) => vn.released ?? '', empty: (v) => !v?.trim(), same: (a, b) => a === b },
  {
    key: 'rating',
    read: (vn) => asString(vn.rating),
    empty: (v) => !v?.trim(),
    same: (a, b) => Number(a) === Number(b),
  },
  {
    key: 'votecount',
    read: (vn) => asString(vn.votecount),
    empty: (v) => !v?.trim(),
    same: (a, b) => Number(a) === Number(b),
  },
  {
    key: 'description',
    read: (vn) => stripBbcode(vn.description),
    empty: (v) => !v?.trim(),
    same: (a, b) => a === b,
  },
  {
    key: 'tags',
    read: (vn) => vn.tags ?? [],
    empty: (v) => !v?.length,
    same: (a, b) => sameNames(a, b, (list) => list.map((t) => t.name)),
  },
  {
    key: 'developers',
    read: (vn) => devList(vn.developers ?? [])
      .map((d) => d.trim())
      .filter(Boolean)
      .join(', '),
    empty: (v) => !v?.trim(),
    same: (a, b) => sameNames(a, b, devList),
  },
  {
    key: 'coverPath',
    read: (vn) => vn.image?.local_path ?? '',
    empty: (v) => !v?.trim(),
    same: (a, b) => a === b,
  },
];

/** 表单这边为空的不算冲突 */
export function conflictRows(current, incoming) {
  return FIELDS.filter((f) => {
    if (f.empty(current[f.key])) return false;
    const value = f.read(incoming);
    if (f.empty(value)) return false;
    return !f.same(current[f.key], value);
  }).map((f) => ({ key: f.key, current: current[f.key], incoming: f.read(incoming) }));
}

/** 非空字段只在 selected 里才取 VNDB 的值，没选中的保持原样 */
export function applyIncoming(current, incoming, selected) {
  const next = { ...current };
  for (const f of FIELDS) {
    const value = f.read(incoming);
    if (f.empty(value)) continue;
    if (f.empty(current[f.key]) || selected.has(f.key)) next[f.key] = value;
  }
  return next;
}

/** 面板里显示用的字符串。tags 存的是对象数组，只取名字 */
export function asText(key, value) {
  if (key === 'tags') return (value ?? []).map((t) => t.name).join(' · ');
  return (value ?? '').toString();
}