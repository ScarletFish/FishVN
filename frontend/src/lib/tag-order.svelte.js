import {
  TAG_GROUP_ORDER,
  TAG_ROUND_ROBIN,
  scopeOfCategory,
  tagRootCategory,
} from './generated/tag-key.js';

export { TAG_GROUP_ORDER };

export const TRAIT_GROUP_ORDER = [
  'Hair',
  'Eyes',
  'Body',
  'Clothes',
  'Items',
  'Personality',
  'Role',
  'Engages in',
  'Subject of',
  'Engages in (Sexual)',
  'Subject of (Sexual)',
  'uncategorized',
];

const SPOILER_THRESHOLD = { hide_all: 1, hide_major: 2, show_all: Infinity };

let roots = $state(null);
let techIds = $state(null);

export function setTagGroups(groups) {
  roots = groups?.roots ?? null;
  techIds = new Set(groups?.excluded ?? []);
}

export function spoilerThreshold(level) {
  return SPOILER_THRESHOLD[level] ?? 1;
}

function idOf(tagOrId) {
  return typeof tagOrId === 'string' ? tagOrId : tagOrId?.id;
}

export function tagCategory(tagOrId) {
  const id = idOf(tagOrId);
  if (!id || id.startsWith('u')) return 'custom';
  if (techIds?.has(id)) return 'tech';
  return tagRootCategory(roots, id);
}

export function tagScope(tagOrId) {
  return scopeOfCategory(tagCategory(tagOrId));
}

export function classifyTags(tags, options = {}) {
  const {
    hideR18 = false,
    spoilerLevel = 'hide_all',
    excludedTags = [],
    revealSpoilers = false,
  } = options;
  const threshold = spoilerThreshold(spoilerLevel);
  const pool = new Set(excludedTags ?? []);
  const visible = [];
  const spoilers = [];
  let poolCount = 0;
  for (const tag of tags ?? []) {
    if (tag.lie) continue;
    if (hideR18 && tag.category === 'ero') continue;
    if (tagScope(tag) === 'hidden') continue;
    if (inPool(tag, pool)) {
      if (revealSpoilers || (tag.spoiler ?? 0) < threshold) poolCount += 1;
      continue;
    }
    if ((tag.spoiler ?? 0) >= threshold) spoilers.push(tag);
    else visible.push(tag);
  }
  return {
    shown: order(visible),
    spoilers: order(spoilers),
    poolCount,
  };
}

function inPool(tag, pool) {
  return pool.has(tag.id);
}

function order(items) {
  if (!roots) return [...items].sort(byRating);
  const groups = {
    custom: [],
    theme: [],
    plot: [],
    character: [],
    setting: [],
    style: [],
    other: [],
    tech: [],
  };
  for (const tag of items) groups[tagCategory(tag)].push(tag);
  for (const key of TAG_ROUND_ROBIN) {
    groups[key].sort(byRating);
  }
  const out = [...groups.custom];
  const maxRounds = Math.max(...TAG_ROUND_ROBIN.map((key) => groups[key].length));
  for (let round = 0; round < maxRounds; round++) {
    for (const key of TAG_ROUND_ROBIN) {
      const bucket = groups[key];
      if (round < bucket.length) out.push(bucket[round]);
    }
  }
  return [...out, ...groups.other, ...groups.tech];
}

function byRating(a, b) {
  return (b.rating ?? 0) - (a.rating ?? 0) || (a.vn_count ?? 0) - (b.vn_count ?? 0);
}
