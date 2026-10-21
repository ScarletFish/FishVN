const KEY = 'fishvn.theme';
const ACCENT_KEY = 'fishvn.accent';

export const ACCENTS = ['default', 'amber', 'ocean', 'sakura', 'emerald', 'violet'];

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

export function getTheme() {
  return isDark() ? 'dark' : 'light';
}

export function viewTransition(update) {
  if (document.startViewTransition && !window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    document.startViewTransition(update);
  } else {
    update();
  }
}

export function setTheme(theme) {
  const dark = theme === 'dark';
  paint(dark);
  localStorage.setItem(KEY, theme);
}

export function toggleTheme() {
  const next = isDark() ? 'light' : 'dark';
  setTheme(next);
  return next;
}

export function syncTheme(theme) {
  const dark = theme === 'dark';
  if (isDark() === dark && localStorage.getItem(KEY) === theme) return;
  paint(dark);
  localStorage.setItem(KEY, theme);
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
