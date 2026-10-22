export const LIBRARY_TAGS_CHANGED = 'fishvn:library-tags-changed';

export function notifyLibraryTagsChanged() {
  window.dispatchEvent(new CustomEvent(LIBRARY_TAGS_CHANGED));
}
