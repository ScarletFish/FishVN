import { api } from './api.js';
import { LIBRARY_TRAITS_CHANGED } from './library-events.js';
import { traitLabel } from './tag-label.svelte.js';

let traits = $state([]);
let inflight = null;

export function getLibraryTraits() {
  return traits;
}

export function loadLibraryTraits() {
  inflight ??= api
    .libraryTraits()
    .then((list) => {
      traits = Array.isArray(list) ? list : [];
    })
    .catch(() => {})
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

if (typeof window !== 'undefined') {
  window.addEventListener(LIBRARY_TRAITS_CHANGED, loadLibraryTraits);
}

export function toTraitItems() {
  return getLibraryTraits().map((trait) => ({
    key: trait.id ?? trait.name,
    label: traitLabel(trait),
    group: trait.group ?? null,
    count: trait.count ?? null,
    raw: { id: trait.id ?? null, name: trait.name, group: trait.group ?? null },
  }));
}
