export const ZH_LANGS = ["zh-Hans", "zh-Hant"];
export const JA_LANGS = ["ja"];
export const ZH_OFFICIAL_FIRST = true;

export function titleChain(input) {
  const out = [];
  const push = (raw) => {
    const text = String(raw ?? "").trim();
    if (text && !out.includes(text)) out.push(text);
  };
  const titles = input.titles ?? [];
  const zh = titles.filter((t) => ZH_LANGS.includes(t.lang));
  const preferred = ZH_OFFICIAL_FIRST
    ? zh.find((t) => t.official !== false) ?? zh[0]
    : zh[0];
  if (preferred) push(preferred.title);
  for (const t of titles.filter((t) => JA_LANGS.includes(t.lang))) push(t.title);
  if (input.olang === "ja") push(input.alttitle);
  push(input.latin);
  return out;
}
