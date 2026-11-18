const DIGIT = /^[1-9]$/;

export function navKeyIndex(event, count) {
  if (!event || event.defaultPrevented) return null;
  if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return null;
  if (event.isComposing) return null;
  if (!DIGIT.test(event.key)) return null;
  const index = event.key.charCodeAt(0) - 49;
  return index < count ? index : null;
}

export function isSidebarToggleKey(event) {
  if (!event || event.defaultPrevented) return false;
  if (event.altKey || event.shiftKey) return false;
  if (!event.ctrlKey && !event.metaKey) return false;
  return event.key?.toLowerCase() === 'b';
}
