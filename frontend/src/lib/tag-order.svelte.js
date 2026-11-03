const ROOT_GROUPS = {
  g1: 'theme',
  g20: 'character',
  g22: 'setting',
  g24: 'plot',
  g674: 'style',
};

const SCOPE_OF = {
  custom: 'full',
  theme: 'full',
  character: 'full',
  setting: 'detail',
  plot: 'detail',
  style: 'detail',
  other: 'full',
  tech: 'hidden',
};

const ROUND_ROBIN_ORDER = ['theme', 'plot', 'character', 'setting', 'style'];

export const TAG_GROUP_ORDER = ['custom', 'theme', 'character', 'setting', 'plot', 'style', 'other', 'tech'];

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
  if (!id) return 'custom';
  if (techIds?.has(id)) return 'tech';
  return ROOT_GROUPS[roots?.[id]] ?? 'other';
}

export function tagScope(tagOrId) {
  return SCOPE_OF[tagCategory(tagOrId)];
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
  return pool.has(tag.id || tag.name);
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
  for (const key of ROUND_ROBIN_ORDER) {
    groups[key].sort(byRating);
  }
  const out = [...groups.custom];
  const maxRounds = Math.max(...ROUND_ROBIN_ORDER.map((key) => groups[key].length));
  for (let round = 0; round < maxRounds; round++) {
    for (const key of ROUND_ROBIN_ORDER) {
      const bucket = groups[key];
      if (round < bucket.length) out.push(bucket[round]);
    }
  }
  return [...out, ...groups.other, ...groups.tech];
}

function byRating(a, b) {
  return (b.rating ?? 0) - (a.rating ?? 0) || (a.vn_count ?? 0) - (b.vn_count ?? 0);
}
