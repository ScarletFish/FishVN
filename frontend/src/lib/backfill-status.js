export function createRun() {
  return {
    runId: 0,
    running: false,
    paused: false,
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
  if (!ev || !ev.phase || !ev.run_id) return state;

  if (ev.run_id !== state.runId) {
    state = { ...createRun(), runId: ev.run_id };
  } else {
    state = { ...state };
  }

  switch (ev.phase) {
    case 'backfill':
      state.finished = null;
      state.paused = false;
      if (ev.done != null) state.done = ev.done;
      if (ev.total != null) state.total = ev.total;
      break;
    case 'images':
      state.stage = 'images';
      if (ev.done != null) state.imagesDone = ev.done;
      if (ev.total != null) state.imagesTotal = ev.total;
      break;
    case 'paused':
      state.paused = true;
      break;
    case 'rate':
    case 'rate-limited':
      if (ev.error) state.rateLimited = true;
      break;
    case 'finished':
      state.paused = false;
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
    next.paused = Boolean(status.paused || status.paused_idle);
    next.rateLimited = status.rate_limited ?? false;
    next.done = status.done ?? 0;
    next.total = status.total ?? 0;
    next.stage = status.stage ?? '';
    next.imagesDone = status.images_done ?? 0;
    next.imagesTotal = status.images_total ?? 0;
    next.finished = null;
  } else if (status.finished) {
    next.running = false;
    next.paused = false;
    next.stage = '';
    next.finished = status.finished;
    next.done = status.done ?? next.done;
    next.total = status.total ?? next.total;
  } else {
    next.running = false;
    next.paused = false;
    next.stage = '';
    next.finished = null;
  }
  return next;
}
