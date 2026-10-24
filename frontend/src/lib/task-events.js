export const BACKFILL_REFRESH = 'backfill-refresh';

export function notifyBackfillRefresh() {
  window.dispatchEvent(new CustomEvent(BACKFILL_REFRESH));
}
