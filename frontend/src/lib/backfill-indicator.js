import { t } from './i18n.js';
import { backfillSummaryText } from './backfill-summary.js';

export function backfillIndicator(run, { current, seenRun }) {
  const dot = dotOf(run, current, seenRun);
  return { dot, tip: tipOf(run, dot) };
}

function dotOf(run, current, seenRun) {
  if (current === 'backfill' || run.runId === 0) return null;
  if (run.running) return 'running';
  if (run.finished && !run.finished.cancelled && seenRun !== run.runId) return 'done';
  return null;
}

function tipOf(run, dot) {
  if (dot === 'running') {
    if (run.paused) return t('backfillPage.paused');
    if (run.rateLimited) return t('mylist.backfillRateLimitedShort');
    if (run.stage === 'images') {
      return t('mylist.backfillImages', { done: run.imagesDone ?? 0, total: run.imagesTotal ?? 0 });
    }
    return t('mylist.backfillRunning', { done: run.done ?? 0, total: run.total ?? 0 });
  }
  if (dot === 'done') return backfillSummaryText(run.finished);
  return t('nav.backfill');
}
