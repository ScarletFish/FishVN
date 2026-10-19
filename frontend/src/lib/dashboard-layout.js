export const DASHBOARD_MODULES = [
  { id: 'continue_play', label: 'dashboard.module.continuePlay' },
  { id: 'stats', label: 'dashboard.module.stats' },
  { id: 'local_library', label: 'dashboard.module.localLibrary' },
];

export function moveModule(list, id, delta) {
  const out = [...list];
  const from = out.findIndex((m) => m.id === id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= out.length) return out;
  const [item] = out.splice(from, 1);
  out.splice(to, 0, item);
  return out;
}

export function setModuleEnabled(list, id, enabled) {
  return list.map((m) => (m.id === id ? { ...m, enabled } : m));
}

export function reorderModule(list, fromId, toId) {
  const from = list.findIndex((m) => m.id === fromId);
  const to = list.findIndex((m) => m.id === toId);
  if (from < 0 || to < 0 || from === to) return list;
  const out = [...list];
  const [item] = out.splice(from, 1);
  out.splice(to, 0, item);
  return out;
}
