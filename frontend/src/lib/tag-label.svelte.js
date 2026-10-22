import { api } from './api.js';

let tags = $state({});
let traits = $state({});
let pending = null;

const labelOf = (table, item) => {
  const id = item?.id;
  return (id && table[id]) || item?.name || '';
};

export function loadTagDict() {
  pending ??= api
    .tagDictionary()
    .then((dict) => {
      tags = dict?.tags ?? {};
      traits = dict?.traits ?? {};
    })
    .catch(() => {
      pending = null;
    });
  return pending;
}

export function tagLabel(tag) {
  return labelOf(tags, tag);
}

export function traitLabel(trait) {
  return labelOf(traits, trait);
}
