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
