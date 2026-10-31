export function requestSearchQuery(query) {
  const text = String(query ?? '').trim();
  if (!text) return;
  document.dispatchEvent(new CustomEvent('search-set-query', { detail: text }));
}
