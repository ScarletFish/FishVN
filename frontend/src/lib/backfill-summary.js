import { t } from './i18n.js';

export function backfillSummaryText(r) {
  if (!r) return '';
  const head = r.cancelled
    ? t('mylist.backfillCancelled', { done: r.fetched + r.bound, total: r.total })
    : t('mylist.backfillFetched', { count: r.fetched });
  const parts = [head];
  if (r.bound > 0) parts.push(t('mylist.backfillBound', { count: r.bound }));
  if (r.unmatched > 0) parts.push(t('mylist.backfillUnmatched', { count: r.unmatched }));
  if (r.failed > 0) parts.push(t('mylist.backfillFailed', { count: r.failed }));
  return parts.join(' | ');
}
