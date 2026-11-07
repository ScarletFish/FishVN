export const myListSort = $state({ by: 'name' });

export const dashboardSort = $state({ by: 'name' });

export function gameComparator(by) {
  if (by === 'vndb') return (a, b) => (b.vndb_rating ?? -1) - (a.vndb_rating ?? -1);
  if (by === 'user') return (a, b) => (b.user_rating ?? -1) - (a.user_rating ?? -1);
  if (by === 'added') return (a, b) => b.created_at - a.created_at;
  return (a, b) => String(a.name ?? '').localeCompare(String(b.name ?? ''), 'zh-CN');
}
