import { api } from './api.js';
import { setTagGroups } from './tag-order.svelte.js';
import { getLibraryTags } from './library-tags.svelte.js';

let tags = $state({});
let traits = $state({});
let pending = null;

const labelOf = (table, item) => {
  const id = item?.id;
  return (id && table[id]) || item?.name || item?.tag || '';
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

export function tagKeyLabel(key) {
  if (tags[key]) return tags[key];
  const lib = getLibraryTags().find((item) => (item.id ?? item.tag) === key);
  return lib?.tag ?? key;
}

export function traitLabel(trait) {
  return labelOf(traits, trait);
}
