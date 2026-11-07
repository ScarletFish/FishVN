export const LIBRARY_TAGS_CHANGED = 'fishvn:library-tags-changed';

export const LIBRARY_TRAITS_CHANGED = 'fishvn:library-traits-changed';

export const LIBRARY_ITEM_UPDATED = 'fishvn:library-item-updated';

export const LIBRARY_CHANGED = 'fishvn:library-changed';

export const ADD_MENU_SUMMON = 'fishvn:add-menu-summon';

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
    handler();
  }

  function onEvent() {
    if (timer) return;
    timer = setTimeout(fire, Math.max(0, last + interval - Date.now()));
  }

  window.addEventListener(LIBRARY_ITEM_UPDATED, onEvent);
  return () => {
    window.removeEventListener(LIBRARY_ITEM_UPDATED, onEvent);
    if (timer) clearTimeout(timer);
  };
}
