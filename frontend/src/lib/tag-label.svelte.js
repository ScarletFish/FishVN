import { api } from './api.js';
import { setTagGroups, tagCategory } from './tag-order.svelte.js';
import { getLibraryTags } from './library-tags.svelte.js';

let tags = $state({});
let traits = $state({});
let pending = null;

const labelOf = (table, item) => {
  const id = item?.id;
  const hit = id && table[id];
  if (hit) return typeof hit === 'string' ? hit : hit.name;
  return item?.name || item?.tag || '';
};

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

export function tagLabel(tag) {
  return labelOf(tags, tag);
}

export function tagKey(item) {
  return item?.id ?? item?.tag ?? item?.name ?? '';
}

export function tagKeyLabel(key) {
  if (tags[key]) return tags[key];
  const lib = getLibraryTags().find((item) => (item.id ?? item.tag) === key);
  return lib?.tag ?? key;
}

export function traitLabel(trait) {
  return labelOf(traits, trait);
}

export function allTagItems() {
  return Object.entries(tags).map(([id, name]) => ({
    key: id,
    label: name,
    group: tagCategory(id),
    count: null,
    raw: { id, tag: name },
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
