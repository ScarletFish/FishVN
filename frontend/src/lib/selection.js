export const CONTENT_SELECTOR = '.shell__main, .modal-overlay';
export const FIELD_SELECTOR = 'input, textarea, [contenteditable]:not([contenteditable="false"])';
export const BUBBLE_MARGIN = 8;
export const BUBBLE_OFFSET = 10;

function anchorElement(node) {
  if (node instanceof Element) return node;
  return node?.parentElement ?? null;
}

export function selectionInfo(selection) {
  if (!selection || selection.isCollapsed || selection.rangeCount === 0) return null;
  const text = selection.toString();
  if (!text.trim()) return null;
  const el = anchorElement(selection.anchorNode);
  if (!el || !el.closest(CONTENT_SELECTOR) || el.closest(FIELD_SELECTOR)) return null;
  const rect = selection.getRangeAt(0).getBoundingClientRect();
  if (!rect.width && !rect.height) return null;
  return { text, rect };
}

export function placeBubble(cx, top, width, height, viewport) {
  const left = Math.min(
    Math.max(BUBBLE_MARGIN, cx - width / 2),
    Math.max(BUBBLE_MARGIN, viewport.width - width - BUBBLE_MARGIN),
  );
  const above = top - height - BUBBLE_OFFSET;
  const y = above >= BUBBLE_MARGIN ? above : top + BUBBLE_OFFSET + 14;
  return { left, top: y };
}
