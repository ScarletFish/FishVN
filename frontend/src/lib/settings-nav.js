export const SETTINGS_SECTIONS = [
  { id: 'appearance', label: 'settings.tab.appearance' },
  { id: 'content', label: 'settings.tab.content' },
  { id: 'startup', label: 'settings.tab.startup' },
  { id: 'vndb', label: 'settings.vndb.tab' },
  { id: 'backup', label: 'settings.tab.backup' },
  { id: 'about', label: 'settings.tab.about' },
];

export function pickActiveSection(tops, line = 0, atBottom = false) {
  if (tops.length === 0) return null;
  if (atBottom) return tops[tops.length - 1].id;
  let active = tops[0].id;
  for (const entry of tops) {
    if (entry.top > line) break;
    active = entry.id;
  }
  return active;
}
