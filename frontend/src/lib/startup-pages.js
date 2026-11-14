export const STARTUP_PAGE_IDS = ['home', 'showcase', 'mylist', 'query', 'backfill'];

export function normalizeStartupPage(value) {
  return STARTUP_PAGE_IDS.includes(value) ? value : 'home';
}
