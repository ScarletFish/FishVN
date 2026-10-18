
export function stripBbcode(text) {
  if (!text) return '';
  return text
    .replace(/\[(spoiler|raw)\][\s\S]*?\[\/\1\]/gi, '')
    .replace(/\[url[^\]]*\]([\s\S]*?)\[\/url\]/gi, '$1')
    .replace(/\[\/?[a-z]+\]/gi, '')
    .trim();
}

export function bbcodeSegments(text) {
  if (!text) return [];
  const segments = [];
  const re = /\[url=([^\]]*)\]([\s\S]*?)\[\/url\]|\[url\]([\s\S]*?)\[\/url\]/gi;
  let last = 0;
  let m;
  while ((m = re.exec(text))) {
    const plain = stripBbcode(text.slice(last, m.index));
    if (plain) segments.push({ type: 'text', text: plain });
    const href = (m[1] || m[3] || m[2]).trim();
    const label = stripBbcode(m[2] || m[3]) || href;
    if (/^https?:\/\//i.test(href)) {
      segments.push({ type: 'link', text: label, href });
    } else if (label) {
      segments.push({ type: 'text', text: label });
    }
    last = m.index + m[0].length;
  }
  const tail = stripBbcode(text.slice(last));
  if (tail) segments.push({ type: 'text', text: tail });
  return segments;
}

export function bbcodeParagraphs(raw) {
  return (raw ?? '')
    .split('\n\n')
    .map((para) => bbcodeSegments(para))
    .filter((segments) => segments.length);
}

const devName = (d) => d.name || d.original || '';

function displayTitle(vn) {
  const zh = (vn.titles ?? []).filter((x) => x.lang === 'zh-Hans' || x.lang === 'zh-Hant');
  const preferred = zh.find((x) => x.official !== false) ?? zh[0];
  if (preferred) return preferred.title ?? '';
  if (vn.olang === 'ja') return vn.alttitle || vn.title || '';
  return vn.title ?? '';
}

function aliasTitle(vn) {
  return displayTitle(vn) === (vn.title ?? '') ? (vn.alttitle ?? '') : (vn.title ?? '');
}

function devList(value) {
  if (typeof value === 'string') return value.split(',');
  return value.map(devName);
}

function sameNames(a, b, toList) {
  const x = toList(a).map((s) => String(s).trim()).filter(Boolean).sort();
  const y = toList(b).map((s) => String(s).trim()).filter(Boolean).sort();
  return x.length === y.length && x.every((s, i) => s === y[i]);
}

const staffKey = (entry) =>
  [entry.role, entry.original ?? '', entry.name ?? '', entry.note ?? ''].join('\u0000');

function sameStaff(a, b) {
  const x = (a ?? []).map(staffKey).sort();
  const y = (b ?? []).map(staffKey).sort();
  return x.length === y.length && x.every((s, i) => s === y[i]);
}

const asString = (v) => (v === null || v === undefined ? '' : String(v));

const FIELDS = [
  { key: 'name', read: (vn) => displayTitle(vn), empty: (v) => !v?.trim(), same: (a, b) => a === b },
  { key: 'alias', read: (vn) => aliasTitle(vn), empty: (v) => !v?.trim(), same: (a, b) => a === b },
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
    key: 'length_minutes',
    read: (vn) => asString(vn.length_minutes),
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
    key: 'staff',
    read: (vn) => vn.staff ?? [],
    empty: (v) => !v?.length,
    same: sameStaff,
  },
  {
    key: 'coverPath',
    read: (vn) => vn.image?.local_path ?? '',
    empty: (v) => !v?.trim(),
    same: (a, b) => a === b,
  },
];

export function conflictRows(current, incoming) {
  return FIELDS.filter((f) => {
    const value = f.read(incoming);
    if (f.empty(value)) return false;
    return f.empty(current[f.key]) || !f.same(current[f.key], value);
  }).map((f) => ({ key: f.key, current: current[f.key], incoming: f.read(incoming) }));
}

export function applyIncoming(current, incoming, selected) {
  const next = { ...current };
  for (const f of FIELDS) {
    const value = f.read(incoming);
    if (f.empty(value)) continue;
    if (selected.has(f.key)) next[f.key] = value;
  }
  return next;
}

export function asText(key, value) {
  if (key === 'tags') return (value ?? []).map((t) => t.name).join(' · ');
  return (value ?? '').toString();
}