const KEY = 'fishvn.theme';

function isDark() {
  return document.documentElement.dataset.theme === 'dark';
}

function paint(dark) {
  if (dark) document.documentElement.dataset.theme = 'dark';
  else delete document.documentElement.dataset.theme;
}

export function getTheme() {
  return isDark() ? 'dark' : 'light';
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