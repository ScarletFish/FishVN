const FORM_TAB_KEY = 'fishvn.formTab';

const TABS = ['basic', 'profile', 'tags', 'installations', 'characters'];

export function resolveFormTab(value) {
  return TABS.includes(value) ? value : 'basic';
}

export function getFormTab() {
  return resolveFormTab(localStorage.getItem(FORM_TAB_KEY));
}

export function setFormTab(tab) {
  localStorage.setItem(FORM_TAB_KEY, tab);
}
