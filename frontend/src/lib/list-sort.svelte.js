const MYLIST_KEY = 'fishvn.mylistSort';
const DASHBOARD_KEY = 'fishvn.dashboardSort';
const MYLIST_GROUP_KEY = 'fishvn.mylistGroup';
const DASHBOARD_GROUP_KEY = 'fishvn.dashboardGroup';
const VALID = new Set(['name', 'vndb', 'user', 'added']);
const GROUP_VALID = new Set(['status', 'flat']);

function load(key, valid, fallback) {
  try {
    const v = localStorage.getItem(key);
    return valid.has(v) ? v : fallback;
  } catch {
    return fallback;
  }
}

export const myListSort = $state({ by: load(MYLIST_KEY, VALID, 'name') });
export const dashboardSort = $state({ by: load(DASHBOARD_KEY, VALID, 'name') });
export const myListGroup = $state({ by: load(MYLIST_GROUP_KEY, GROUP_VALID, 'status') });
export const dashboardGroup = $state({ by: load(DASHBOARD_GROUP_KEY, GROUP_VALID, 'status') });

$effect.root(() => {
  $effect(() => {
    try {
      localStorage.setItem(MYLIST_KEY, myListSort.by);
    } catch {}
  });
  $effect(() => {
    try {
      localStorage.setItem(DASHBOARD_KEY, dashboardSort.by);
    } catch {}
  });
  $effect(() => {
    try {
      localStorage.setItem(MYLIST_GROUP_KEY, myListGroup.by);
    } catch {}
  });
  $effect(() => {
    try {
      localStorage.setItem(DASHBOARD_GROUP_KEY, dashboardGroup.by);
    } catch {}
  });
});

export function gameComparator(by) {
  if (by === 'vndb') return (a, b) => (b.vndb_rating ?? -1) - (a.vndb_rating ?? -1);
  if (by === 'user') return (a, b) => (b.user_rating ?? -1) - (a.user_rating ?? -1);
  if (by === 'added') return (a, b) => b.created_at - a.created_at;
  return (a, b) => String(a.name ?? '').localeCompare(String(b.name ?? ''), 'zh-CN');
}

function compareSortName(a, b) {
  const x = a.sort_name?.trim() ?? '';
  const y = b.sort_name?.trim() ?? '';
  if (!x && !y) return 0;
  if (!x) return 1;
  if (!y) return -1;
  return x.localeCompare(y, 'zh-CN');
}

export function groupBySeries(items) {
  if (!items?.length) return items ?? [];

  const buckets = new Map();
  for (const item of items) {
    const series = item.series_name?.trim();
    if (!series) continue;
    if (!buckets.has(series)) buckets.set(series, []);
    buckets.get(series).push(item);
  }
  for (const [series, members] of buckets) {
    if (members.length < 2) buckets.delete(series);
  }

  const emitted = new Set();
  const out = [];
  for (const item of items) {
    const series = item.series_name?.trim();
    if (series && buckets.has(series)) {
      if (emitted.has(series)) continue;
      emitted.add(series);
      out.push(...buckets.get(series).slice().sort(compareSortName));
    } else {
      out.push(item);
    }
  }
  return out;
}
