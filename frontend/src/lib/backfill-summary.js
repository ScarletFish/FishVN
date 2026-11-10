import { t } from './i18n.js';

export function backfillSummaryParts(r) {
  if (!r) return [];
  const processed = r.fetched + r.unmatched + r.failed;
  const parts = [
    r.cancelled
      ? t('mylist.backfillProcessed', { done: processed, total: r.total })
      : t('mylist.backfillOk', { done: r.fetched, total: r.total }),
  ];
  if (r.cancelled && r.fetched > 0) parts.push(t('mylist.backfillOkCount', { count: r.fetched }));
  if (r.unmatched > 0) parts.push(t('mylist.backfillUnmatched', { count: r.unmatched }));
  if (r.failed > 0) parts.push(t('mylist.backfillFailed', { count: r.failed }));
  return parts;
}

export function backfillSummaryText(r) {
  return backfillSummaryParts(r).join('，');
}
