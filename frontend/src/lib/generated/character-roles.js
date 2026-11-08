export const CHARACTER_SCOPES = [
  { id: "main_only", rank: 0, default: true },
  { id: "with_side", rank: 1, default: false },
];

export const DEFAULT_CHARACTER_SCOPE = "main_only";

export const ROLE_SPECS = [
  { id: "main", tier: 0, minScope: "main_only" },
  { id: "primary", tier: 1, minScope: "main_only" },
  { id: "side", tier: 2, minScope: "with_side" },
  { id: "appears", tier: 3, minScope: null },
];

export const UNKNOWN_ROLE_TIER = 4;

export function roleTier(role) {
  const id = String(role ?? "").trim();
  return ROLE_SPECS.find((spec) => spec.id === id)?.tier ?? UNKNOWN_ROLE_TIER;
}
