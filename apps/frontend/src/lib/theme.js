// 首帧由 index.html 的内联脚本定，切换只发生在运行时。两边的判断必须一致：
// 深色加 data-theme 属性、其余删属性，不要往 localStorage 写 'light'。
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