export const PHASE_I18N = {
  meta: 'mylist.backfillPhaseMeta',
  cover: 'mylist.backfillPhaseCover',
  characters: 'mylist.backfillPhaseCharacters',
  images: 'mylist.backfillPhaseImages',
};

const ERROR_LIMIT = 20;

export function createDockState() {
  return {
    runId: 0,
    running: false,
    paused: false,
    pausedIdle: false,
    rateLimited: false,
    done: 0,
    total: 0,
    phase: null,
    errors: [],
    errorCount: 0,
    overflow: 0,
    finished: null,
  };
}

export function reduce(state, ev) {
  if (!ev || !ev.phase) return state;
  state = { ...state };

  if (ev.run_id && ev.run_id !== state.runId) {
    state.runId = ev.run_id;
    state.running = false;
    state.paused = false;
    state.pausedIdle = false;
    state.rateLimited = false;
    state.done = 0;
    state.total = 0;
    state.phase = null;
    state.errors = [];
    state.errorCount = 0;
    state.overflow = 0;
    state.finished = null;
  }

  switch (ev.phase) {
    case 'backfill':
      state.running = true;
      state.finished = null;
      state.pausedIdle = false;
      if (ev.done != null) state.done = ev.done;
      if (ev.total != null) state.total = ev.total;
      break;
    case 'meta':
    case 'cover':
    case 'characters':
    case 'images':
      state.running = true;
      state.phase = ev.phase;
      break;
    case 'rate':
    case 'rate-limited':
      state.running = true;
      if (ev.error) state.rateLimited = true;
      break;
    case 'unmatched':
    case 'failed':
      state.running = true;
      if (ev.error) pushError(state, ev);
      break;
    case 'paused':
      state.running = true;
      state.pausedIdle = true;
      break;
    case 'finished':
      state.running = false;
      state.pausedIdle = false;
      break;
  }
  return state;
}

function pushError(state, ev) {
  state.errorCount += 1;
  const idx = state.errors.findIndex((e) => e.message === ev.error);
  if (idx >= 0) {
    state.errors = state.errors.map((e, i) =>
      i === idx ? { ...e, times: e.times + 1 } : e
    );
    return;
  }
  if (state.errors.length >= ERROR_LIMIT) {
    state.overflow += 1;
    return;
  }
  state.errors = [...state.errors, { phase: ev.phase, message: ev.error, times: 1 }];
}

export function seedFromStatus(state, status) {
  if (!status) return state;
  const next = { ...state, runId: status.run_id ?? state.runId };
  if (status.running) {
    next.running = true;
    next.paused = status.paused ?? false;
    next.pausedIdle = status.paused_idle ?? false;
    next.rateLimited = status.rate_limited ?? false;
    next.done = status.done ?? 0;
    next.total = status.total ?? 0;
    next.finished = null;
  } else if (status.finished) {
    next.running = false;
    next.finished = status.finished;
  } else {
    next.running = false;
    next.finished = null;
  }
  return next;
}

export function markFinished(state, report) {
  return { ...state, running: false, finished: report };
}

export function dockView(state, ui, cancelling = false) {
  if (ui.hidden) return { form: 'hidden' };
  const hasRun = state.runId !== 0;
  if (state.finished) {
    if (ui.closedRun === state.runId) return { form: 'hidden' };
    return { form: 'panel', variant: 'summary' };
  }
  if (state.running && hasRun) {
    if (ui.collapsedRun === state.runId) return { form: 'handle' };
    let variant = 'running';
    if (cancelling) variant = 'cancelling';
    else if (state.paused) variant = state.pausedIdle ? 'paused' : 'pausing';
    return { form: 'panel', variant };
  }
  return { form: 'hidden' };
}

export function currentIndex(state) {
  if (state.total <= 0) return 0;
  return Math.min(state.done + 1, state.total);
}
