import { t } from './i18n.js';

export const STAFF_ROLES = [
  'scenario',
  'director',
  'chardesign',
  'art',
  'music',
  'songs',
  'translator',
  'editor',
  'qa',
  'staff',
];

export function staffRoleLabel(role) {
  const key = `staffRoles.${role}`;
  const value = t(key);
  return value === key ? role : value;
}

export function staffName(entry) {
  return (entry.original || entry.name || '').trim();
}

function staffTip(entry) {
  const romaji = (entry.name ?? '').trim();
  return romaji && romaji !== staffName(entry) ? romaji : '';
}

export function groupStaff(list) {
  const groups = [];
  const index = new Map();
  for (const entry of list ?? []) {
    const name = staffName(entry);
    if (!name) continue;
    const label = staffRoleLabel(entry.role);
    if (!index.has(label)) {
      const group = { label, names: [] };
      index.set(label, group);
      groups.push(group);
    }
    index
      .get(label)
      .names.push({ name, note: (entry.note ?? '').trim(), tip: staffTip(entry) });
  }
  return groups;
}

export function staffText(list) {
  return groupStaff(list)
    .map((g) => `${g.label}：${g.names.map((n) => n.name).join(' · ')}`)
    .join('；');
}

function nameSet(group) {
  return new Set((group?.names ?? []).map((n) => n.name));
}

export function staffDiff(current, incoming) {
  const a = groupStaff(current);
  const b = groupStaff(incoming);
  const labels = [];
  for (const g of [...a, ...b]) {
    if (!labels.includes(g.label)) labels.push(g.label);
  }
  const diff = labels.filter((label) => {
    const x = nameSet(a.find((g) => g.label === label));
    const y = nameSet(b.find((g) => g.label === label));
    return x.size !== y.size || [...x].some((n) => !y.has(n));
  });
  return { labels: diff, rest: labels.length - diff.length };
}

export function staffGroupText(list, labels) {
  const groups = groupStaff(list);
  return labels
    .map((label) => {
      const names = (groups.find((g) => g.label === label)?.names ?? []).map((n) => n.name);
      return `${label}：${names.join(' · ') || '—'}`;
    })
    .join('\n');
}
