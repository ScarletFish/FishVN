
import { DICT_ZH } from './i18n-zh.js';

const DICT = {
  'zh-CN': DICT_ZH,
};

const FALLBACK = 'zh-CN';

function detectLocale() {
  const langs = navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const lang of langs) {
    if (DICT[lang]) return lang;
    const base = lang.split('-')[0];
    const hit = Object.keys(DICT).find((k) => k.split('-')[0] === base);
    if (hit) return hit;
  }
  return FALLBACK;
}

export function getLocale() {
  return detectLocale();
}

export function t(key, params) {
  const dict = DICT[detectLocale()] ?? DICT[FALLBACK];
  const value = key.split('.').reduce((o, k) => o?.[k], dict) ?? key;
  if (!params) return value;
  return value.replace(/\{(\w+)\}/g, (whole, name) => params[name] ?? whole);
}