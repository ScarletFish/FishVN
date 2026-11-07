export function createQueryRunner({ query, store, onSettled }) {
  let seq = 0;
  let reloads = 0;
  let queries = 0;
  let applied = 0;
  let dropped = 0;
  let failed = 0;

  function paint(spec) {
    const subject = spec.subject;
    if (store.resultsSubject === subject) return true;
    const cached = store.resultsCache[subject];
    store.results = cached ?? [];
    store.resultsSubject = subject;
    return Array.isArray(cached) && cached.length > 0;
  }

  async function load(spec, { push = false } = {}) {
    if (push) reloads += 1;
    queries += 1;
    const mine = ++seq;
    const subject = spec.subject;
    const startedAt = Date.now();
    try {
      const rows = await query(spec);
      if (mine !== seq) {
        dropped += 1;
        return false;
      }
      store.resultsCache[subject] = rows;
      store.results = rows;
      store.resultsSubject = subject;
      store.lastJson[subject] = JSON.stringify(spec);
      applied += 1;
      onSettled?.(null, { subject, rows: rows.length, ms: Date.now() - startedAt });
      return true;
    } catch (e) {
      if (mine !== seq) {
        dropped += 1;
        return false;
      }
      failed += 1;
      onSettled?.(e?.message ?? String(e), { subject, rows: 0, ms: Date.now() - startedAt });
      return false;
    }
  }

  function stats() {
    return { reloads, queries, applied, dropped, failed };
  }

  return { paint, load, stats };
}
