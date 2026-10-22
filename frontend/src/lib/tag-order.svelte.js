const ROOT_GROUPS = {
  g1: 'theme',
  g20: 'character',
  g22: 'setting',
  g24: 'plot',
  g674: 'style',
};

const ROUND_ROBIN_ORDER = ['theme', 'plot', 'character', 'setting', 'style'];

const SPOILER_THRESHOLD = { hide_all: 1, hide_major: 2, show_all: Infinity };

let roots = $state(null);
let excluded = $state(null);

export function setTagGroups(groups) {
  roots = groups?.roots ?? null;
  excluded = new Set(groups?.excluded ?? []);
}

export function spoilerThreshold(level) {
  return SPOILER_THRESHOLD[level] ?? 1;
}

export function orderTagsForDisplay(tags, hideR18, spoilerLevel = 'hide_all', excludedTags = []) {
  const threshold = spoilerThreshold(spoilerLevel);
  const pool = new Set(excludedTags ?? []);
  const list = (tags ?? []).filter(
    (tag) =>
      !tag.lie &&
      !(hideR18 && tag.category === 'ero') &&
      !(tag.id && excluded?.has(tag.id)) &&
      !(tag.id ? pool.has(tag.id) : pool.has(tag.name))
  );
  const visible = [];
  const spoilers = [];
  for (const tag of list) {
    if ((tag.spoiler ?? 0) >= threshold) spoilers.push(tag);
    else visible.push(tag);
  }
  if (!roots) {
    return { visible: visible.sort(byRating), spoilers: spoilers.sort(byRating) };
  }
  return { visible: order(visible), spoilers: order(spoilers) };
}

function order(items) {
  const groups = { custom: [], theme: [], plot: [], character: [], setting: [], style: [], other: [] };
  for (const tag of items) {
    const key = tag.id ? (roots[tag.id] && ROOT_GROUPS[roots[tag.id]]) || 'other' : 'custom';
    groups[key].push(tag);
  }
  for (const key of ROUND_ROBIN_ORDER) {
    groups[key].sort(byRating);
  }
  const out = [...groups.custom];
  const maxRounds = Math.max(...ROUND_ROBIN_ORDER.map((key) => groups[key].length));
  for (let round = 0; round < maxRounds; round++) {
    for (const key of ROUND_ROBIN_ORDER) {
      const items = groups[key];
      if (round < items.length) out.push(items[round]);
    }
  }
  return [...out, ...groups.other];
}

function byRating(a, b) {
  return (b.rating ?? 0) - (a.rating ?? 0) || (a.vn_count ?? 0) - (b.vn_count ?? 0);
}
