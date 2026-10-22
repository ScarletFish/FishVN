import { api } from './api.js';
import { LIBRARY_TAGS_CHANGED } from './library-events.js';

let tags = $state([]);
let inflight = null;

export function getLibraryTags() {
  return tags;
}

export function loadLibraryTags() {
  inflight ??= api
    .libraryTags()
    .then((list) => {
      tags = Array.isArray(list) ? list : [];
    })
    .catch(() => {})
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

if (typeof window !== 'undefined') {
  window.addEventListener(LIBRARY_TAGS_CHANGED, loadLibraryTags);
}
