const CARD_TITLE_KEY = 'fishvn.cardTitle';

export function getShowCardTitle() {
  return localStorage.getItem(CARD_TITLE_KEY) === '1';
}

export function setShowCardTitle(on) {
  if (on) document.documentElement.dataset.cardTitle = 'on';
  else delete document.documentElement.dataset.cardTitle;
  localStorage.setItem(CARD_TITLE_KEY, on ? '1' : '0');
  return on;
}

export function syncShowCardTitle() {
  const on = getShowCardTitle();
  if (on === (document.documentElement.dataset.cardTitle === 'on')) return;
  setShowCardTitle(on);
}
