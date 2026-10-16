import { api } from './api.js';
import { readCache, writeCache } from './vndb-cache.js';

export function createVndbSearch() {
  let searching = $state(false);
  let searchError = $state('');
  let hits = $state([]);

  async function search(keyword) {
    const term = keyword.trim();
    if (!term || searching) return;

    searchError = '';

    const cached = readCache(term);
    if (cached) {
      hits = cached;
      searching = false;
      return;
    }

    hits = [];
    searching = true;
    try {
      const result = await api.searchVndb(term);
      writeCache(term, result);
      hits = result;
    } catch (err) {
      searchError = err.message;
    } finally {
      searching = false;
    }
  }

  function reset() {
    searching = false;
    searchError = '';
    hits = [];
  }

  return {
    get searching() {
      return searching;
    },
    get searchError() {
      return searchError;
    },
    get hits() {
      return hits;
    },
    search,
    reset,
  };
}