const SIDEBAR_KEY = 'fishvn.sidebarCollapsed';

export function getSidebarCollapsed() {
  return localStorage.getItem(SIDEBAR_KEY) === '1';
}

export function setSidebarCollapsed(collapsed) {
  localStorage.setItem(SIDEBAR_KEY, collapsed ? '1' : '0');
  return collapsed;
}
