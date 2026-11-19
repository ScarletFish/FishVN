import { api } from './api.js';
import { getExcludedTags } from './config-store.svelte.js';
import { LIBRARY_TAGS_CHANGED } from './library-events.js';
import { tagKey, tagLabel } from './tag-label.svelte.js';
import { tagCategory } from './tag-order.svelte.js';

let tags = $state([]);
let inflight = null;

export function getLibraryTags() {
  const excluded = new Set(getExcludedTags());
  return tags.filter((tag) => !excluded.has(tag.id));
}

export function getAllLibraryTags() {
  return tags;
}

export function loadLibraryTags() {
  inflight ??= api
    .libraryTags(true)
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

function toTagItemsFrom(source) {
  return source.map((tag) => ({
    key: tagKey(tag),
    label: tagLabel(tag),
    group: tagCategory(tag),
    count: tag.count ?? null,
    raw: tag,
  }));
}

export function toTagItems() {
  return toTagItemsFrom(getLibraryTags());
}

export function toAllTagItems() {
  return toTagItemsFrom(getAllLibraryTags());
}
