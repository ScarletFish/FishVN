import { api } from './api.js';
import { setTagGroups, tagCategory } from './tag-order.svelte.js';
import { getLibraryTags } from './library-tags.svelte.js';
import { getLibraryTraits } from './library-traits.svelte.js';

let tags = $state({});
let traits = $state({});
let pending = null;

export function loadTagDict() {
  pending ??= api
    .tagDictionary()
    .then((dict) => {
      tags = dict?.tags ?? {};
      traits = dict?.traits ?? {};
      setTagGroups(dict?.tag_groups ?? null);
    })
    .catch(() => {
      pending = null;
    });
  return pending;
}

export const tagKey = (item) => item?.id ?? '';

export function tagLabel(item) {
  const id = item?.id ?? null;
  const name = item?.name ?? '';
  return (id && tags[id]?.name) || name;
}

export function tagKeyLabel(key) {
  const hit = key && tags[key];
  if (hit) return hit.name;
  const lib = getLibraryTags().find((item) => item.id === key);
  return lib?.name ?? key;
}

export function traitLabel(trait) {
  const id = trait?.id ?? null;
  const name = trait?.name ?? '';
  return (id && traits[id]?.name) || name;
}

export function allTagItems() {
  return Object.entries(tags).map(([id, entry]) => ({
    key: id,
    label: entry.name,
    group: tagCategory(id),
    count: null,
    raw: { id, name: entry.name },
  }));
}

export function getTraitDict() {
  return traits;
}

export function traitItems() {
  return Object.entries(traits).map(([id, entry]) => ({
    key: id,
    label: entry.name,
    group: entry.group || null,
    raw: { id, name: entry.name, group: entry.group || null },
  }));
}

export function traitKeyLabel(key) {
  const hit = key && traits[key];
  if (hit) return hit.name;
  const lib = getLibraryTraits().find((t) => t.id === key);
  return lib?.name ?? key;
}
