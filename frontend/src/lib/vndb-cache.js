const TTL = 10 * 60 * 1000;
const MAX_ENTRIES = 32;

const store = new Map();

export function readCache(keyword) {
  const hit = store.get(keyword);
  if (!hit) return null;
  if (Date.now() - hit.at >= TTL) {
    store.delete(keyword);
    return null;
  }
  store.delete(keyword);
  store.set(keyword, hit);
  return hit.data;
}

export function writeCache(keyword, data) {
  store.set(keyword, { data, at: Date.now() });
  while (store.size > MAX_ENTRIES) {
    store.delete(store.keys().next().value);
  }
}
