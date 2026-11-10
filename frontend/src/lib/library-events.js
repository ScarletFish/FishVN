import { report } from './frontend-log.js';

export const LIBRARY_TAGS_CHANGED = 'fishvn:library-tags-changed';

export const LIBRARY_TRAITS_CHANGED = 'fishvn:library-traits-changed';

export const LIBRARY_ITEM_UPDATED = 'fishvn:library-item-updated';

export const LIBRARY_CHANGED = 'fishvn:library-changed';

export const ADD_MENU_SUMMON = 'fishvn:add-menu-summon';

export const PLAY_SESSION_STARTED = 'fishvn:play-session-started';
export const PLAY_SESSION_TICK = 'fishvn:play-session-tick';
export const PLAY_SESSION_ENDED = 'fishvn:play-session-ended';

export function notifyLibraryTagsChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_TAGS_CHANGED));
}

export function notifyLibraryTraitsChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_TRAITS_CHANGED));
}

export function notifyLibraryItemUpdated(gameId) {
  window.dispatchEvent(new CustomEvent(LIBRARY_ITEM_UPDATED, { detail: { gameId } }));
}

export function notifyLibraryChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_CHANGED));
}

export function notifyAddMenuSummon() {
  window.dispatchEvent(new CustomEvent(ADD_MENU_SUMMON));
}

export function onLibraryItemUpdated(handler) {
  function onEvent(ev) {
    handler(ev?.detail?.gameId ?? null);
  }

  window.addEventListener(LIBRARY_ITEM_UPDATED, onEvent);
  return () => window.removeEventListener(LIBRARY_ITEM_UPDATED, onEvent);
}

export function onLibraryItemsThrottled(handler, interval = 5000) {
  let last = 0;
  let timer = null;

  function fire() {
    timer = null;
    last = Date.now();
    report('debug', '库变更节流：触发');
    handler();
  }

  function onEvent() {
    if (timer) {
      report('debug', '库变更事件：已在队列，合并');
      return;
    }
    const delay = Math.max(0, last + interval - Date.now());
    report('debug', `库变更事件：${delay}ms 后触发`);
    timer = setTimeout(fire, delay);
  }

  window.addEventListener(LIBRARY_ITEM_UPDATED, onEvent);
  return () => {
    window.removeEventListener(LIBRARY_ITEM_UPDATED, onEvent);
    if (timer) clearTimeout(timer);
  };
}

export function notifyPlaySessionStarted(gameId) {
  window.dispatchEvent(new CustomEvent(PLAY_SESSION_STARTED, { detail: { gameId } }));
}

export function notifyPlaySessionTick(payload) {
  window.dispatchEvent(new CustomEvent(PLAY_SESSION_TICK, { detail: payload }));
}

export function notifyPlaySessionEnded(gameId, seconds) {
  window.dispatchEvent(new CustomEvent(PLAY_SESSION_ENDED, { detail: { gameId, seconds } }));
}

export function onPlaySessionStarted(handler) {
  function onEvent(ev) {
    handler(ev?.detail?.gameId ?? null);
  }
  window.addEventListener(PLAY_SESSION_STARTED, onEvent);
  return () => window.removeEventListener(PLAY_SESSION_STARTED, onEvent);
}

export function onPlaySessionTick(handler) {
  function onEvent(ev) {
    handler(ev?.detail ?? null);
  }
  window.addEventListener(PLAY_SESSION_TICK, onEvent);
  return () => window.removeEventListener(PLAY_SESSION_TICK, onEvent);
}

export function onPlaySessionEnded(handler) {
  function onEvent(ev) {
    handler(ev?.detail?.gameId ?? null, ev?.detail?.seconds ?? 0);
  }
  window.addEventListener(PLAY_SESSION_ENDED, onEvent);
  return () => window.removeEventListener(PLAY_SESSION_ENDED, onEvent);
}
