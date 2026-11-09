export const TAG_ROOT_GROUPS = {
  "g1": "theme",
  "g20": "character",
  "g22": "setting",
  "g24": "plot",
  "g674": "style",
};

export const TAG_SCOPE = {
  character: "full",
  custom: "full",
  other: "full",
  plot: "detail",
  setting: "detail",
  style: "detail",
  tech: "hidden",
  theme: "full",
};

export const TAG_ROUND_ROBIN = ["theme", "plot", "character", "setting", "style"];

export const TAG_GROUP_ORDER = ["custom", "theme", "character", "setting", "plot", "style", "other", "tech"];

export function tagRootCategory(roots, id) {
  if (!id) return 'other';
  return TAG_ROOT_GROUPS[roots?.[id]] ?? 'other';
}
export function scopeOfCategory(category) {
  return TAG_SCOPE[category] ?? 'full';
}
