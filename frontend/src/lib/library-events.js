export const LIBRARY_TAGS_CHANGED = 'fishvn:library-tags-changed';

export const LIBRARY_CHANGED = 'fishvn:library-changed';

export function notifyLibraryTagsChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_TAGS_CHANGED));
}

export function notifyLibraryChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_CHANGED));
}
