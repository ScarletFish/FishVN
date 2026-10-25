const KEY = 'fishvn.theme';
const ACCENT_KEY = 'fishvn.accent';
const DARK_QUERY = '(prefers-color-scheme: dark)';

export const ACCENTS = ['default', 'amber', 'ocean', 'sakura', 'emerald', 'violet'];
export const THEMES = ['light', 'dark', 'system'];

let media = null;

function darkMedia() {
  if (!media) {
    media = window.matchMedia(DARK_QUERY);
    media.addEventListener('change', () => {
      if (getThemeMode() === 'system') paint(resolveTheme('system', media.matches) === 'dark');
    });
  }
  return media;
}

export function resolveTheme(mode, systemDark) {
  if (mode === 'dark') return 'dark';
  if (mode === 'system') return systemDark ? 'dark' : 'light';
  return 'light';
}

export function getThemeMode() {
  const saved = localStorage.getItem(KEY);
  return THEMES.includes(saved) ? saved : 'light';
}

function isDark() {
  return document.documentElement.dataset.theme === 'dark';
}

function paint(dark) {
  if (dark) document.documentElement.dataset.theme = 'dark';
  else delete document.documentElement.dataset.theme;
}

function paintAccent(accent) {
  if (ACCENTS.includes(accent) && accent !== 'default') {
    document.documentElement.dataset.accent = accent;
  } else {
    delete document.documentElement.dataset.accent;
  }
}

export function viewTransition(update) {
  if (document.startViewTransition && !window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    document.startViewTransition(update);
  } else {
    update();
  }
}

export function setTheme(mode) {
  const valid = THEMES.includes(mode) ? mode : 'light';
  paint(resolveTheme(valid, darkMedia().matches) === 'dark');
  localStorage.setItem(KEY, valid);
  return valid;
}

export function syncTheme(mode) {
  const valid = THEMES.includes(mode) ? mode : 'light';
  const dark = resolveTheme(valid, darkMedia().matches) === 'dark';
  if (isDark() === dark && localStorage.getItem(KEY) === valid) return;
  paint(dark);
  localStorage.setItem(KEY, valid);
}

export function getAccent() {
  return document.documentElement.dataset.accent || 'default';
}

export function setAccent(accent) {
  paintAccent(accent);
  localStorage.setItem(ACCENT_KEY, accent);
  return getAccent();
}

export function syncAccent(accent) {
  const valid = ACCENTS.includes(accent) ? accent : 'default';
  if (getAccent() === valid && localStorage.getItem(ACCENT_KEY) === valid) return;
  paintAccent(valid);
  localStorage.setItem(ACCENT_KEY, valid);
}

const FONT_SCALE_KEY = 'fishvn.fontScale';

export function getFontScale() {
  const raw = parseFloat(localStorage.getItem(FONT_SCALE_KEY));
  return Number.isFinite(raw) && raw >= 0.5 && raw <= 2 ? raw : 1;
}

export function setFontScale(scale) {
  const clamped = Math.min(2, Math.max(0.5, scale));
  document.documentElement.style.setProperty('--app-font-scale', String(clamped));
  localStorage.setItem(FONT_SCALE_KEY, String(clamped));
  return clamped;
}

export function syncFontScale() {
  setFontScale(getFontScale());
}
