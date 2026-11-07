const MYLIST_KEY = 'fishvn.mylistSort';
const DASHBOARD_KEY = 'fishvn.dashboardSort';
const VALID = new Set(['name', 'vndb', 'user', 'added']);

function load(key) {
  try {
    const v = localStorage.getItem(key);
    return VALID.has(v) ? v : 'name';
  } catch {
    return 'name';
  }
}

export const myListSort = $state({ by: load(MYLIST_KEY) });

export const dashboardSort = $state({ by: load(DASHBOARD_KEY) });

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
});

export function gameComparator(by) {
  if (by === 'vndb') return (a, b) => (b.vndb_rating ?? -1) - (a.vndb_rating ?? -1);
  if (by === 'user') return (a, b) => (b.user_rating ?? -1) - (a.user_rating ?? -1);
  if (by === 'added') return (a, b) => b.created_at - a.created_at;
  return (a, b) => String(a.name ?? '').localeCompare(String(b.name ?? ''), 'zh-CN');
}
