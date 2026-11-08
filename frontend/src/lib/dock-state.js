export function createDockState() {
  return {
    runId: 0,
    running: false,
    pausedIdle: false,
    rateLimited: false,
    done: 0,
    total: 0,
    stage: '',
    imagesDone: 0,
    imagesTotal: 0,
    finished: null,
  };
}

export function reduce(state, ev) {
  if (!ev || !ev.phase) return state;

  if (ev.run_id && ev.run_id !== state.runId) {
    state = { ...createDockState(), runId: ev.run_id };
  } else {
    state = { ...state };
  }

  switch (ev.phase) {
    case 'backfill':
      state.finished = null;
      state.pausedIdle = false;
      if (ev.done != null) state.done = ev.done;
      if (ev.total != null) state.total = ev.total;
      break;
    case 'images':
      state.stage = 'images';
      if (ev.done != null) state.imagesDone = ev.done;
      if (ev.total != null) state.imagesTotal = ev.total;
      break;
    case 'paused':
      state.pausedIdle = true;
      break;
    case 'rate':
    case 'rate-limited':
      if (ev.error) state.rateLimited = true;
      break;
    case 'finished':
      state.pausedIdle = false;
      state.stage = '';
      break;
  }

  state.running = ev.phase !== 'finished';
  return state;
}

export function seedFromStatus(state, status) {
  if (!status) return state;
  const next = { ...state, runId: status.run_id ?? state.runId };
  if (status.running) {
    next.running = true;
    next.pausedIdle = status.paused_idle ?? false;
    next.rateLimited = status.rate_limited ?? false;
    next.done = status.done ?? 0;
    next.total = status.total ?? 0;
    next.stage = status.stage ?? '';
    next.imagesDone = status.images_done ?? 0;
    next.imagesTotal = status.images_total ?? 0;
    next.finished = null;
  } else if (status.finished) {
    next.running = false;
    next.stage = '';
    next.finished = status.finished;
  } else {
    next.running = false;
    next.stage = '';
    next.finished = null;
  }
  return next;
}

export function markFinished(state, report) {
  return { ...state, running: false, pausedIdle: false, finished: report };
}

export function dockVisible(state, ui) {
  if (ui.hidden) return false;
  if (state.finished) return !state.finished.cancelled && ui.closedRun !== state.runId;
  return state.running && !state.pausedIdle && state.runId !== 0;
}
